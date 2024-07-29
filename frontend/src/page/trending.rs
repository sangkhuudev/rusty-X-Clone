#![allow(non_snake_case)]

use dioxus::prelude::*;
use chrono::Duration;
use crate::page::post::content::Content;
use uchat_domain::PostId;
use uchat_endpoint::post::{endpoint::{TrendingPost, TrendingPostOk}, types::PublicPost};

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

    rsx!(
        {TrendingPosts}
    )
}

#[component]
pub fn Header(post: PublicPost) -> Element {
    let (posted_date, posted_time) = {
        let date = post.time_posted.format("%Y-%m-%d");
        let time = post.time_posted.format("%H:%m:%s");
        (date, time)
    };
    
    let display_name = match &post.by_user.dislay_name {
        Some(name) => name.as_ref(),
        None => ""
    };

    let handle = &post.by_user.handle;

    rsx!(
        div {
            class: "flex flex-row justify-between",
            div {
                class: "cursor-pointer",
                onclick: move |_| {},
                div {
                    "{display_name}"
                }
                div {
                    class: "font-light",
                    "{handle}"
                },
            }
            div {
                class: "text-right",
                "{posted_date}",
                "{posted_time}"
            }
        }
    )
}

#[component]
pub fn PublicPostEntry(post_id: PostId) -> Element {
    let post_manager = POSTMANAGER.read();
    let this_post = post_manager.get(&post_id).unwrap();
    // let this_post = POSTMANAGER.signal().read().get(&post_id).unwrap();

    rsx!(
        div {
            key: "{this_post.id.to_string()}",
            class: "grid grid-cols-[50px_1fr] gap-2 mb-4",
            div { /*profile image */},
            div {
                class: "flex flex-col gap-3",
                // header
                Header { post: this_post.clone()},
                // reply to
                // content
                Content{ post: this_post.clone()},
                // action bar
                hr {}
            }
        }
    )
}