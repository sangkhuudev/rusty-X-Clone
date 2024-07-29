use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uchat_domain::{PostId, UserId};
use uuid::Uuid;
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