use crate::{error::ApiResult, extractor::{DbConnection, UserSession}, AppState};
use axum::{async_trait, extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use core::fmt::Debug;
pub mod user;
pub mod post;
#[async_trait]
pub trait PublicApiRequest {
    type Response: IntoResponse;
    async fn process_request(
        self,
        conn: DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response>;
}

pub async fn with_public_handler<'a, Req>(
    conn: DbConnection,
    State(state): State<AppState>,
    Json(payload): Json<Req>,
) -> ApiResult<Req::Response>
where
    Req: PublicApiRequest + Deserialize<'a>,
{
    payload.process_request(conn, state).await
}

#[async_trait]
pub trait AuthorizedApiRequest {
    type Response: IntoResponse;
    async fn process_request(
        self,
        conn: DbConnection,
        session: UserSession,
        state: AppState,
    ) -> ApiResult<Self::Response>;
}

pub async fn with_handler<'a, Req>(
    conn: DbConnection,
    session: UserSession,
    State(state): State<AppState>,
    Json(payload): Json<Req>,
) -> ApiResult<Req::Response>
where
    Req: AuthorizedApiRequest + Deserialize<'a> + Debug,
{
    tracing::debug!("with_handler called with payload: {:?}", payload);
    payload.process_request(conn, session, state).await
}