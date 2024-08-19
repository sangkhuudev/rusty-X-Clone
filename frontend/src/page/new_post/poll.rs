#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use serde::{Deserialize, Serialize};
use uchat_domain::{PollChoiceDescription, PollChoiceId, PollHeadline};
use uchat_endpoint::post::{
    endpoint::{NewPost, NewPostOk},
    types::{NewPostOptions, Poll, PollChoice},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PageState {
    pub headline: String,
    pub poll_choices: BTreeMap<usize, String>,
    pub next_id: usize,
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            headline: "".to_string(),
            poll_choices: {
                let mut map = BTreeMap::new();
                map.insert(0, "".to_string());
                map.insert(1, "".to_string());
                map
            },
            next_id: 2,
        }
    }
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        if PollHeadline::try_new(&self.headline).is_err() {
            return false;
        }

        if self.poll_choices.len() < 2 {
            return false;
        }

        if self
            .poll_choices
            .values()
            .map(PollChoiceDescription::try_new)
            .collect::<Result<Vec<PollChoiceDescription>, _>>()
            .is_err()
        {
            return false;
        }
        true
    }

    pub fn push_choice<T: Into<String>>(&mut self, choice: T) {
        self.poll_choices.insert(self.next_id, choice.into());
        self.next_id += 1;
    }

    pub fn replace_choice<T: Into<String>>(&mut self, key: usize, choice: T) {
        self.poll_choices.insert(key, choice.into());
    }
}

#[component]
pub fn HeadlineInput(page_state: Signal<PageState>) -> Element {
    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().headline.len() > PollHeadline::MAX_CHARS
            || page_state.read().headline.is_empty()
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
                        "{page_state.read().headline.len()}/{PollHeadline::MAX_CHARS}"
                    }
                }
            }
            input {
                class: "input-field",
                id: "headline",
                value: "{page_state.read().headline}",
                oninput: move |ev| {
                    info!("Poll headline input: {:#?}", ev);
                    page_state.with_mut(|state| state.headline = ev.value());
                }
            }

        }
    )
}

#[component]
pub fn PollChoices(page_state: Signal<PageState>) -> Element {
    let poll_choices = &page_state.read().poll_choices;
    let choices = poll_choices.iter().map(|(&key, choice)| {
        let choice = choice.clone();
        let wrong_len = maybe_class!(
            "err-text-color",
            PollChoiceDescription::try_new(&choice).is_err()
        );
        rsx!(
            li {
                key: "{key}",
                div {
                    class: "grid grid-cols-[1fr_3rem_3rem]
                        gap-2 items-center h-8 w-full",
                    input {
                        class: "input-field",
                        placeholder: "Choice description",
                        oninput: move |ev| {
                            page_state.with_mut(|state| state.replace_choice(key, &ev.data.value()))
                        },
                        value: "{choice}"
                    }
                    div {
                        class: "text-right {wrong_len}",
                        "{choice.len()}/{PollChoiceDescription::MAX_CHARS}"
                    }
                    button {
                        class: "p-0 h-full bg-red-700",
                        onclick: move |_| {
                            page_state.with_mut(|state| state.poll_choices.remove(&key));
                        },
                        "X"
                    }
                }
            }
        )
    });
    rsx!(
        div {
            class: "flex flex-col gap-2",
            "Poll Choice",
            ol {
                class: "flex flex-col gap-2 list-decimal",
                {choices}
            }
            div {
                class: "flex flex-row justify-end",
                button {
                    class: "btn w-12",
                    prevent_default: "onclick",
                    onclick: move |_| {
                        page_state.with_mut(|state| state.push_choice(""))
                    },
                    "+"
                }
            }
        }
    )
}

#[component]
pub fn NewPoll() -> Element {
    info!("NewChat component initialized!");

    let api_client = ApiClient::global();
    let router = router();
    let page_state = use_signal(|| PageState::default());
    let submit_btn_style = maybe_class!("btn-disabled", !page_state.read().can_submit());
    let form_onsubmit = async_handler!([api_client, page_state, router], move |_| async move {
        info!("Form submitted!");
        let request_data = NewPost {
            content: Poll {
                headline: {
                    let headline = &page_state.read().headline;
                    PollHeadline::try_new(headline).expect("Headline can not be empty")
                },
                choices: {
                    page_state
                        .read()
                        .poll_choices
                        .values()
                        .map(|choice| {
                            let id = PollChoiceId::new();
                            PollChoice {
                                id,
                                num_votes: 0,
                                description: PollChoiceDescription::try_new(choice).unwrap(),
                            }
                        })
                        .collect::<Vec<PollChoice>>()
                },
                voted: None,
            }
            .into(),
            options: NewPostOptions::default(),
        };

        let response = fetch_json!(<NewPostOk>, api_client, request_data);

        match response {
            Ok(_res) => {
                info!("Post new poll successfully!");
                TOASTER
                    .write()
                    .success("Posted successfully", Duration::microseconds(1500));
                router.replace(Route::Home {});
            }
            Err(e) => {
                TOASTER.write().error(
                    format!("Post poll failed: {e}"),
                    Duration::milliseconds(1500),
                );
            }
        }
    });
    rsx!(
        Appbar {
            title: "New Poll",
            AppbarImgButton {
                click_handler: move |_| {
                    router.push(Route::NewChat {});
                },
                img: ICON_MESSAGES,
                label: "Chat",
                title: "Post a new chat",
            },
            AppbarImgButton {
                click_handler: move |_| {
                    router.push(Route::NewImage {});
                },
                img: ICON_IMAGE,
                label: "Image",
                title: "Post a new image",
            },
            AppbarImgButton {
                click_handler: move |_| {},
                img: ICON_POLL,
                label: "Poll",
                title: "Post a new poll",
                disabled: true,
                append_class: appbar::BUTTON_SELECTED,
            },
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().go_back();
                },
                img: ICON_BACK,
                label: "Back",
                title: "Go to the previous page",
            }
        }

        form {
            class: "flex flex-col gap-4",
            onsubmit: form_onsubmit,
            // Image input
            HeadlineInput {
                page_state: page_state
            }
            // poll choices
            PollChoices {
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
