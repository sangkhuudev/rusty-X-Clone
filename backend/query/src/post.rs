use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uchat_domain::{PostId, UserId};
use uuid::Uuid;
use crate::schema::bookmarks::{self, columns};
use crate::schema::posts;
use crate::{schema, DieselError};
use uchat_endpoint::post::types;


#[derive(Clone, Debug, DieselNewType, Deserialize, Serialize)]
pub struct Content(pub serde_json::Value);


#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = schema::posts)]
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

pub fn new(conn: &mut PgConnection, post: Post) -> Result<PostId, DieselError> {
    conn.transaction::<PostId, DieselError, _>(|conn| {
        diesel::insert_into(schema::posts::table)
            .values(&post)
            .execute(conn)?;
        Ok(post.id)
    })
}

pub fn get(conn: &mut PgConnection, post_id: PostId) -> Result<Post, DieselError> {
    schema::posts::table
        .filter(posts::columns::id.eq(post_id.as_uuid()))
        .get_result(conn)
}

pub fn get_trending(conn: &mut PgConnection) -> Result<Vec<Post>, DieselError> {
    schema::posts::table
        .filter(posts::columns::time_posted.lt(Utc::now()))
        .filter(posts::columns::direct_message_to.is_null())
        .order(posts::columns::time_posted.desc())
        .limit(30)
        .get_results(conn)
}

pub fn bookmark(
    conn: &mut PgConnection, 
    user_id: UserId,
    post_id: PostId
) -> Result<(), DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let pid = post_id;
    conn.transaction::<(), DieselError, _>(|conn| {
        diesel::insert_into(schema::bookmarks::table)
            .values((
                columns::user_id.eq(uid),
                columns::post_id.eq(pid)
            ))
            .on_conflict((
                columns::user_id, 
                columns::post_id
            ))
            .do_nothing()
            .execute(conn)?;
        Ok(())
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeleteStatus {
    Deleted,
    NotFound
}

pub fn delete_bookmark(
    conn: &mut PgConnection, 
    user_id: UserId,
    post_id: PostId
) -> Result<DeleteStatus, DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let pid = post_id;
    conn.transaction::<DeleteStatus, DieselError, _>(|conn| {
        diesel::delete(schema::bookmarks::table)
            .filter(bookmarks::columns::user_id.eq(uid))
            .filter(bookmarks::columns::post_id.eq(pid))
            .execute(conn)
            .map(|rowcount| {
                if rowcount > 0 {
                    DeleteStatus::Deleted
                } else {
                    DeleteStatus::NotFound
                }
            })
    })
}