use std::str::FromStr;

use crate::AppState;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::COOKIE, request::Parts, StatusCode},
    Extension, Json, RequestPartsExt,
};
use chrono::Utc;
use diesel_async::{pooled_connection::deadpool::Object, AsyncPgConnection};
use uchat_endpoint::RequestFailed;
use uchat_query::{SessionId, UserId};

pub struct DbConnection(pub Object<AsyncPgConnection>);

#[async_trait]
impl<S> FromRequestParts<S> for DbConnection
where
    S: Sync + Send,
{
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(state) = parts
            .extract::<Extension<AppState>>()
            .await
            .expect("Failed to extract AppState");
        let conn = state.db_pool.get().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to connect to database",
            )
        })?;

        Ok(Self(conn))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UserSession {
    pub user_id: UserId,
    pub session_id: SessionId,
}

#[async_trait]
impl<S> FromRequestParts<S> for UserSession
where
    S: Sync + Send,
{
    type Rejection = (StatusCode, Json<RequestFailed>);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        tracing::debug!("Starting extraction of UserSession");

        // Debug log the extracted cookies
        let unauthorized = || {
            (
                StatusCode::UNAUTHORIZED,
                Json(RequestFailed {
                    msg: "Unauthorized".into(),
                }),
            )
        };

        // Extract DbConnection
        let DbConnection(mut conn) = parts.extract::<DbConnection>().await.map_err(|err| {
            tracing::debug!("Failed to extract DbConnection: {:?}", err);
            unauthorized()
        })?;

        // Extract AppState
        let Extension(state) = parts
            .extract::<Extension<AppState>>()
            .await
            .map_err(|err| {
                tracing::debug!("Failed to extract AppState: {:?}", err);
                unauthorized()
            })?;

        // Extract Cookies
        let cookies = parts
            .headers
            .get(COOKIE)
            .and_then(|header| header.to_str().ok());

        match cookies {
            Some(cookies) => {
                let session_id = uchat_cookie::get_from_str(cookies, uchat_cookie::SESSION_ID)
                    .ok_or_else(|| {
                        tracing::debug!("Failed to extract SESSION_ID from cookies");
                        unauthorized()
                    })?;

                let session_signature =
                    uchat_cookie::get_from_str(cookies, uchat_cookie::SESSION_SIGNATURE)
                        .ok_or_else(|| {
                            tracing::debug!("Failed to extract SESSION_SIGNATURE from cookies");
                            unauthorized()
                        })?;

                let session_id = SessionId::from_str(session_id).map_err(|_| {
                    tracing::debug!("Failed to parse SESSION_ID");
                    unauthorized()
                })?;

                let session_signature = uchat_crypto::decode_base64(session_signature)
                    .ok()
                    .and_then(|sig| uchat_crypto::sign::signature_from_bytes(sig).ok())
                    .ok_or_else(|| {
                        tracing::debug!("Failed to decode SESSION_SIGNATURE");
                        unauthorized()
                    })?;

                tracing::debug!("Verifying session signature...");
                state
                    .signing_keys
                    .verify(session_id.as_uuid().as_bytes(), session_signature)
                    .map_err(|_| {
                        tracing::debug!("Session signature verification failed");
                        unauthorized()
                    })?;

                let session = uchat_query::session::get(&mut conn, session_id)
                    .await
                    .map_err(|err| {
                        tracing::debug!("Failed to retrieve session: {:?}", err);
                        unauthorized()
                    })?
                    .ok_or_else(|| {
                        tracing::debug!("Session not found");
                        unauthorized()
                    })?;

                if session.expires_at < Utc::now() {
                    tracing::debug!("Session has expired");
                    return Err(unauthorized());
                }

                tracing::info!(
                    user_id = session.user_id.into_inner().to_string(),
                    "User logged in."
                );

                Ok(Self {
                    user_id: session.user_id,
                    session_id: session.id,
                })
            }
            None => {
                tracing::debug!("Failed to extract cookies from headers");
                Err(unauthorized())
            }
        }
    }
}
