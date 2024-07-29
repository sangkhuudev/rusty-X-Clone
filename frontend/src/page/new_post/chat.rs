#![allow(non_snake_case)]

use crate::page::Route;
use crate::TOASTER;
use crate::{fetch_json, prelude::*, util::ApiClient};
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use serde::{Deserialize, Serialize};
use uchat_domain::{Headline, Message};
use uchat_endpoint::post::{
    endpoint::{NewPost, NewPostOk},
    types::{Chat, NewPostOptions},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PageState {
    pub message: String,
    pub headline: String,
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        if Message::try_new(&self.message).is_err() {
            return false;
        }
        if Headline::try_new(&self.headline).is_err() && !self.headline.is_empty() {
            return false;
        }
        true
    }
}

#[component]
pub fn MessageInput(page_state: Signal<PageState>) -> Element {
    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().message.len() > Message::MAX_CHARS
            || page_state.read().message.is_empty()
    );

    rsx!(
        div {
            label {
                r#for: "message",
                div {
                    class: "flex flex-row justify-between",
                    span {"Message"},
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().message.len()}/{Message::MAX_CHARS}"
                    }
                }
            }
            textarea {
                class: "input-field",
                rows: 5,
                id: "message",
                value: "{page_state.read().message}",
                oninput: move |ev| {
                    info!("message input: {:#?}", ev);
                    page_state.with_mut(|state| state.message = ev.value());
                }
            }

        }
    )
}

#[component]
pub fn HeadlineInput(page_state: Signal<PageState>) -> Element {
    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().headline.len() > Headline::MAX_CHARS
    );

    rsx!(
        div {
            label {
                r#for: "headline",
                div {
                    class: "flex flex-row justify-between",
                    span {"Headline"},
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().headline.len()}/{Headline::MAX_CHARS}"
                    }
                }
            }
            input {
                class: "input-field",
                id: "headline",
                value: "{page_state.read().headline}",
                oninput: move |ev| {
                    info!("headline input: {:#?}", ev);
                    page_state.with_mut(|state| state.headline = ev.value());
                }
            }

        }
    )
}

#[component]
pub fn NewChat() -> Element {
    info!("NewChat component initialized!");

    let api_client = ApiClient::global();
    let router = router();
    let page_state = use_signal(|| PageState::default());
    let submit_btn_style = maybe_class!("btn-disabled", !page_state.read().can_submit());
    let form_onsubmit = async_handler!([api_client, page_state, router], move |_| async move {
        info!("Form submitted!");
        let request_data = NewPost {
            content: Chat {
                message: Message::try_new(&page_state.read().message).unwrap(),
                headline: {
                    let headline = &page_state.read().headline;
                    if headline.is_empty() {
                        None
                    } else {
                        Some(Headline::try_new(headline).unwrap())
                    }
                },
            }
            .into(),
            options: NewPostOptions::default(),
        };

        let response = fetch_json!(<NewPostOk>, api_client, request_data);

        match response {
            Ok(_res) => {
                info!("Post new chat successfully!");
                TOASTER.write().success("Posted successfully", Duration::seconds(3));
                router.replace(Route::Home {});
            }
            Err(e) => {
                TOASTER
                    .write()
                    .error(format!("Posted failed: {e}"), Duration::seconds(3));
            }
        }
    });
    rsx!(
        form {
            class: "flex flex-col gap-4",
            onsubmit: form_onsubmit,
            // Message input
            MessageInput {
                page_state: page_state
            }
            // Headline input
            HeadlineInput {
                page_state: page_state
            }
            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !page_state.read().can_submit(),
                "Post"
            }
        }
    )
}
