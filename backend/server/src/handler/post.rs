use crate::handler::save_image;
use anyhow::anyhow;
use axum::{async_trait, http::StatusCode, Json};
use chrono::Utc;
use diesel_async::AsyncPgConnection;
use uchat_domain::{ImageId, Username};
use uchat_endpoint::{
    app_url::construct_image_url,
    post::{endpoint::*, types::*},
    RequestFailed,
};
use uchat_query::post::{did_vote, Post, Reaction};

use crate::{
    error::{ApiError, ApiResult},
    extractor::{DbConnection, UserSession},
    AppState,
};

use super::AuthorizedApiRequest;

#[tracing::instrument(
    name = "Make the post public",
    skip(conn, session),
    fields(
        user_id = ?post.user_id,
        content = ?post.content
    )
)]
pub async fn to_public(
    conn: &mut AsyncPgConnection,
    post: Post,
    session: Option<&UserSession>,
) -> ApiResult<PublicPost> {
    if let Ok(mut content) = serde_json::from_value(post.content.0) {
        match content {
            Content::Image(ref mut img) => {
                if let ImageKind::Id(id) = img.kind {
                    tracing::debug!("Change the kind of image from ImageKind::Id)");

                    let url = construct_image_url(&id.to_string()).await.unwrap();
                    tracing::info!("Image url: {}", url);
                    tracing::debug!("Kind of image: ImageKind::Url(url)");
                    img.kind = ImageKind::Url(url);
                }
            }
            Content::Poll(ref mut poll) => {
                for (id, result) in uchat_query::post::get_poll_results(conn, post.id)
                    .await?
                    .results
                {
                    for choice in poll.choices.iter_mut() {
                        if choice.id == id {
                            choice.num_votes = result;
                            break;
                        }
                    }
                }
                if let Some(session) = session {
                    poll.voted = did_vote(conn, session.user_id, post.id).await?;
                }
            }
            _ => {}
        }
        let aggregate_reactions = uchat_query::post::aggregate_reactions(conn, post.id).await?;

        Ok(PublicPost {
            id: post.id,
            by_user: {
                let profile = uchat_query::user::get(conn, post.user_id).await?;
                super::user::to_public(profile).await?
            },
            content,
            time_posted: post.time_posted,
            reply_to: {
                match post.reply_to {
                    Some(other_post_id) => {
                        let original_post = uchat_query::post::get(conn, other_post_id).await?;
                        let original_user =
                            uchat_query::user::get(conn, original_post.user_id).await?;
                        Some((
                            Username::try_new(original_user.handle).unwrap(),
                            original_user.id,
                            other_post_id,
                        ))
                    }
                    None => None,
                }
            },
            // Display current like status
            like_status: {
                match session {
                    Some(session) => {
                        match uchat_query::post::get_reaction(conn, post.id, session.user_id)
                            .await?
                        {
                            Some(reaction) if reaction.like_status == 1 => LikeStatus::Like,
                            Some(reaction) if reaction.like_status == -1 => LikeStatus::Dislike,
                            _ => LikeStatus::NoReaction,
                        }
                    }
                    None => LikeStatus::NoReaction,
                }
            },
            // We check the existenc of session here, if not exist, set to false
            bookmarked: {
                match session {
                    Some(session) => {
                        uchat_query::post::get_bookmark(conn, session.user_id, post.id).await?
                    }
                    None => false,
                }
            },
            boosted: {
                match session {
                    Some(session) => {
                        uchat_query::post::get_boost(conn, session.user_id, post.id).await?
                    }
                    None => false,
                }
            },
            likes: aggregate_reactions.likes,
            dislikes: aggregate_reactions.dislikes,
            boosts: aggregate_reactions.boosts,
        })
    } else {
        Err(ApiError {
            code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            error: anyhow!(RequestFailed {
                msg: "Invalid post data".to_string()
            }),
        })
    }
}

#[async_trait]
impl AuthorizedApiRequest for NewPost {
    type Response = (StatusCode, Json<NewPostOk>);

