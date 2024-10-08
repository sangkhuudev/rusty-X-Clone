use serde::{Deserialize, Serialize};
use uchat_domain::{PollChoiceId, PostId};

use super::types::{
    BookmarkAction, BoostAction, Content, LikeStatus, NewPostOptions, PublicPost, VoteCast,
};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    pub post_id: PostId,
    pub action: BookmarkAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookmarkOk {
    pub status: BookmarkAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct React {
    pub post_id: PostId,
    pub like_status: LikeStatus,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReactOk {
    pub like_status: LikeStatus,
    pub likes: i64,
    pub dislikes: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Boost {
    pub post_id: PostId,
    pub action: BoostAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoostOk {
    pub status: BoostAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub post_id: PostId,
    pub choice_id: PollChoiceId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoteOk {
    pub cast: VoteCast,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HomePost;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HomePostOk {
    pub posts: Vec<PublicPost>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LikedPost;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LikedPostOk {
    pub posts: Vec<PublicPost>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookmarkedPost;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookmarkedPostOk {
    pub posts: Vec<PublicPost>,
}
