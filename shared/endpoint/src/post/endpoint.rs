use serde::{Deserialize, Serialize};
use uchat_domain::PostId;

use super::types::{Content, NewPostOptions, PublicPost};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewPost {
    pub content: Content,
    pub options: NewPostOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewPostOk {
    pub post_id: PostId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendingPost;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrendingPostOk {
    pub posts: Vec<PublicPost>,
}

