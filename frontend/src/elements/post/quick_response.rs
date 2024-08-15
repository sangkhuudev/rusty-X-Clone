#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use uchat_domain::Message;
use uchat_domain::PostId;
use uchat_endpoint::post::{
    endpoint::{NewPost, NewPostOk},
    types::{Chat, NewPostOptions},
};

//------------------------------------------------------------------------------------------
fn can_submit(message: &str) -> bool {
    message.len() <= Message::MAX_CHARS && !message.is_empty()
}

#[component]
pub fn MessageInput(message: String, on_input: EventHandler<FormEvent>) -> Element {
    let wrong_len = maybe_class!("err-text-color", !can_submit(&message));

    rsx!(
        div {
            class: "flex flex-row relative",
            textarea {
                class: "input-field",
                rows: 3,
                id: "message",
                value: "{message}",
                oninput: move |ev| {
                    info!("message input: {:#?}", ev);
                    on_input.call(ev)
                }
            },
            div {
                class: "text-right {wrong_len}",
                "{message.len()}/{Message::MAX_CHARS}"
            }
        }
    )
}

#[component]
pub fn QuickResponse(post_id: PostId, opened: Signal<bool>) -> Element {
    let api_client = ApiClient::global();
    let mut message = use_signal(|| String::new());

    let form_onsubmit = async_handler!([api_client], move |_| async move {
        info!("Form submitted!");
        let request_data = NewPost {
            content: Chat {
                message: Message::try_new(message.read().to_string()).unwrap(),
                headline: None,
            }
            .into(),
            options: NewPostOptions::default(),
        };

        let response = fetch_json!(<NewPostOk>, api_client, request_data);

        match response {
            Ok(_res) => {
                info!("Post new chat successfully!");
                TOASTER
                    .write()
                    .success("Posted successfully", Duration::seconds(3));
                opened.set(false);
            }
            Err(e) => {
                TOASTER
                    .write()
                    .error(format!("Reply failed: {e}"), Duration::seconds(3));
            }
        }
    });

    let submit_cursor = if can_submit(&message.read()) {
        "cursor-pointer"
    } else {
        "cursor-not-allowed"
    };

    let submit_btn_style = maybe_class!("btn-disabled", !can_submit(&message.read()));

    rsx!(
        form {
            onsubmit: form_onsubmit,
            prevent_default: "onsubmit",

            // message
            MessageInput {
                message: message,
                on_input: move |ev: FormEvent| {
                    message.set(ev.value());
                }
            }
            div {
                class: "flex flex-row justify-end w-full",
                button {
                    class: "mt-2 btn {submit_cursor} {submit_btn_style}",
                    r#type: "submit",
                    disabled: !can_submit(&message.read()),
                    "Respond"
                }
            }
        }
    )
}
