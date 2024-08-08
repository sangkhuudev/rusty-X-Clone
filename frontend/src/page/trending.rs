#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

pub fn Trending() -> Element {
    let api_client = ApiClient::global();

    // Initialize tracing
    tracing::info!("Initializing Trending component.");

    // Fetch trending posts asynchronously
    let _fetch_posts = use_resource(move || async move {
        tracing::info!("Starting request to fetch trending posts.");

        // Define a timeout duration and start fetching data
        match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
            Ok(data) => {
                tracing::info!("Successfully retrieved trending posts.");
                POSTMANAGER.write().populate(data.posts.into_iter());

                TOASTER
                    .write()
                    .info("Retrieving trending posts", Duration::milliseconds(600));
            }
            Err(err) => {
                tracing::error!("Failed to fetch trending posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {err}"),
                    Duration::milliseconds(600),
                );
            }
        }
    });

    let post_manager = POSTMANAGER.read();
    let trending_posts = post_manager.all_to_public();

    rsx!({ trending_posts.into_iter() })
}
