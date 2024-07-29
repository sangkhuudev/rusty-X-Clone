
#![allow(non_snake_case)]

use dioxus::prelude::*;
use uchat_domain::PostId;
use uchat_endpoint::post::types::{Chat as EndpointChat, Content as EndpointContent, PublicPost};

#[component]
pub fn Chat(post_id: PostId, content: EndpointChat) -> Element {
    let HeadLine = content.headline.as_ref().map(|headline| {
        rsx!(
            div {
                class: "font-bold",
                {headline.as_ref()}
            }
        )
    });

    rsx!(
        {HeadLine},
        p { "{content.message.as_ref()}" }
    )
}

#[component]
pub fn Content(post: PublicPost) -> Element {
    rsx!(
        match post.content {
            EndpointContent::Chat(content) => rsx!( Chat { post_id: post.id, content})
        }
    )
}