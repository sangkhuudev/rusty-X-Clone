#![allow(non_snake_case)]

use uchat_domain::UserId;
use url::Url;

#[derive(Default)]
pub struct LocalProfile {
    pub image: Option<Url>,
    pub user_id: Option<UserId>,
}
