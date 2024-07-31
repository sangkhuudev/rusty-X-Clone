use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uchat_domain::{Password, SessionId, UserId, Username};
use url::Url;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: Username,
    pub password: Password,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUserOk {
    pub user_id: UserId,
    pub username: Username,
    pub session_signature: String,
    pub session_id: SessionId,
    pub session_expires: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Login {
    pub username: Username,
    pub password: Password,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginOk {
    pub session_signature: String,
    pub session_id: SessionId,
    pub session_expires: DateTime<Utc>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub profile_image: Option<Url>,
    pub user_id: UserId,
}
