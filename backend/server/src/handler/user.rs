use axum::{async_trait, http::StatusCode, Json};
use chrono::{Duration, Utc};
use tracing::{info, Instrument};
use uchat_crypto::{encode_base64, hash_password, password::deserialize_hash, verify_password};
use uchat_endpoint::user::endpoint::{CreateUser, CreateUserOk, Login, LoginOk};
use uchat_query::{session, user::get_hashed_password};

use crate::{error::ApiResult, extractor::DbConnection, AppState};

use super::PublicApiRequest;

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
            username = self.username.as_ref(),
            "New user created successfully."
        );

        Ok((
            StatusCode::CREATED,
            Json(CreateUserOk {
                user_id,
                username: self.username,
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

        // New session
        let (session, signature, duration) = {
            let fingerprint = serde_json::json!({});
            let session_duration = Duration::weeks(3);
            let session = session::new(
                &mut conn, 
                user.id, 
                session_duration, 
                fingerprint.into()
            )?;
            let mut rng = state.rng.clone();
            let signature = state
                .signing_keys
                .sign(&mut rng, session.id.as_uuid().as_bytes());
            let signature = encode_base64(signature.to_string());
            (session, signature, session_duration)
        };

        info!(
            username = self.username.as_ref(),
            "Login successfully."
        );

        Ok((
            StatusCode::OK,
            Json(LoginOk {
                session_signature: signature,
                session_id: session.id,
                session_expires: Utc::now() + duration,
                display_name: user.display_name,
                email: user.email,
                profile_image: None,
                user_id: user.id
            })
        ))
    }
}
