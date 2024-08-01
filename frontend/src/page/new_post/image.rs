#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use serde::{Deserialize, Serialize};
use uchat_domain::Caption;
use uchat_endpoint::post::{
    endpoint::{NewPost, NewPostOk},
    types::{Image, ImageKind, NewPostOptions},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PageState {
    pub caption: String,
    pub image: Option<String>,
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        if !self.caption.is_empty() && Caption::try_new(&self.caption).is_err() {
            return false;
        }
        if self.image.is_none() {
            return false;
        }
        true
    }
}

#[component]
pub fn CaptionInput(page_state: Signal<PageState>) -> Element {
    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().caption.len() > Caption::MAX_CHARS
    );

    rsx!(
        div {
            label {
                r#for: "caption",
                div {
                    class: "flex flex-row justify-between",
                    span {"caption (optional)"},
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().caption.len()}/{Caption::MAX_CHARS}"
                    }
                }
            }
            input {
                class: "input-field",
                id: "caption",
                value: "{page_state.read().caption}",
                oninput: move |ev| {
                    info!("caption input: {:#?}", ev);
                    page_state.with_mut(|state| state.caption = ev.value());
                }
            }

        }
    )
}

#[component]
pub fn NewImage() -> Element {
    info!("NewChat component initialized!");

    let api_client = ApiClient::global();
    let router = router();
    let page_state = use_signal(|| PageState::default());
    let submit_btn_style = maybe_class!("btn-disabled", !page_state.read().can_submit());
    let form_onsubmit = async_handler!([api_client, page_state, router], move |_| async move {
        info!("Form submitted!");
        let request_data = NewPost {
            content: Image {
                caption: {
                    let caption = &page_state.read().caption;
                    if caption.is_empty() {
                        None
                    } else {
                        Some(Caption::try_new(caption).unwrap())
                    }
                },
                kind: ImageKind::DataUrl(page_state.read().image.clone().unwrap()),
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
            // Image input
            // Image preview
            CaptionInput {
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
