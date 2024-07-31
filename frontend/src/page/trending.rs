#![allow(non_snake_case)]

use crate::page::post::PublicPostEntry;
use chrono::Duration;
use dioxus::prelude::*;
use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

use crate::{fetch_json, ApiClient, POSTMANAGER, TOASTER};

pub fn Trending() -> Element {
    let api_client = ApiClient::global();

    let _fetch_trending_posts = use_resource(move || async move {
        TOASTER
            .write()
            .info("Retrieving trending posts", Duration::seconds(3));
        let trending_posts = fetch_json!(<TrendingPostOk>, api_client, TrendingPost);
        match trending_posts {
            Ok(res) => POSTMANAGER.write().populate(res.posts.into_iter()),
            Err(e) => TOASTER.write().error(
                format!("Failed to retrieve posts : {e}"),
                Duration::seconds(3),
            ),
        }
    });
    let post_manager = POSTMANAGER.read();
    let TrendingPosts = post_manager.posts.iter().map(|(&id, _)| {
        rsx!(
            div {
                PublicPostEntry {post_id : id}
            }
        )
    });

    rsx!({ TrendingPosts })
}
