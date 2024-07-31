#![allow(non_snake_case)]

use chrono::Duration;
use dioxus::prelude::*;
use uchat_domain::PostId;
use uchat_endpoint::post::{
    endpoint::{Bookmark, BookmarkOk, Boost, BoostOk, React, ReactOk},
    types::{BookmarkAction, BoostAction, LikeStatus},
};

use crate::{
    async_handler, fetch_json,
    icon::{
        ICON_BOOKMARK, ICON_BOOKMARK_SAVED, ICON_BOOST, ICON_BOOSTED, ICON_DISLIKE,
        ICON_DISLIKE_SELECTED, ICON_LIKE, ICON_LIKE_SELECTED,
    },
    util::ApiClient,
    POSTMANAGER, TOASTER,
};

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
            false => BookmarkAction::Add,
        };

        let request_data = Bookmark { action, post_id };
        match fetch_json!(<BookmarkOk>, api_client, request_data) {
            Ok(res) => {
                POSTMANAGER
                    .write()
                    .update(post_id, |post| post.bookmarked = res.status.clone().into());
            }
            Err(e) => TOASTER.write().error(
                format!("Failed to bookmark post : {e}"),
                Duration::seconds(3),
            ),
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
pub fn Boost(post_id: PostId, boosted: bool, boosts: i64) -> Element {
    let api_client = ApiClient::global();
    let icon = match boosted {
        true => ICON_BOOSTED,
        false => ICON_BOOST,
    };

    let boost_onclick = async_handler!([api_client, post_id], move |_| async move {
        let action = match POSTMANAGER.read().get(&post_id).unwrap().boosted {
            true => BoostAction::Remove,
            false => BoostAction::Add,
        };

        let request_data = Boost { action, post_id };
        match fetch_json!(<BoostOk>, api_client, request_data) {
            Ok(res) => {
                POSTMANAGER.write().update(post_id, |post| {
                    post.boosted = res.status.clone().into();
                    if post.boosted {
                        post.boosts += 1;
                    } else {
                        post.boosts -= 1;
                    }
                });
            }
            Err(e) => TOASTER.write().error(
                format!("Failed to boost the post : {e}"),
                Duration::seconds(3),
            ),
        }
    });

    rsx!(
        div {
            class: "cursor-pointer",
            onclick: boost_onclick,
            img {
                class: "actionbar-icon",
                src: "{icon}",
            }
            div {
                class: "text-center",
                {boosts.to_string()}
            }
        }
    )
}

#[component]
pub fn LikeDislike(post_id: PostId, like_status: LikeStatus, likes: i64, dislikes: i64) -> Element {
    let api_client = ApiClient::global();
    let like_icon = match like_status {
        LikeStatus::Like => ICON_LIKE_SELECTED,
        _ => ICON_LIKE,
    };

    let dislike_icon = match like_status {
        LikeStatus::Dislike => ICON_DISLIKE_SELECTED,
        _ => ICON_DISLIKE,
    };

    let like_onclick = async_handler!([api_client, post_id], move |like_status| async move {
        let like_status = {
            if POSTMANAGER.read().get(&post_id).unwrap().like_status == like_status {
                LikeStatus::NoReaction
            } else {
                like_status
            }
        };
        let request_data = React {
            post_id,
            like_status,
        };
        match fetch_json!(<ReactOk>, api_client, request_data) {
            Ok(res) => {
                POSTMANAGER.write().update(post_id, |post| {
                    post.like_status = res.like_status;
                    post.likes = res.likes;
                    post.dislikes = res.dislikes
                });
            }
            Err(e) => TOASTER.write().error(
                format!("Failed to react to post : {e}"),
                Duration::seconds(3),
            ),
        }
    });

    rsx!(
        div {
            class: "cursor-pointer",
            onclick: move |_| like_onclick(LikeStatus::Like),
            img {
                class: "actionbar-icon",
                src: "{like_icon}",
            }
            div {
                class: "text-center",
                {likes.to_string()}
            }
        }

        div {
            class: "cursor-pointer",
            onclick: move |_| like_onclick(LikeStatus::Dislike),
            img {
                class: "actionbar-icon",
                src: "{dislike_icon}",
            }
            div {
                class: "text-center",
                {dislikes.to_string()}
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
            Boost {
                post_id: this_post_id,
                boosted: this_post.boosted,
                boosts: this_post.boosts
            }

            // bookmark
            Bookmark {
                post_id: this_post_id,
                bookmark: this_post.bookmarked
            }
            // like and dislike
            LikeDislike {
                like_status: this_post.like_status,
                post_id: this_post_id,
                likes: this_post.likes,
                dislikes: this_post.dislikes
            }
            // comment
        }

        // quick response
    )
}
