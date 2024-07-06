use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Extension, RequestPartsExt,
};
use uchat_query::OwnedAsyncConnection;

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