    #[tracing::instrument(name = "Creating a new post", skip_all)]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut content = self.content;
        if let Content::Image(ref mut img) = content {
            if let ImageKind::DataUrl(data) = &img.kind {
                let id = ImageId::new();
                save_image(id, data).await?;
                img.kind = ImageKind::Id(id);
            }
        }
        let post = Post::new(session.user_id, content, self.options)?;
        let post_id = uchat_query::post::new(&mut conn, post).await?;
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
        for post in uchat_query::post::get_trending(&mut conn).await? {
            let post_id = post.id;
            match to_public(&mut conn, post, Some(&session)).await {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(error = %e.error, post_id = ?post_id, "Post contains invalid data");
                }
            }
        }
        Ok((StatusCode::OK, Json(TrendingPostOk { posts })))
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
                uchat_query::post::bookmark(&mut conn, session.user_id, self.post_id).await?;
            }

            BookmarkAction::Remove => {
                uchat_query::post::delete_bookmark(&mut conn, session.user_id, self.post_id)
                    .await?;
            }
        }

        tracing::info!("Toggle bookmark successfully");
        Ok((
            StatusCode::OK,
            Json(BookmarkOk {
                status: self.action,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for Boost {
    type Response = (StatusCode, Json<BoostOk>);

    #[tracing::instrument(
        name = "Add or remove a boost",
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
            BoostAction::Add => {
                uchat_query::post::boost(&mut conn, session.user_id, self.post_id, Utc::now())
                    .await?;
            }

            BoostAction::Remove => {
                uchat_query::post::delete_boost(&mut conn, session.user_id, self.post_id).await?;
            }
        }

        tracing::info!("Boost a post successfully");
        Ok((
            StatusCode::OK,
            Json(BoostOk {
                status: self.action,
            }),
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
                LikeStatus::NoReaction => 0,
            },
            created_at: Utc::now(),
        };

        tracing::info!("Querying data from reactions");
        uchat_query::post::react(&mut conn, reaction).await?;

        tracing::info!("Like status has been updated");
        let aggregate_reactions =
            uchat_query::post::aggregate_reactions(&mut conn, self.post_id).await?;

        Ok((
            StatusCode::OK,
            Json(ReactOk {
                like_status: self.like_status,
                likes: aggregate_reactions.likes,
                dislikes: aggregate_reactions.dislikes,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for Vote {
    type Response = (StatusCode, Json<VoteOk>);

    #[tracing::instrument(
        name = "Cast a new vote",
        skip_all,
        fields(
            post_id = ?self.post_id,
            // action = ?self.action
        )
    )]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let cast =
            uchat_query::post::vote(&mut conn, session.user_id, self.post_id, self.choice_id)
                .await?;

        tracing::info!("Cast a vote successfully");
        Ok((StatusCode::OK, Json(VoteOk { cast })))
    }
}

#[async_trait]
impl AuthorizedApiRequest for HomePost {
    type Response = (StatusCode, Json<HomePostOk>);

    #[tracing::instrument(name = "Getting home posts", skip_all)]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut posts = vec![];
        for post in uchat_query::post::get_home_posts(&mut conn, session.user_id).await? {
            let post_id = post.id;
            match to_public(&mut conn, post, Some(&session)).await {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(error = %e.error, post_id = ?post_id, "Post contains invalid data");
                }
            }
        }
        Ok((StatusCode::OK, Json(HomePostOk { posts })))
    }
}

#[async_trait]
impl AuthorizedApiRequest for LikedPost {
    type Response = (StatusCode, Json<LikedPostOk>);

    #[tracing::instrument(name = "Getting liked posts", skip_all)]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut posts = vec![];
        for post in uchat_query::post::get_liked_posts(&mut conn, session.user_id).await? {
            let post_id = post.id;
            match to_public(&mut conn, post, Some(&session)).await {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(error = %e.error, post_id = ?post_id, "Post contains invalid data");
                }
            }
        }
        Ok((StatusCode::OK, Json(LikedPostOk { posts })))
    }
}

#[async_trait]
impl AuthorizedApiRequest for BookmarkedPost {
    type Response = (StatusCode, Json<BookmarkedPostOk>);

    #[tracing::instrument(name = "Getting bookmarked posts", skip_all)]
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut posts = vec![];
        for post in uchat_query::post::get_bookmarked_posts(&mut conn, session.user_id).await? {
            let post_id = post.id;
            match to_public(&mut conn, post, Some(&session)).await {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(error = %e.error, post_id = ?post_id, "Post contains invalid data");
                }
            }
        }
        Ok((StatusCode::OK, Json(BookmarkedPostOk { posts })))
    }
}
