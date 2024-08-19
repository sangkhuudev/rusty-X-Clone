#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use uchat_endpoint::post::endpoint::{BookmarkedPost, BookmarkedPostOk};

#[component]
pub fn HomeBookmarked() -> Element {
    let api_client = ApiClient::global();

    // Initialize tracing
    tracing::info!("Initializing Bookmarked component.");

    // Fetch trending posts asynchronously
    let _fetch_posts = use_resource(move || async move {
        POSTMANAGER.write().clear();
        // Define a timeout duration and start fetching data
        match fetch_json!(<BookmarkedPostOk>, api_client, BookmarkedPost) {
            Ok(data) => {
                tracing::info!("Successfully retrieved bookmarked posts.");
                POSTMANAGER.write().populate(data.posts.into_iter());
                TOASTER
                    .write()
                    .info("Retrieving bookmarked posts", Duration::milliseconds(1500));
            }
            Err(err) => {
                tracing::error!("Failed to fetch bookmarked posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve bookmarked posts : {err}"),
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
                        "You haven't bookmarked any posts yet. Checkout what's:   " {TrendingLink}
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
            title: "Bookmarked",
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().push(Route::HomeLiked {});
                },
                img: ICON_LIKE,
                label: "Liked",
                title: "Show liked posts",
            },
            AppbarImgButton {
                click_handler: move |_| {},
                img: ICON_BOOKMARK,
                label: "Saved",
                title: "Show bookmarked posts",
                disabled: true,
                append_class: appbar::BUTTON_SELECTED,
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
