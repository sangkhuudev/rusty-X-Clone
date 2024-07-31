#[macro_use]
extern crate diesel_derive_newtype;

#[cfg(test)]
pub mod test_db;

pub use diesel::result::Error as DieselError;

pub mod error;
pub mod schema;
pub use error::QueryError;

pub mod user;
pub use uchat_domain::id::*;

pub mod util;
pub use util::{AsyncConnection, AsyncConnectionPool, OwnedAsyncConnection};

pub mod post;
pub mod session;
