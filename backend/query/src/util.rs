use deadpool_diesel::Status;
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{
        deadpool::{Object, Pool},
        AsyncDieselConnectionManager,
    },
    AsyncPgConnection,
};

use crate::error::QueryError;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::error::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

#[derive(Clone)]
pub struct AsyncConnectionPool(pub Pool<AsyncPgConnection>);

impl AsyncConnectionPool {
    pub async fn new<S: AsRef<str>>(url: S) -> Result<Self, QueryError> {
        let pool = new_async_pool(url)?;
        {
            // check connection
            let _ = pool
                .0
                .get()
                .await
                .map_err(|e| QueryError::Connection(e.to_string()))?;
        }
        Ok(pool)
    }

    pub async fn get(&self) -> Result<Object<AsyncPgConnection>, QueryError> {
        self.0
            .get()
            .await
            .map_err(|e| QueryError::Connection(e.to_string()))
    }

    pub fn status(&self) -> Status {
        self.0.status()
    }
}

/// Run database migrations
pub async fn run_migrations(
    connection: &mut impl MigrationHarness<diesel::pg::Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

/// Connect to the database
pub async fn connect<S: AsRef<str>>(url: S) -> Result<AsyncPgConnection, ConnectionError> {
    use diesel_async::AsyncConnection;
    let url = url.as_ref();
    AsyncPgConnection::establish(url).await
}

/// Usage:
/// ```ignore
/// let async_pool = new_async_pool("postgres://login@localhost/sample");
/// let conn = &mut async_pool.get().await?;
/// ```
pub fn new_async_pool<S: AsRef<str>>(url: S) -> Result<AsyncConnectionPool, QueryError> {
    let url = url.as_ref();
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
    Pool::builder(manager)
        .build()
        .map(AsyncConnectionPool)
        .map_err(|e| QueryError::Pool(e.to_string()))
}
