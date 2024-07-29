use anyhow::anyhow;
use axum::{async_trait, http::StatusCode, Json};
use uchat_domain::Username;
use uchat_endpoint::{post::{endpoint::{NewPost, NewPostOk, TrendingPost, TrendingPostOk}, types::{LikeStatus, PublicPost}}, RequestFailed}; 
use uchat_query::{post::Post, AsyncConnection};

use crate::{error::{ApiError, ApiResult}, extractor::{DbConnection, UserSession}, AppState};

use super::AuthorizedApiRequest;

pub fn to_public(
    conn: &mut AsyncConnection,
    post: Post,
    session: Option<&UserSession>,
) -> ApiResult<PublicPost> {
    if let Ok(mut content) = serde_json::from_value(post.content.0) {
        Ok(PublicPost {
            id: post.id,
            by_user: {
                let profile = uchat_query::user::get(conn, post.user_id)?;
                super::user::to_public(profile)?
            },
            content,
            time_posted: post.time_posted,
            reply_to: {
                match post.reply_to {
                    Some(other_post_id) => {
                        let original_post = uchat_query::post::get(conn, other_post_id)?;
                        let original_user = uchat_query::user::get(conn, original_post.user_id)?;
                        Some((
                            Username::try_new(original_user.handle).unwrap(),
                            original_user.id,
                            other_post_id
                        ))
                    },
                    None => None
                }
            },
            like_status: LikeStatus::NoReaction,
            bookmarked: false,
            boosted: false,
            likes: 0,
            dislikes: 0,
            boosts: 0,
        })
    } else {
        Err(ApiError {
            code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            error: anyhow!(RequestFailed {
                msg: "Invalid post data".to_string()
            })
        })
    }
}

#[async_trait]
impl AuthorizedApiRequest for NewPost {
    type Response = (StatusCode, Json<NewPostOk>);

    #[tracing::instrument(
        name = "Creating a new post",
        skip_all,
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let post = Post::new(session.user_id, self.content, self.options)?;
        let post_id = uchat_query::post::new(&mut conn, post)?;
        tracing::info!(post_id = ?post_id, "New post created successfully");

        Ok((StatusCode::OK, Json(NewPostOk { post_id })))
    }
}

#[async_trait]
impl AuthorizedApiRequest for TrendingPost {
    type Response = (StatusCode, Json<TrendingPostOk>);

    #[tracing::instrument(
        name = "Getting trending posts",
        skip_all,
        // fields(post_id = %self.id)
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut posts = vec![];
        for post in uchat_query::post::get_trending(&mut conn)? {
            let post_id = post.id;
            match to_public(&mut conn, post, Some(&session)) {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(error = %e.error, post_id = ?post_id, "Post contains invalid data");
                }
            }
        }
        Ok((StatusCode::OK, Json(TrendingPostOk { posts})))
    }
}