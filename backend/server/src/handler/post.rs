use anyhow::anyhow;
use axum::{async_trait, http::StatusCode, Json};
use chrono::Utc;
use uchat_domain::Username;
use uchat_endpoint::{post::{endpoint::{Bookmark, BookmarkOk, NewPost, NewPostOk, React, ReactOk, TrendingPost, TrendingPostOk}, types::{BookmarkAction, LikeStatus, PublicPost}}, RequestFailed}; 
use uchat_query::{post::{Post, Reaction}, AsyncConnection};

use crate::{error::{ApiError, ApiResult}, extractor::{DbConnection, UserSession}, AppState};

use super::AuthorizedApiRequest;

pub fn to_public(
    conn: &mut AsyncConnection,
    post: Post,
    session: Option<&UserSession>,
) -> ApiResult<PublicPost> {
    if let Ok(content) = serde_json::from_value(post.content.0) {
        let aggregate_reactions = uchat_query::post::aggregate_reactions(conn, post.id)?;

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
            // We check the existenc of session here, if not exist, set to false
            bookmarked: {
                match session {
                    Some(session) => uchat_query::post::get_bookmark(conn, session.user_id, post.id)?,
                    None => false
                }
            },
            boosted: false,
            likes: aggregate_reactions.likes,
            dislikes: aggregate_reactions.dislikes,
            boosts: aggregate_reactions.boosts,
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

#[async_trait]
impl AuthorizedApiRequest for Bookmark {
    type Response = (StatusCode, Json<BookmarkOk>);

    #[tracing::instrument(
        name = "Add or remove a bookmark",
        skip_all,
        fields(
            post_id = ?self.post_id,
            action = ?self.action
        )
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        match self.action {
            BookmarkAction::Add => {
                uchat_query::post::bookmark(&mut conn, session.user_id, self.post_id)?;
            }

            BookmarkAction::Remove => {
                uchat_query::post::delete_bookmark(&mut conn, session.user_id, self.post_id)?;
            }
        }

        tracing::info!("Toggle bookmark successfully");
        Ok((
            StatusCode::OK,
            Json(BookmarkOk {
                status: self.action
            })
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for React {
    type Response = (StatusCode, Json<ReactOk>);

    #[tracing::instrument(
        name = "Update Like status",
        skip_all,
        fields(
            post_id = ?self.post_id,
            like_status = ?self.like_status
        )
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let reaction = Reaction {
            post_id: self.post_id,
            user_id: session.user_id,
            reaction: None,
            like_status: match self.like_status {
                LikeStatus::Like => 1,
                LikeStatus::Dislike => -1,
                LikeStatus::NoReaction => 0
            },
            created_at: Utc::now()
        };

        tracing::info!("Querying data from reactions");
        uchat_query::post::react(&mut conn, reaction)?;

        tracing::info!("Like status has been updated");
        let aggregate_reactions = uchat_query::post::aggregate_reactions(&mut conn, self.post_id)?;
        
        Ok((
            StatusCode::OK,
            Json(ReactOk {
                like_status: self.like_status,
                likes: aggregate_reactions.likes,
                dislikes: aggregate_reactions.dislikes
            })
        ))

    }    
}

