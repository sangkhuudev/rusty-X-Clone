use chrono::{Duration, Utc};
use axum::{async_trait, http::StatusCode, Json};
use tracing::info;
use uchat_crypto::{encode_base64, hash_password, password::deserialize_hash, verify_password};
use uchat_endpoint::{post::endpoint::{NewPost, NewPostOk}, user::endpoint::{CreateUser, CreateUserOk, Login, LoginOk}};
use uchat_query::{
    post::Post, session::{self, Session}, user::get_hashed_password, AsyncConnection, UserId
};

use crate::{error::ApiResult, extractor::{DbConnection, UserSession}, AppState};

use super::AuthorizedApiRequest;

#[async_trait]
impl AuthorizedApiRequest for NewPost {
    type Response = (StatusCode, Json<NewPostOk>);

    #[tracing::instrument(
        name = "Creating a new post",
        skip_all,
        // fields(username = %self.username)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        state: AppState,
    ) -> ApiResult<Self::Response> {
        let post = Post::new(session.user_id, self.content, self.options)?;
        let post_id = uchat_query::post::new(&mut conn, post)?;

        Ok((StatusCode::OK, Json(NewPostOk { post_id })))
    }
}