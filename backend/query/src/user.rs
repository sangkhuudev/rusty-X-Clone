use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection ;
use password_hash::PasswordHashString;
use uchat_domain::UserId;
use uchat_domain::Username;
use crate::QueryError;
use crate::schema::users::{self, columns}; 
use crate::DieselError;

pub fn new<T: AsRef<str>>(
    conn: &mut PgConnection,
    hash: PasswordHashString,
    handle: T,
) -> Result<UserId, QueryError> {
    // Create a new unique user ID
    let user_id = UserId::new();

    // Insert new user into the users table
    diesel::insert_into(users::table)
        .values((
            columns::id.eq(user_id), // Set the user ID
            columns::password_hash.eq(hash.as_str()), // Set the password hash
            columns::handle.eq(handle.as_ref()), // Set the handle
        ))
        .execute(conn) // Execute the query and propagate any errors
        .map_err(|e| {
            tracing::log::error!("Failed to insert new user: {}", e);
            QueryError::from(e)
        })?;

    // Return the new user ID if successful
    Ok(user_id)
}

pub fn get_hashed_password(
    conn: &mut PgConnection,
    username: &Username
) -> Result<String, QueryError> {
    Ok(users::table
        .filter(columns::handle.eq(username.as_ref()))
        .select(columns::password_hash)
        .get_result(conn)?
    )
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

pub fn get(
    conn: &mut PgConnection,
    user_id: UserId
) -> Result<User, DieselError> {
    users::table
        .filter(columns::id.eq(user_id))
        .get_result(conn)
}

pub fn find(
    conn: &mut PgConnection,
    username: &Username
) -> Result<User, DieselError> {
    users::table
        .filter(columns::handle.eq(username.as_ref()))
        .get_result(conn)
}