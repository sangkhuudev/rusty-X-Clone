use crate::{error::ApiResult, extractor::DbConnection, AppState};
use axum::{async_trait, extract::State, response::IntoResponse, Json};
use serde::Deserialize;

pub mod user;

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
) where
    Req: PublicApiRequest + Deserialize<'a>,
{
    let _ = payload.process_request(conn, state).await;
}
