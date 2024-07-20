use serde::{Deserialize, Serialize};
use uchat_domain::PostId;

use crate::Endpoint;

use super::types::{Content, NewPostOptions};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewPost {
    pub content: Content,
    pub options: NewPostOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewPostOk {
    pub post_id: PostId,
}


impl Endpoint for NewPost {
    const URL: &'static str = "/post/new";
}