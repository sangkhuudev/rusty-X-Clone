use crate::schema::*;
use crate::DieselError;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uchat_domain::PollChoiceId;
use uchat_domain::{PostId, UserId};
use uchat_endpoint::post::types::VoteCast;
use uchat_endpoint::post::types::{self, Content as EndpointContent};
use uuid::Uuid;

//------------------------------------------------------------------------------
#[derive(Clone, Debug, DieselNewType, Deserialize, Serialize)]
pub struct Content(pub serde_json::Value);

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: PostId,
    pub user_id: UserId,
    pub content: Content,
    pub time_posted: DateTime<Utc>,
    pub direct_message_to: Option<UserId>,
    pub reply_to: Option<PostId>,
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub fn new(
        posted_by: UserId,
        content: types::Content,
        options: types::NewPostOptions,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            id: Uuid::new_v4().into(),
            user_id: posted_by,
            content: Content(serde_json::to_value(content)?),
            time_posted: options.time_posted,
            direct_message_to: options.direct_message_to,
            reply_to: options.reply_to,
            created_at: Utc::now(),
        })
    }
}

//------------------------------------------------------------------------------

pub async fn new(conn: &mut AsyncPgConnection, post: Post) -> Result<PostId, DieselError> {
    use diesel_async::AsyncConnection;

    conn.transaction::<PostId, DieselError, _>(|conn| {
        async move {
            diesel::insert_into(posts::table)
                .values(&post)
                .execute(conn)
                .await?;

            match serde_json::from_value::<EndpointContent>(post.content.0) {
                Ok(EndpointContent::Poll(poll)) => {
                    for choice in &poll.choices {
                        use poll_choices::{self, columns};
                        diesel::insert_into(poll_choices::table)
                            .values((
                                columns::post_id.eq(post.id),
                                columns::id.eq(choice.id),
                                columns::choice.eq(choice.description.as_ref()),
                            ))
                            .execute(conn)
                            .await?;
                    }
                    Ok(post.id)
                }
                _ => Ok(post.id),
            }
        }
        .scope_boxed()
    })
    .await
}

pub async fn get(conn: &mut AsyncPgConnection, post_id: PostId) -> Result<Post, DieselError> {
    posts::table
        .filter(posts::columns::id.eq(post_id.as_uuid()))
        .get_result(conn)
        .await
}

pub async fn get_trending(conn: &mut AsyncPgConnection) -> Result<Vec<Post>, DieselError> {
    use crate::schema::posts::dsl::*;
    posts
        .filter(time_posted.lt(Utc::now()))
        .filter(direct_message_to.is_null())
        .order(time_posted.desc())
        .limit(30)
        .load(conn)
        .await
}

pub async fn get_public_posts(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
) -> Result<Vec<Post>, DieselError> {
    let uid = user_id;

    {
        use crate::schema::posts::dsl::*;
        posts
            .filter(user_id.eq(uid))
            .filter(time_posted.lt(Utc::now()))
            .filter(direct_message_to.is_null())
            .order(time_posted.desc())
            .limit(30)
            .load(conn)
            .await
    }
}

//------------------------------------------------------------------------------
pub async fn bookmark(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
) -> Result<(), DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::bookmarks::dsl::*;
        diesel::insert_into(bookmarks)
            .values((user_id.eq(uid), post_id.eq(pid)))
            .on_conflict((user_id, post_id))
            .do_nothing()
            .execute(conn)
            .await
            .map(|_| ())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeleteStatus {
    Deleted,
    NotFound,
}

pub async fn delete_bookmark(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
) -> Result<DeleteStatus, DieselError> {
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::bookmarks::dsl::*;
        diesel::delete(bookmarks)
            .filter(post_id.eq(pid))
            .filter(user_id.eq(uid))
            .execute(conn)
            .await
            .map(|rowcount| {
                if rowcount > 0 {
                    DeleteStatus::Deleted
                } else {
                    DeleteStatus::NotFound
                }
            })
    }
}

pub async fn get_bookmark(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
) -> Result<bool, DieselError> {
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::bookmarks::dsl::*;
        use diesel::dsl::count;

        bookmarks
            .filter(post_id.eq(pid))
            .filter(user_id.eq(uid))
            .select(count(post_id))
            .get_result(conn)
            .await
            .optional()
            .map(|n: Option<i64>| match n {
                Some(n) => n == 1,
                None => false,
            })
    }
}

//------------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize, DieselNewType)]
pub struct ReactionData(serde_json::Value);

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = reactions)]
pub struct Reaction {
    pub user_id: UserId,
    pub post_id: PostId,
    pub created_at: DateTime<Utc>,
    pub like_status: i16,
    pub reaction: Option<ReactionData>,
}

pub async fn react(conn: &mut AsyncPgConnection, reaction: Reaction) -> Result<(), DieselError> {
    diesel::insert_into(reactions::table)
        .values(&reaction)
        .on_conflict((reactions::user_id, reactions::post_id))
        .do_update()
        .set((
            reactions::like_status.eq(&reaction.like_status),
            reactions::reaction.eq(&reaction.reaction),
        ))
        .execute(conn)
        .await
        .map(|_| ())
}

pub async fn get_reaction(
    conn: &mut AsyncPgConnection,
    post_id: PostId,
    user_id: UserId,
) -> Result<Option<Reaction>, DieselError> {
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::reactions::dsl::*;

        reactions
            .filter(post_id.eq(pid))
            .filter(user_id.eq(uid))
            .get_result(conn)
            .await
            .optional()
    }
}

//------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
pub struct AggregatePostInfo {
    pub post_id: PostId,
    pub likes: i64,
    pub dislikes: i64,
    pub boosts: i64,
}

