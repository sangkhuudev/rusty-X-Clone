#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use uchat_endpoint::post::endpoint::{HomePost, HomePostOk};

pub mod bookmarked;
pub mod liked;

pub fn Home() -> Element {
    let api_client = ApiClient::global();

    // Initialize tracing
    tracing::info!("Initializing Home component.");

    // Fetch trending posts asynchronously
    let _fetch_posts = use_resource(move || async move {
        tracing::info!("Starting request to fetch trending posts.");
        POSTMANAGER.write().clear();
        // Define a timeout duration and start fetching data
        match fetch_json!(<HomePostOk>, api_client, HomePost) {
            Ok(res) => {
                tracing::info!("Successfully retrieved home posts.");
                POSTMANAGER.write().populate(res.posts.into_iter());
                TOASTER
                    .write()
                    .info("Retrieving home posts", Duration::milliseconds(1200));
            }
            Err(err) => {
                tracing::error!("Failed to fetch home posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {err}"),
                    Duration::milliseconds(1200),
                );
            }
        }
    });

    let post_manager = POSTMANAGER.read();
    let Posts = {
        let posts = post_manager.all_to_public();
        if posts.is_empty() {
            let TrendingLink = rsx!(
                Link {
                    to: Route::Trending {},
                    class: "link",
                    "trending"
                }
            );
            rsx!(
                div {
                    // Tailwind doesn't support spaces so we have to use underscore
                    class: "flex flex-col text-center justify-center
                    h-[calc(100vh_-_var(--navbar-height)_-_var(--appbar-height))]",
                    span {
                        "Checkout what's:   " {TrendingLink}
                        "   and follow some users to get started."
                    }
                }
            )
        } else {
            rsx!({ posts.into_iter() })
        }
    };

    rsx!(
        Appbar {
            title: "Home",
            AppbarImgButton {
                click_handler: move |_| {
                    router().push(Route::HomeLiked {});
                },
                img: ICON_LIKE,
                label: "Liked",
                title: "Show liked posts",
            },
            AppbarImgButton {
                click_handler: move |_| {
                    router().push(Route::HomeBookmarked {});
                },
                img: ICON_BOOKMARK,
                label: "Saved",
                title: "Show bookmarked posts",
            },
            AppbarImgButton {
                click_handler: move |_| {},
                img: ICON_HOME,
                label: "Home",
                title: "Go to Home page",
                disabled: true,
                append_class: appbar::BUTTON_SELECTED,
            },
        }
        {Posts.into_iter()}
    )
}
