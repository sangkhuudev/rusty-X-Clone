#![allow(non_snake_case)]

use dioxus::prelude::*;
use chrono::Duration;
use crate::page::post::content::Content;
use uchat_domain::PostId;
use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

use crate::{fetch_json, ApiClient, POSTMANAGER, TOASTER};

pub fn Trending() -> Element {
    let api_client = ApiClient::global();

    let _fetch_trending_posts = use_resource(move || async move {
        TOASTER.write().info("Retrieving trending posts", Duration::seconds(3));
        let trending_posts = fetch_json!(<TrendingPostOk>, api_client, TrendingPost);
        match trending_posts {
            Ok(res) => {
                POSTMANAGER.write().populate(res.posts.into_iter())
            },
            Err(e) => {
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {e}"),
                    Duration::seconds(3)
                )
            }
        }
    });
    let post_manager = POSTMANAGER.read();
    let TrendingPosts = post_manager
        .posts
        .iter()
        .map(|(&id,_)| {
            rsx!(
                div {
                    PublicPostEntry {post_id : id}
                }
            )
        });
        // .collect::<Vec<LazyNodes>>();

    rsx!(
        {TrendingPosts}
    )
}

#[component]
pub fn PublicPostEntry(post_id: PostId) -> Element {
    let this_post = POSTMANAGER.read().get(&post_id).unwrap().clone();

    rsx!(
        div {
            key: "{this_post.id.to_string()}",
            class: "grid grid-cols-[50px_1fr] gap-2 mb-4",
            div { /*profile image */},
            div {
                class: "flex flex-col gap-3",
                // header
                // reply to
                // content
                Content{ post: this_post},
                // action bar
                hr {}
            }
        }
    )
}