pub async fn aggregate_reactions(
    conn: &mut AsyncPgConnection,
    post_id: PostId,
) -> Result<AggregatePostInfo, DieselError> {
    let pid = post_id;
    let (likes, dislikes) = {
        use crate::schema::reactions::dsl::*;
        let likes = reactions
            .filter(post_id.eq(pid))
            .filter(like_status.eq(1))
            .count()
            .get_result(conn)
            .await?;

        let dislikes = reactions
            .filter(post_id.eq(pid))
            .filter(like_status.eq(-1))
            .count()
            .get_result(conn)
            .await?;

        (likes, dislikes)
    };

    let boosts = {
        use crate::schema::boosts::dsl::*;
        boosts
            .filter(post_id.eq(pid))
            .count()
            .get_result(conn)
            .await?
    };

    Ok(AggregatePostInfo {
        post_id,
        likes,
        dislikes,
        boosts,
    })
}

//------------------------------------------------------------------------------
pub async fn boost(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
    when: DateTime<Utc>,
) -> Result<(), DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::boosts::dsl::*;
        diesel::insert_into(boosts)
            .values((user_id.eq(uid), post_id.eq(pid), boosted_at.eq(when)))
            .on_conflict((user_id, post_id))
            .do_update()
            .set(boosted_at.eq(when))
            .execute(conn)
            .await
            .map(|_| ())
    }
}

pub async fn delete_boost(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
) -> Result<DeleteStatus, DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::boosts::dsl::*;
        diesel::delete(boosts)
            .filter(post_id.eq(pid))
            .filter(user_id.eq(uid))
            .execute(conn)
            .await
            .map(|rowcount| {
                if rowcount > 0 {
                    DeleteStatus::Deleted
                } else {
                    DeleteStatus::NotFound
                }
            })
    }
}

pub async fn get_boost(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
) -> Result<bool, DieselError> {
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::boosts::dsl::*;
        use diesel::dsl::count;

        boosts
            .filter(post_id.eq(pid))
            .filter(user_id.eq(uid))
            .select(count(post_id))
            .get_result(conn)
            .await
            .optional()
            .map(|n: Option<i64>| match n {
                Some(n) => n == 1,
                None => false,
            })
    }
}

//------------------------------------------------------------------------------

pub async fn vote(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
    choice_id: PollChoiceId,
) -> Result<VoteCast, DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let pid = post_id;
    let cid = choice_id;
    {
        use crate::schema::poll_votes::dsl::*;
        diesel::insert_into(poll_votes)
            .values((user_id.eq(uid), post_id.eq(pid), choice_id.eq(cid)))
            .on_conflict((user_id, post_id))
            .do_nothing()
            .execute(conn)
            .await
            .map(|n| {
                if n == 1 {
                    VoteCast::Yes
                } else {
                    VoteCast::AlreadyVoted
                }
            })
    }
}

pub async fn did_vote(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    post_id: PostId,
) -> Result<Option<PollChoiceId>, DieselError> {
    let uid = user_id;
    let pid = post_id;
    {
        use crate::schema::poll_votes::dsl::*;

        poll_votes
            .filter(post_id.eq(pid))
            .filter(user_id.eq(uid))
            .select(choice_id)
            .get_result(conn)
            .await
            .optional()
    }
}

pub struct PollResults {
    pub post_id: PostId,
    pub results: Vec<(PollChoiceId, i64)>,
}
pub async fn get_poll_results(
    conn: &mut AsyncPgConnection,
    post_id: PostId,
) -> Result<PollResults, DieselError> {
    let pid = post_id;
    {
        use crate::schema::poll_votes::dsl::*;
        use diesel::dsl::count;
        let results = poll_votes
            .filter(post_id.eq(pid))
            .group_by(choice_id)
            .select((choice_id, count(choice_id)))
            .load::<(PollChoiceId, i64)>(conn)
            .await?;

        Ok(PollResults {
            post_id: pid,
            results,
        })
    }
}

pub async fn get_home_posts(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
) -> Result<Vec<Post>, DieselError> {
    let uid = user_id;
    let on_schedule = posts::time_posted.lt(Utc::now());
    let public_only = posts::direct_message_to.is_null();
    let order = posts::time_posted.desc();
    let limit = 30;

    followers::table
        .filter(followers::user_id.eq(uid))
        .inner_join(posts::table.on(followers::follows.eq(posts::user_id)))
        .filter(on_schedule)
        .filter(public_only)
        .select(Post::as_select())
        .order(order)
        .limit(limit)
        .union(
            followers::table
                .filter(followers::user_id.eq(uid))
                .inner_join(boosts::table.on(boosts::user_id.eq(followers::follows)))
                .inner_join(posts::table.on(posts::id.eq(boosts::post_id)))
                .filter(on_schedule)
                .filter(public_only)
                .select(Post::as_select())
                .order(order)
                .limit(limit),
        )
        .load(conn)
        .await
}

pub async fn get_liked_posts(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
) -> Result<Vec<Post>, DieselError> {
    reactions::table
        .inner_join(posts::table)
        .filter(reactions::user_id.eq(user_id))
        .filter(reactions::like_status.eq(1))
        .filter(posts::direct_message_to.is_null())
        .select(Post::as_select())
        .limit(30)
        .get_results(conn)
        .await
}

pub async fn get_bookmarked_posts(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
) -> Result<Vec<Post>, DieselError> {
    bookmarks::table
        .inner_join(posts::table)
        .filter(bookmarks::user_id.eq(user_id))
        .filter(posts::direct_message_to.is_null())
        .select(Post::as_select())
        .limit(30)
        .get_results(conn)
        .await
}
