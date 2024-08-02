#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use itertools::Itertools;
use std::collections::HashSet;
use uchat_domain::{PollChoiceId, PostId};
use uchat_endpoint::post::{
    endpoint::{Vote, VoteOk},
    types::{
        Chat as EndpointChat, Content as EndpointContent, Image as EndpointImage, ImageKind,
        Poll as EndpointPoll, PublicPost, VoteCast,
    },
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
pub fn Poll(post_id: PostId, content: EndpointPoll) -> Element {
    let api_client = ApiClient::global();

    let vote_onclick = async_handler!(
        [api_client, post_id],
        move |post_id, choice_id| async move {
            let request_data = Vote { post_id, choice_id };
            match fetch_json!(<VoteOk>, api_client, request_data) {
                Ok(res) => match res.cast {
                    VoteCast::Yes => {
                        TOASTER
                            .write()
                            .success(format!("Vote casted"), Duration::seconds(3));
                    }
                    VoteCast::AlreadyVoted => {
                        TOASTER
                            .write()
                            .info(format!("Vote already casted"), Duration::seconds(3));
                    }
                },
                Err(e) => TOASTER
                    .write()
                    .error(format!("Failed to cast a vote : {e}"), Duration::seconds(3)),
            }
        }
    );

    let total_votes = content
        .choices
        .iter()
        .map(|choice| choice.num_votes)
        .sum::<i64>();
    let leader_ids = {
        let leaders = content
            .choices
            .iter()
            .max_set_by(|x, y| x.num_votes.cmp(&y.num_votes));
        let ids: HashSet<PollChoiceId> = HashSet::from_iter(leaders.iter().map(|choice| choice.id));
        ids
    };

    let Choices = content.choices.into_iter().map(|choice| {
        let percent = if total_votes > 0 {
            let percent = (choice.num_votes as f64 / total_votes as f64) * 100.0;
            format!("{percent:.0}%")
        } else {
            "0%".to_string()
        };

        let background_color = if leader_ids.contains(&choice.id) {
            "bg-blue-300"
        } else {
            "bg-neutral-300"
        };

        let foreground_styles = maybe_class!("font-bold", leader_ids.contains(&choice.id));
        rsx!(
            li {
                class: "grid grid-cols-[3rem_1fre] m-2 p-2 relative
                cursor-pointer border rounded border-slate-300",
                key: "{choice.id.to_string()}",
                onclick: move |_| vote_onclick(post_id, choice.id),
                div {
                    class: "absolute h-full rounded z-[-1] left-0 {background_color}",
                    style: "width: {percent}"
                }
                div {
                    class: "{foreground_styles}",
                    {percent.clone()}
                }
                div {
                    class: "{foreground_styles}",
                    {choice.description.as_ref()}
                }
            }
        )
    });

    let Headline = rsx!(figcaption {"{content.headline.as_ref()}"});

    rsx!(
        {Headline},
        ul {
            {Choices}
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
        EndpointContent::Poll(content) => rsx!(Poll {
            post_id: post.id,
            content: content
        }),
    })
}
