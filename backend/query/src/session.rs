use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uchat_domain::{SessionId, UserId};

use crate::schema::web;
use crate::DieselError;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, 
    Serialize, Deserialize, DieselNewType
)]
pub struct FingerPrint(Value);

impl From<Value> for FingerPrint {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Insertable)]
#[diesel(table_name = web)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub fingerprint: FingerPrint,
}

pub fn new(
    conn: &mut PgConnection,
    user_id: UserId,
    duration: Duration,
    fingerprint: FingerPrint,
) -> Result<Session, DieselError> {
    let new_session = Session {
        id: SessionId::new(),
        user_id,
        expires_at: Utc::now() + duration,
        created_at: Utc::now(),
        fingerprint,
    };

    diesel::insert_into(web::table)
        .values(&new_session)
        .on_conflict((web::user_id, web::fingerprint))
        .do_update()
        .set(web::expires_at.eq(new_session.expires_at))
        .get_result::<Session>(conn)
}

pub fn get(
    conn: &mut PgConnection,
    session_id: SessionId
) -> Result<Option<Session>, DieselError> {
    tracing::debug!("Retrieving session with ID: {:?}", session_id);
    
    let session = web::table
        .filter(web::id.eq(session_id))
        .get_result::<Session>(conn)
        .optional()?;

    if let Some(ref s) = session {
        if s.expires_at < Utc::now() {
            tracing::debug!("Session has expired.");
            return Ok(None);
        } else {
            tracing::debug!("Session is valid. Expires at: {:?}", s.expires_at);
        }
    } else {
        tracing::debug!("Session not found.");
    }

    Ok(session)
}

pub fn find(
    conn: &mut PgConnection,
    user_id: UserId,
    fingerprint: FingerPrint
) -> Result<Session, DieselError> {
    web::table
        .filter(web::user_id.eq(user_id))
        .filter(web::fingerprint.eq(fingerprint))
        .get_result(conn)
}

