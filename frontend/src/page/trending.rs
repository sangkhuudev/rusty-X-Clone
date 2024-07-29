#![allow(non_snake_case)]

use chrono::Duration;
use dioxus::prelude::*;
use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

use crate::{fetch_json, ApiClient, TOASTER};

pub fn Trending() -> Element {
    let api_client = ApiClient::global();

    let _fetch_trending_posts = use_resource(move || async move {
        TOASTER.write().info("Retrieving trending posts", Duration::seconds(3));
        let trending_posts = fetch_json!(<TrendingPostOk>, api_client, TrendingPost);
        match trending_posts {
            Ok(res) => {},
            Err(e) => {
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {e}"),
                    Duration::seconds(3)
                )
            }
        }
    });

    rsx!(
        h1 {
            "Trending"
        }
    )
}