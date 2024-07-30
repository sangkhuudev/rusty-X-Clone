use post::endpoint::{Bookmark, NewPost, TrendingPost};
use serde::{Deserialize, Serialize};
use user::endpoint::{CreateUser, Login};

pub mod user;
pub mod post;

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
    pub msg: String
}

// public routes
route!("/account/create" => CreateUser);
route!("/account/login" => Login);

// authorized routes
route!("/post/new" => NewPost);
route!("/post/bookmark" => Bookmark);
route!("/posts/trending" => TrendingPost);