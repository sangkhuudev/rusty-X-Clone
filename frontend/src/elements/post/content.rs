#![allow(non_snake_case)]

use dioxus::prelude::*;
use uchat_domain::PostId;
use uchat_endpoint::post::types::{
    Chat as EndpointChat, Content as EndpointContent, Image as EndpointImage, ImageKind, PublicPost,
};

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
pub fn Image(post_id: PostId, content: EndpointImage) -> Element {
    let url = if let ImageKind::Url(url) = &content.kind {
        url
    } else {
        return rsx!(div { "Image not found"});
    };
    let caption = content
        .caption
        .as_ref()
        .map(|caption| rsx!( figcaption { em { "{caption.as_ref()}"}}));

    rsx!(
        figure {
            class: "flex flex-col gap-2",
            {caption},
            img {
                class: "w-full object-contain max-h-[80vh]",
                src: "{url}"
            }
        }
    )
}

#[component]
pub fn Content(post: PublicPost) -> Element {
    rsx!(match post.content {
        EndpointContent::Chat(content) => rsx!(Chat {
            post_id: post.id,
            content: content
        }),
        EndpointContent::Image(content) => rsx!(Image {
            post_id: post.id,
            content: content
        }),
        _ => rsx!( div {""}),
    })
}
