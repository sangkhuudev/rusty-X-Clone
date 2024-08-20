use anyhow::anyhow;
use axum::{async_trait, http::StatusCode, Json};
use chrono::{Duration, Utc};
use diesel_async::AsyncPgConnection;
use tracing::info;
use uchat_crypto::{encode_base64, hash_password, password::deserialize_hash, verify_password};
use uchat_domain::user::DisplayName;
use uchat_endpoint::{
    app_url::construct_image_url,
    user::{
        endpoint::*,
        types::{FollowAction, PublicUserProfile},
    },
    RequestFailed, Update,
};
use uchat_query::{
    session::{self, Session},
    user::{get_hashed_password, UpdateProfileParams, User},
    ImageId, UserId,
};

use crate::{
    error::{ApiError, ApiResult, ServerError},
    extractor::{DbConnection, UserSession},
    AppState,
};

use super::{save_image, AuthorizedApiRequest, PublicApiRequest};

#[tracing::instrument(
    name = "Make the post public",
    skip_all,
    fields(
        user_id = ?user.id,
    )
)]
pub async fn to_public(
    conn: &mut AsyncPgConnection,
    session: Option<&UserSession>,
    user: User,
) -> ApiResult<PublicUserProfile> {
    tracing::info!("Make profile public");

    let profile_image_url = if let Some(id) = &user.profile_image {
        match construct_image_url(id).await {
            Ok(url) => Some(url),
            Err(e) => {
                tracing::error!("Failed to construct profile image URL: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    Ok(PublicUserProfile {
        id: user.id,
        display_name: user
            .display_name
            .and_then(|name| DisplayName::try_new(name).ok()),
        handle: user.handle,
        profile_image: profile_image_url,
        created_at: user.created_at,
        am_following: {
            match session {
                Some(session) => {
                    uchat_query::user::is_following(conn, session.user_id, user.id).await?
                }
                None => false,
            }
        },
    })
}

#[derive(Debug, Clone)]
pub struct SessionSignature(String);

async fn new_session(
    conn: &mut AsyncPgConnection,
    state: &AppState,
    user_id: UserId,
) -> ApiResult<(Session, SessionSignature, Duration)> {
    // New session
    let fingerprint = serde_json::json!({});
    let session_duration = Duration::weeks(3);
    let session = session::new(conn, user_id, session_duration, fingerprint.into()).await?;
    let mut rng = state.rng.clone();
    let signature = state
        .signing_keys
        .sign(&mut rng, session.id.as_uuid().as_bytes());
    let signature = encode_base64(signature);
    Ok((session, SessionSignature(signature), session_duration))
}
#[async_trait]
impl PublicApiRequest for CreateUser {
    type Response = (StatusCode, Json<CreateUserOk>);

    #[tracing::instrument(
        name = "Creating user",
        skip_all,
        fields(username = %self.username)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response> {
        let hashed_password = hash_password(self.password)?;
        let user_id = uchat_query::user::new(&mut conn, hashed_password, &self.username).await?;

        info!(
            username = %self.username.as_ref(),
            "New user created successfully."
        );

        let (session, signature, duration) = new_session(&mut conn, &state, user_id).await?;
        Ok((
            StatusCode::CREATED,
            Json(CreateUserOk {
                user_id,
                username: self.username,
                session_id: session.id,
                session_signature: signature.0,
                session_expires: Utc::now() + duration,
            }),
        ))
    }
}

#[async_trait]
impl PublicApiRequest for Login {
    type Response = (StatusCode, Json<LoginOk>);
    #[tracing::instrument(
        name = "Logging in",
        skip_all,
        fields(username = %self.username)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response> {
        let hashed_password = get_hashed_password(&mut conn, &self.username)
            .await
            .map_err(|_| ServerError::wrong_password())?;
        let hashed_password =
            deserialize_hash(&hashed_password).map_err(|_| ServerError::wrong_password())?;
        verify_password(self.password, &hashed_password)
            .map_err(|_| ServerError::wrong_password())?;

        let user = uchat_query::user::find(&mut conn, &self.username)
            .await
            .map_err(|_| ServerError::missing_login())?;
        info!(username = %self.username.as_ref(), "Login successfully.");

        let (session, signature, duration) = new_session(&mut conn, &state, user.id).await?;

        let profile_image_url = if let Some(id) = &user.profile_image {
            match construct_image_url(id).await {
                Ok(url) => Some(url),
                Err(e) => {
                    tracing::error!("Failed to construct profile image URL: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok((
            StatusCode::OK,
            Json(LoginOk {
                session_signature: signature.0,
                session_id: session.id,
                session_expires: Utc::now() + duration,
                display_name: user.display_name,
                email: user.email,
                profile_image: profile_image_url,
                user_id: user.id,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for GetMyProfile {
    type Response = (StatusCode, Json<GetMyProfileOk>);
    #[tracing::instrument(
        name = "Get my profile",
        skip_all,
        fields(user_id = ?session.user_id)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let user = uchat_query::user::get(&mut conn, session.user_id).await?;

        tracing::info!("Getting profile...");
        let profile_image_url = if let Some(id) = &user.profile_image {
            match construct_image_url(id).await {
                Ok(url) => Some(url),
                Err(e) => {
                    tracing::error!("Failed to construct profile image URL: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        tracing::info!("Profile got sent.");
        Ok((
            StatusCode::OK,
            Json(GetMyProfileOk {
                user_id: user.id,
                display_name: user.display_name,
                email: user.email,
                profile_image: profile_image_url,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for UpdateProfile {
    type Response = (StatusCode, Json<UpdateProfileOk>);
    #[tracing::instrument(
        name = "Update my profile",
        skip_all,
        fields(dislay_name = ?self.display_name)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut payload = self;
        let user = uchat_query::user::get(&mut conn, session.user_id).await?;

        let password = {
            if let Update::Change(ref password) = payload.password {
                Update::Change(uchat_crypto::hash_password(password)?)
            } else {
                Update::NoChange
            }
        };

        if let Update::Change(ref img) = payload.profile_image {
            let id = ImageId::new();
            save_image(id, img).await?;
            // This line added to fix performance load image from the server
            // because everytime fetch something from the Frontend
            // The server send entire data which is huge memory
            // So we set image to absolute url
            payload.profile_image = Update::Change(id.to_string());
        }
        tracing::info!("Fetching public posts...");

        let query_params = UpdateProfileParams {
            id: session.user_id,
            display_name: payload.display_name,
            email: payload.email,
            password_hash: password,
            profile_image: payload.profile_image.clone(),
        };
        tracing::info!("Updating my profile...");
        uchat_query::user::update_profile(&mut conn, query_params).await?;

        let profile_image_url = if let Some(id) = &user.profile_image {
            match construct_image_url(id).await {
                Ok(url) => Some(url),
                Err(e) => {
                    tracing::error!("Failed to construct profile image URL: {:?}", e);
                    None
                }
            }
        } else {
            None
        };

        tracing::info!("Profile updated successfully");

        Ok((
            StatusCode::OK,
            Json(UpdateProfileOk {
                profile_image: profile_image_url,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for ViewProfile {
    type Response = (StatusCode, Json<ViewProfileOk>);
    #[tracing::instrument(
        name = "View public profile",
        skip_all,
        fields(for_user = ?self.for_user)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        tracing::info!("Getting profile from database...");
        let profile = uchat_query::user::get(&mut conn, self.for_user).await?;

        let profile = to_public(&mut conn, Some(&session), profile).await?;

        let mut posts = vec![];
        tracing::info!("Fetching public posts...");
        for post in uchat_query::post::get_public_posts(&mut conn, self.for_user).await? {
            let post_id = post.id;
            match super::post::to_public(&mut conn, post, Some(&session)).await {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(error = %e.error, post_id = ?post_id, "Post contains invalid data");
                }
            }
        }

        info!("Fetching public posts successfully");

        Ok((StatusCode::OK, Json(ViewProfileOk { profile, posts })))
    }
}

#[async_trait]
impl AuthorizedApiRequest for FollowUser {
    type Response = (StatusCode, Json<FollowUserOk>);
    #[tracing::instrument(
        name = "Follow a user",
        skip_all,
        fields(
            user_id = ?session.user_id,
            action = ?self.action
        )
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        // If the user itself: Can not follow self
        if self.user_id == session.user_id {
            return Err(ApiError {
                code: Some(StatusCode::BAD_REQUEST),
                error: anyhow!(RequestFailed {
                    msg: "Can't follow self".to_string()
                }),
            });
        }

        match self.action {
            FollowAction::Follow => {
                uchat_query::user::follow(&mut conn, session.user_id, self.user_id).await?;
            }
            FollowAction::Unfollow => {
                uchat_query::user::unfollow(&mut conn, session.user_id, self.user_id).await?;
            }
        }

        tracing::info!("Success in toggle following.");
        Ok((
            StatusCode::OK,
            Json(FollowUserOk {
                status: self.action,
            }),
        ))
    }
}
