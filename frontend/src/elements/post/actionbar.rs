
#![allow(non_snake_case)]

use chrono::Duration;
use dioxus::prelude::*;
use uchat_domain::PostId;
use uchat_endpoint::post::{endpoint::{Bookmark, BookmarkOk}, types::BookmarkAction};

use crate::{async_handler, fetch_json, icon::{ICON_BOOKMARK, ICON_BOOKMARK_SAVED}, util::ApiClient, POSTMANAGER, TOASTER};

#[component]
pub fn Bookmark(post_id: PostId, bookmark: bool) -> Element {
    let api_client = ApiClient::global();
    let icon = match bookmark {
        true => ICON_BOOKMARK_SAVED,
        false => ICON_BOOKMARK, 
    };

    let bookmark_onclick = async_handler!([api_client, post_id], move |_| async move {
        let action = match POSTMANAGER.read().get(&post_id).unwrap().bookmarked {
            true => BookmarkAction::Remove,
            false => BookmarkAction::Add
        };

        let request_data = Bookmark { action, post_id };
        match fetch_json!(<BookmarkOk>, api_client, request_data) {
            Ok(res) => {
                POSTMANAGER.write().update(post_id, |post| {
                    post.bookmarked = res.status.clone().into()
                });
            }
            Err(e) => {
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {e}"),
                    Duration::seconds(3)
                )
            }
        }
    });
    
    rsx!(
        div {
            class: "cursor-pointer",
            onclick: bookmark_onclick,
            img {
                class: "actionbar-icon",
                src: "{icon}",
            }
        }
    )
}
#[component]
pub fn Actionbar(post_id: PostId) -> Element {
    let post_manager = POSTMANAGER.read();
    let this_post = post_manager.get(&post_id).unwrap();
    let this_post_id = this_post.id;

    rsx!(
        div {
            key: "{this_post.id.to_string()}",
            class: "flex flex-row justify-between w-full opacity-70 mt-4",
            // boost
            // bookmark
            // like and dislike
            // comment
        }

        // quick response
    )
}