use axum::{async_trait, http::StatusCode, Json};
use chrono::{Duration, Utc};
use tracing::info;
use uchat_crypto::{encode_base64, hash_password, password::deserialize_hash, verify_password};
use uchat_domain::user::DisplayName;
use uchat_endpoint::{
    user::{
        endpoint::{
            CreateUser, CreateUserOk, GetMyProfile, GetMyProfileOk, Login, LoginOk, UpdateProfile,
            UpdateProfileOk,
        },
        types::PublicUserProfile,
    },
    Update,
};
use uchat_query::{
    session::{self, Session},
    user::{get_hashed_password, UpdateProfileParams, User},
    AsyncConnection, ImageId, UserId,
};
use url::Url;

use crate::{
    error::ApiResult,
    extractor::{DbConnection, UserSession},
    AppState,
};

use super::{save_image, AuthorizedApiRequest, PublicApiRequest};

pub fn to_public(user: User) -> ApiResult<PublicUserProfile> {
    Ok(PublicUserProfile {
        id: user.id,
        dislay_name: user
            .display_name
            .and_then(|name| DisplayName::try_new(name).ok()),
        handle: user.handle,
        profile_image: None,
        created_at: user.created_at,
        am_following: false,
    })
}

fn profile_id_to_url(id: &str) -> Url {
    use uchat_endpoint::app_url::{self, user_content};
    app_url::domain_and(user_content::ROOT)
        .join(user_content::IMAGE)
        .unwrap()
        .join(id)
        .unwrap()
}
#[derive(Debug, Clone)]
pub struct SessionSignature(String);

fn new_session(
    conn: &mut AsyncConnection,
    state: &AppState,
    user_id: UserId,
) -> ApiResult<(Session, SessionSignature, Duration)> {
    // New session
    let fingerprint = serde_json::json!({});
    let session_duration = Duration::weeks(3);
    let session = session::new(conn, user_id, session_duration, fingerprint.into())?;
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
        let user_id = uchat_query::user::new(&mut conn, hashed_password, &self.username)?;

        info!(
            username = %self.username.as_ref(),
            "New user created successfully."
        );

        let (session, signature, duration) = new_session(&mut conn, &state, user_id)?;
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
        let hashed_password = get_hashed_password(&mut conn, &self.username)?;
        let hashed_password = deserialize_hash(&hashed_password)?;
        verify_password(self.password, &hashed_password)?;

        let user = uchat_query::user::find(&mut conn, &self.username)?;
        info!(username = %self.username.as_ref(), "Login successfully.");

        let (session, signature, duration) = new_session(&mut conn, &state, user.id)?;

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
        // fields(dislay_name = %self.dislay_name)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let user = uchat_query::user::get(&mut conn, session.user_id)?;

        tracing::info!("Getting profile...");
        let profile_image_url = user.profile_image.as_ref().map(|id| profile_id_to_url(id));

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
        // fields(dislay_name = %self.dislay_name)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let user = uchat_query::user::get(&mut conn, session.user_id)?;

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

        let query_params = UpdateProfileParams {
            id: session.user_id,
            display_name: self.display_name,
            email: self.email,
            password_hash: password,
            profile_image: self.profile_image.clone(),
        };
        tracing::info!("Updating my profile...");
        uchat_query::user::update_profile(&mut conn, query_params)?;

        let profile_image_url = user.profile_image.as_ref().map(|id| profile_id_to_url(id));

        tracing::info!("Profile updated successfully");

        Ok((
            StatusCode::OK,
            Json(UpdateProfileOk {
                profile_image: profile_image_url,
            }),
        ))
    }
}
