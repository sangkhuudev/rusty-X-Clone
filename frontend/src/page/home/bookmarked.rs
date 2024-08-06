#![allow(non_snake_case)]

use chrono::Duration;
use uchat_endpoint::post::endpoint::{BookmarkedPost, BookmarkedPostOk};
use crate::prelude::*;
use dioxus::prelude::*;

#[component]
pub fn HomeBookmarked() -> Element {
    let api_client = ApiClient::global();

    // Initialize tracing
    tracing::info!("Initializing Home component.");

    // Fetch trending posts asynchronously
    let mut fetch_posts = use_resource(move || async move {
        tracing::info!("Starting request to fetch trending posts.");
        // Define a timeout duration and start fetching data
        match fetch_json!(<BookmarkedPostOk>, api_client, BookmarkedPost) {
            Ok(data) => {
                tracing::info!("Successfully retrieved home posts.");
                POSTMANAGER.write().populate(data.posts.into_iter());
                TOASTER
                    .write()
                    .info("Retrieving home posts", Duration::milliseconds(600));

            }
            Err(err) => {
                tracing::error!("Failed to fetch home posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {err}"),
                    Duration::milliseconds(600),
                );
            }
        }
    });

    let post_manager = POSTMANAGER.read();
    let Posts = post_manager.all_to_public();

    rsx!(
        Appbar {
            title: "Bookmarked",
            AppbarImgButton {
                click_handler: move |_| {
                    router().push(Route::HomeLiked {});
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
                    router().push(Route::Home {});
                },
                img: ICON_HOME,
                label: "Home",
                title: "Go to Home page",
            },
        }
        {Posts.into_iter()}
    )
}