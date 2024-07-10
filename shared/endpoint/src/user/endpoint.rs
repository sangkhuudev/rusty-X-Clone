use serde::{Deserialize, Serialize};
use uchat_domain::{Password, UserId, Username};

use crate::Endpoint;




#[derive(Clone,Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: Username,
    pub password: Password,
}

impl Endpoint for CreateUser {
    const URL: &'static str = "/account/create";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateUserOk {
    pub user_id: UserId,
    pub username: Username
}