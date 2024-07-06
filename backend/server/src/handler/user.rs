use axum::{async_trait, http::StatusCode, Json};
use tracing::info;
use uchat_crypto::hash_password;
use uchat_endpoint::user::endpoint::{CreateUser, CreateUserOk};

use crate::{error::ApiResult, extractor::DbConnection, AppState};

use super::PublicApiRequest;

#[async_trait]
impl PublicApiRequest for CreateUser {
    type Response = (StatusCode, Json<CreateUserOk>);

    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response> {
        let hashed_password = hash_password(self.password)?;
        let user_id = uchat_query::user::new(&mut conn, hashed_password, &self.username)?;

        info!(username = self.username.as_ref() , "New user created successfully.");

        Ok((
            StatusCode::CREATED,
            Json(CreateUserOk {
                user_id,
                username: self.username
            })
        ))
    }
}