use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uchat_domain::{Caption, Headline, ImageId, Message, PostId, UserId, Username};
use url::Url;

use crate::user::types::PublicUserProfile;

//-------------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Chat {
    pub headline: Option<Headline>,
    pub message: Message,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImageKind {
    DataUrl(String),
    Id(ImageId),
    Url(Url)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Image {
    pub kind: ImageKind,
    pub caption: Option<Caption>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Content {
    Chat(Chat),
    Image(Image),
}

impl From<Chat> for Content {
    fn from(value: Chat) -> Self {
        Content::Chat(value)
    }
}

impl From<Image> for Content {
    fn from(value: Image) -> Self {
        Content::Image(value)
    }
}
//-------------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewPostOptions {
    pub reply_to: Option<PostId>,
    pub direct_message_to: Option<UserId>,
    pub time_posted: DateTime<Utc>,
}

impl Default for NewPostOptions {
    fn default() -> Self {
        Self {
            reply_to: None,
            direct_message_to: None,
            time_posted: Utc::now(),
        }
    }
}

//-------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LikeStatus {
    Like,
    Dislike,
    NoReaction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicPost {
    // Section 1: Allow us to see the post
    pub id: PostId,
    pub by_user: PublicUserProfile,
    pub content: Content,
    pub time_posted: DateTime<Utc>,
    pub reply_to: Option<(Username, UserId, PostId)>,
    // Section 2: Allow us to interact with post
    pub like_status: LikeStatus,
    pub bookmarked: bool,
    pub boosted: bool,
    pub likes: i64,
    pub dislikes: i64,
    pub boosts: i64,
}

//-------------------------------------------------------------------------------------
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookmarkAction {
    Add,
    Remove,
}

impl From<BookmarkAction> for bool {
    fn from(value: BookmarkAction) -> Self {
        match value {
            BookmarkAction::Add => true,
            BookmarkAction::Remove => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoostAction {
    Add,
    Remove,
}

impl From<BoostAction> for bool {
    fn from(value: BoostAction) -> Self {
        match value {
            BoostAction::Add => true,
            BoostAction::Remove => false,
        }
    }
}
