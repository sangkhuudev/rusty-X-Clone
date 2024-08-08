use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uchat_domain::{Password, SessionId, UserId, Username};
use url::Url;

use crate::Update;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMyProfile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMyProfileOk {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub profile_image: Option<Url>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProfile {
    pub display_name: Update<String>,
    pub email: Update<String>,
    pub profile_image: Update<String>,
    pub password: Update<Password>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProfileOk {
    pub profile_image: Option<Url>,
}
