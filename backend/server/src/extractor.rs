use std::str::FromStr;

use axum::{
    async_trait, extract::FromRequestParts, http::{header, request::Parts, StatusCode}, Extension, Json, RequestPartsExt
};
use uchat_cookie::{SESSION_ID, SESSION_SIGNATURE};
use uchat_endpoint::RequestFailed;
use uchat_query::{OwnedAsyncConnection, SessionId, UserId};

use crate::AppState;

pub struct DbConnection(pub OwnedAsyncConnection);

#[async_trait]
impl<S> FromRequestParts<S> for DbConnection
where
    S: Sync + Send,
{
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(state) = parts.extract::<Extension<AppState>>().await.unwrap();
        let conn = state.db_pool.get_owned().await.map_err(|_| {
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
    pub session_id: SessionId
}

#[async_trait]
impl<S> FromRequestParts<S> for UserSession
where
    S: Sync + Send,
{
    type Rejection = (StatusCode, Json<RequestFailed>);
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let unauthorized = || {
            (
                StatusCode::UNAUTHORIZED,
                Json(RequestFailed {
                    msg: "Unauthorized".into()
                })
            )
        };

        let DbConnection(mut conn) = parts.extract::<DbConnection>().await.unwrap();
        let Extension(state) = parts.extract::<Extension<AppState>>().await.unwrap();

        let cookies = parts
            .headers
            .get(header::COOKIE)
            .and_then(|header| header.to_str().ok())
            .ok_or_else(unauthorized)?;
        let session_id = uchat_cookie::get_from_str(cookies, SESSION_ID)
            .and_then(|id| SessionId::from_str(id).ok())
            .ok_or_else(unauthorized)?;

        let session_signature = uchat_cookie::get_from_str(cookies, SESSION_SIGNATURE)
            .and_then(|signature| uchat_crypto::decode_base64(signature).ok())
            .and_then(|signature| uchat_crypto::sign::signature_from_bytes(signature).ok())
            .ok_or_else(unauthorized)?;

        state
            .signing_keys
            .verify(session_id.as_uuid().as_bytes(), session_signature)
            .map_err(|_| unauthorized())?;

        let session = uchat_query::session::get(&mut conn, session_id)
            .ok()
            .flatten()
            .ok_or_else(unauthorized)?;

        tracing::info!(
            user_id = session.user_id.into_inner().to_string(),
            "User logged in."
        );
        Ok(Self {
            user_id: session.user_id,
            session_id: session.id
        })
    }
}
