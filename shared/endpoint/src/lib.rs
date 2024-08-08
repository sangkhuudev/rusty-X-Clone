use load_dotenv::load_dotenv;
use post::endpoint::{
    Bookmark, BookmarkedPost, Boost, HomePost, LikedPost, NewPost, React, TrendingPost, Vote,
};
use serde::{Deserialize, Serialize};
use user::endpoint::{CreateUser, GetMyProfile, Login, UpdateProfile};

pub mod post;
pub mod user;

pub trait Endpoint {
    const URL: &'static str;

    fn url(&self) -> &'static str {
        Self::URL
    }
}

macro_rules! route {
    ($url:literal => $request_type:ty) => {
        impl Endpoint for $request_type {
            const URL: &'static str = $url;
        }
    };
}
#[derive(thiserror::Error, Debug, Deserialize, Serialize)]
#[error("{msg}")]
pub struct RequestFailed {
    pub msg: String,
}

//--------------------------------------------------------------
// Loading .env file
load_dotenv!();

pub mod app_url {
    use std::str::FromStr;

    use url::Url;

    pub const API_URL: &str = std::env!("API_URL");

    pub fn domain_and(fragment: &str) -> Url {
        Url::from_str(API_URL)
            .and_then(|url| url.join(fragment))
            .unwrap()
    }

    pub mod user_content {
        pub const ROOT: &str = "usercontent/";
        pub const IMAGE: &str = "image/";
    }
}

//---------------------------------------------------------------
// public routes
route!("/account/create" => CreateUser);
route!("/account/login" => Login);
// route!("/profile/view" => ViewProfile);

// authorized routes
route!("/post/new" => NewPost);
route!("/post/bookmark" => Bookmark);
route!("/post/boost" => Boost);
route!("/post/react" => React);
route!("/post/vote" => Vote);
route!("/posts/trending" => TrendingPost);
route!("/posts/home" => HomePost);
route!("/posts/liked" => LikedPost);
route!("/posts/bookmarked" => BookmarkedPost);
route!("/profile/update" => UpdateProfile);
route!("/profile/me" => GetMyProfile);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update<T> {
    Change(T),
    NoChange,
    SetNull,
}

impl<T> Update<T> {
    pub fn into_option(self) -> Option<T> {
        match self {
            Update::Change(data) => Some(data),
            Update::NoChange => None,
            Update::SetNull => None,
        }
    }

    pub fn into_nullable(self) -> Option<Option<T>> {
        match self {
            Update::Change(data) => Some(Some(data)),
            Update::NoChange => None,
            Update::SetNull => Some(None),
        }
    }
}
