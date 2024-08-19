#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

pub fn Trending() -> Element {
    let api_client = ApiClient::global();

    // Fetch trending posts asynchronously using use_resource
    let _fetch_posts = use_resource(move || async move {
        tracing::info!("Starting request to fetch trending posts.");
        POSTMANAGER.write().clear();
        // Define a timeout duration and start fetching data
        match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
            Ok(res) => {
                POSTMANAGER.write().populate(res.posts.clone().into_iter());
                TOASTER
                    .write()
                    .info("Retrieving trending posts", Duration::milliseconds(1200));
            }
            Err(err) => {
                tracing::error!("Failed to fetch trending posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {err}"),
                    Duration::milliseconds(1000),
                );
            }
        }
    });
    let post_manager = POSTMANAGER.read();
    let trending_posts = post_manager.all_to_public();

    rsx!(
        Appbar {
            title: "Trending posts",
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().go_back();
                },
                img: ICON_BACK,
                label: "Back",
                title: "Go to the previous page",
            }
        }
        {trending_posts.into_iter()}
    )
}
