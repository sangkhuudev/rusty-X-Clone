use axum::{async_trait, http::StatusCode, Json};
use chrono::{Duration, Utc};
use diesel_async::AsyncPgConnection;
use tracing::info;
use uchat_crypto::{encode_base64, hash_password, password::deserialize_hash, verify_password};
use uchat_domain::user::DisplayName;
use uchat_endpoint::{
    app_url::construct_image_url,
    user::{endpoint::*, types::PublicUserProfile},
    Update,
};
use uchat_query::{
    session::{self, Session},
    user::{get_hashed_password, UpdateProfileParams, User},
    ImageId, UserId,
};

use crate::{
    error::ApiResult,
    extractor::{DbConnection, UserSession},
    AppState,
};

use super::{save_image, AuthorizedApiRequest, PublicApiRequest};

pub fn to_public(user: User) -> ApiResult<PublicUserProfile> {
    tracing::info!("Make profile public");

    // Use async and caching
    let profile_image_url = user.profile_image.as_ref().and_then(|id| {
        // Assuming an async and cached construct_image_url function
        tokio::task::block_in_place(|| construct_image_url(id)).ok()
    });

    Ok(PublicUserProfile {
        id: user.id,
        display_name: user
            .display_name
            .and_then(|name| DisplayName::try_new(name).ok()),
        handle: user.handle,
        profile_image: profile_image_url,
        created_at: user.created_at,
        am_following: false,
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
        let hashed_password = get_hashed_password(&mut conn, &self.username).await?;
        let hashed_password = deserialize_hash(&hashed_password)?;
        verify_password(self.password, &hashed_password)?;

        let user = uchat_query::user::find(&mut conn, &self.username).await?;
        info!(username = %self.username.as_ref(), "Login successfully.");

        let (session, signature, duration) = new_session(&mut conn, &state, user.id).await?;

        Ok((
            StatusCode::OK,
            Json(LoginOk {
                session_signature: signature.0,
                session_id: session.id,
                session_expires: Utc::now() + duration,
                display_name: user.display_name,
                email: user.email,
                profile_image: None,
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
            match construct_image_url(id) {
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
        let user = uchat_query::user::get(&mut conn, session.user_id).await?;

        let password = {
            if let Update::Change(ref password) = self.password {
                Update::Change(uchat_crypto::hash_password(password)?)
            } else {
                Update::NoChange
            }
        };

        if let Update::Change(ref img) = self.profile_image {
            let id = ImageId::new();
            save_image(id, img).await?;
        }
        tracing::info!("Fetching public posts...");

        let query_params = UpdateProfileParams {
            id: session.user_id,
            display_name: self.display_name,
            email: self.email,
            password_hash: password,
            profile_image: self.profile_image.clone(),
        };
        tracing::info!("Updating my profile...");
        uchat_query::user::update_profile(&mut conn, query_params).await?;

        let profile_image_url = if let Some(id) = &user.profile_image {
            match construct_image_url(id) {
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

        let profile = to_public(profile)?;

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
