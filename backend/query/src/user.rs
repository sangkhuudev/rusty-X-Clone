use diesel::prelude::*;
use diesel::PgConnection ;
use password_hash::PasswordHashString;
use uchat_domain::UserId;
use crate::QueryError;


pub fn new<T: AsRef<str>>(
    conn: &mut PgConnection,
    hash: PasswordHashString,
    handle: T,
) -> Result<UserId, QueryError> {
    use crate::schema::users::{self, columns}; // Importing necessary schema

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
