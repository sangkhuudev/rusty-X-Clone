#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use uchat_endpoint::post::endpoint::{LikedPost, LikedPostOk};

#[component]
pub fn HomeLiked() -> Element {
    let api_client = ApiClient::global();

    // Initialize tracing
    tracing::info!("Initializing Home component.");

    // Fetch trending posts asynchronously
    let _fetch_posts = use_resource(move || async move {
        tracing::info!("Starting request to fetch liked posts.");
        POSTMANAGER.write().clear();
        // Define a timeout duration and start fetching data
        match fetch_json!(<LikedPostOk>, api_client, LikedPost) {
            Ok(data) => {
                tracing::info!("Successfully retrieved liked posts.");
                POSTMANAGER.write().populate(data.posts.into_iter());
                TOASTER
                    .write()
                    .info("Retrieving liked posts", Duration::milliseconds(1500));
            }
            Err(err) => {
                tracing::error!("Failed to fetch liked posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve liked posts : {err}"),
                    Duration::milliseconds(1500),
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
                        "You haven't liked any posts yet. Checkout what's:   " {TrendingLink}
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
            title: "Liked",
            AppbarImgButton {
                click_handler: move |_| {},
                img: ICON_LIKE,
                label: "Liked",
                title: "Show liked posts",
                disabled: true,
                append_class: appbar::BUTTON_SELECTED,
            },
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().push(Route::HomeBookmarked {});
                },
                img: ICON_BOOKMARK,
                label: "Saved",
                title: "Show bookmarked posts",
            },
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().push(Route::Home {});
                },
                img: ICON_HOME,
                label: "Home",
                title: "Go to Home page",
            },
        }
        {Posts}
    )
}
