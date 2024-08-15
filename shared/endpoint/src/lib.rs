use load_dotenv::load_dotenv;
use post::endpoint::{
    Bookmark, BookmarkedPost, Boost, HomePost, LikedPost, NewPost, React, TrendingPost, Vote,
};
use serde::{Deserialize, Serialize};
use user::endpoint::{CreateUser, GetMyProfile, Login, UpdateProfile, ViewProfile};

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
    // use anyhow::Context;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::sync::Mutex;
    use url::Url;

    // Compile-time API URL
    pub const API_URL: &str = std::env!("API_URL");

    // Thread-safe cache for constructed image URLs
    static IMAGE_URL_CACHE: Lazy<Mutex<HashMap<String, Url>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    // Base URL precomputed with Lazy
    pub static BASE_IMAGE_URL: Lazy<Url> = Lazy::new(|| {
        Url::from_str(&std::env!("API_URL"))
            .expect("Invalid API_URL format")
            .join("usercontent/image/")
            .expect("Failed to join IMAGE path")
    });

    /// Constructs the full image URL for a given image ID.
    ///
    /// If the URL has already been constructed, it is retrieved from the cache.
    /// Otherwise, the URL is constructed, stored in the cache, and returned.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice representing the image ID.
    ///
    /// # Returns
    ///
    /// A `Result` containing the constructed URL or an error if the URL cannot be constructed.
    // pub fn construct_image_url(id: &str) -> Result<Url, url::ParseError> {
    //     BASE_IMAGE_URL.join(id)
    // }

    pub fn construct_image_url(id: &str) -> Result<Url, url::ParseError> {
        let mut cache = IMAGE_URL_CACHE.lock().unwrap();

        if let Some(cached_url) = cache.get(id) {
            return Ok(cached_url.clone());
        }

        let url = BASE_IMAGE_URL.join(id)?;
        cache.insert(id.to_string(), url.clone());
        Ok(url)
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
route!("/profile/view" => ViewProfile);

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
