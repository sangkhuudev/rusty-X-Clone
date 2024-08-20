use crate::post::DeleteStatus;
use crate::schema::users::{self, columns};
use crate::DieselError;
use crate::QueryError;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use password_hash::PasswordHashString;
use uchat_domain::UserId;
use uchat_domain::Username;
use uchat_endpoint::Update;

pub async fn new<T: AsRef<str>>(
    conn: &mut AsyncPgConnection,
    hash: PasswordHashString,
    handle: T,
) -> Result<UserId, QueryError> {
    // Create a new unique user ID
    let user_id = UserId::new();

    // Insert new user into the users table
    let _ = diesel::insert_into(users::table)
        .values((
            columns::id.eq(user_id),                  // Set the user ID
            columns::password_hash.eq(hash.as_str()), // Set the password hash
            columns::handle.eq(handle.as_ref()),      // Set the handle
        ))
        .execute(conn)
        .await // Execute the query and propagate any errors
        .map_err(|e| {
            tracing::log::error!("Failed to insert new user: {}", e);
            QueryError::from(e)
        });

    // Return the new user ID if successful
    Ok(user_id)
}

pub async fn get_hashed_password(
    conn: &mut AsyncPgConnection,
    username: &Username,
) -> Result<String, QueryError> {
    Ok(users::table
        .filter(columns::handle.eq(username.as_ref()))
        .select(columns::password_hash)
        .get_result(conn)
        .await?)
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: UserId,
    pub email: Option<String>,
    pub email_confirmed: Option<DateTime<Utc>>,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub handle: String,
    pub created_at: DateTime<Utc>,
    pub profile_image: Option<String>,
}

pub async fn get(conn: &mut AsyncPgConnection, user_id: UserId) -> Result<User, QueryError> {
    users::table
        .filter(columns::id.eq(user_id))
        .get_result(conn)
        .await
        .map_err(QueryError::from)
}

pub async fn find(conn: &mut AsyncPgConnection, username: &Username) -> Result<User, DieselError> {
    users::table
        .filter(columns::handle.eq(username.as_ref()))
        .get_result(conn)
        .await
}

#[derive(Debug)]
pub struct UpdateProfileParams {
    pub id: UserId,
    pub display_name: Update<String>,
    pub email: Update<String>,
    pub password_hash: Update<PasswordHashString>,
    pub profile_image: Update<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
struct UpdateProfileParamsInternal {
    pub display_name: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub password_hash: Option<String>,
    pub profile_image: Option<Option<String>>,
}

pub async fn update_profile(
    conn: &mut AsyncPgConnection,
    query_params: UpdateProfileParams,
) -> Result<(), DieselError> {
    use crate::schema::users;

    let update = UpdateProfileParamsInternal {
        display_name: query_params.display_name.into_nullable(),
        email: query_params.email.into_nullable(),
        password_hash: query_params
            .password_hash
            .into_option()
            .map(|s| s.to_string()),
        profile_image: query_params.profile_image.into_nullable(),
    };

    diesel::update(users::table)
        .filter(users::id.eq(&query_params.id))
        .set(&update)
        .execute(conn)
        .await
        .map(|_| ())
}

//------------------------------------------------------------------------------
pub async fn follow(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    follow: UserId,
) -> Result<(), DieselError> {
    // Change names of user_id and post_id because we dont want to mess with database.
    let uid = user_id;
    let fid = follow;
    {
        use crate::schema::followers::dsl::*;
        diesel::insert_into(followers)
            .values((user_id.eq(uid), follows.eq(fid)))
            .on_conflict((user_id, follows))
            .do_nothing()
            .execute(conn)
            .await
            .map(|_| ())
    }
}

pub async fn unfollow(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    stop_following: UserId,
) -> Result<DeleteStatus, DieselError> {
    let uid = user_id;
    let fid = stop_following;
    {
        use crate::schema::followers::dsl::*;
        diesel::delete(followers)
            .filter(follows.eq(fid))
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

pub async fn is_following(
    conn: &mut AsyncPgConnection,
    user_id: UserId,
    is_following: UserId,
) -> Result<bool, DieselError> {
    let uid = user_id;
    let fid = is_following;
    {
        use crate::schema::followers::dsl::*;
        use diesel::dsl::count;

        followers
            .filter(user_id.eq(uid))
            .filter(follows.eq(fid))
            .select(count(user_id))
            .get_result(conn)
            .await
            .optional()
            .map(|n: Option<i64>| match n {
                Some(n) => n == 1,
                None => false,
            })
    }
}

#[cfg(test)]
pub mod tests {
    // use crate::test_db::Result;
    pub mod util {
        use crate::user as user_query;
        use crate::user::User;
        use diesel_async::AsyncPgConnection;
        pub async fn new_user(conn: &mut AsyncPgConnection, handle: &str) -> User {
            let hash = uchat_crypto::hash_password("password").unwrap();
            let id = user_query::new(conn, hash, handle).await.unwrap();
            user_query::get(conn, id).await.unwrap()
        }
    }
}
