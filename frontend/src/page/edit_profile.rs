#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use keyed_notifications_box::KeyedNotifications;
use web_sys::HtmlInputElement;
use uchat_domain::user::Email;

#[derive(Debug, Clone)]
pub enum PreviewImageData {
    DataUrl(String),
    Remote(String)
}

#[derive(Debug, Clone, Default)]
pub struct PageState {
    pub display_name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub profile_image: Option<PreviewImageData>,

    pub form_error: KeyedNotifications
}

#[component]
pub fn ImageInput(page_state: Signal<PageState>) -> Element {
    let image_oninput = move |_| {
        async move {
            // this is used for unchecked_into()
            use gloo_file::{futures::read_as_data_url, File};
            use wasm_bindgen::JsCast;

            let element_html = crate::util::document()
                .get_element_by_id("image-input")
                .unwrap()
                .unchecked_into::<HtmlInputElement>();
            let file: File = element_html.files().unwrap().get(0).unwrap().into();

            match read_as_data_url(&file).await {
                Ok(data) => page_state
                .with_mut(|state| 
                    state.profile_image = Some(PreviewImageData::DataUrl(data))
                ),
                Err(e) => TOASTER
                    .write()
                    .error(format!("Failed to load file: {}", e), Duration::seconds(5)),
            }
        }
    };

    rsx!(
        div {
            label {
                r#for: "image-input",
                "Upload image"
            }
            input {
                class: "w-full",
                id: "image-input",
                r#type: "file",
                accept: "image/*",
                oninput: image_oninput
            }
        }
    )
}

#[component]
pub fn ImagePreview(page_state: Signal<PageState>) -> Element {
    let image_data = page_state.with(|state| state.profile_image.clone());
    let image_element = |img_src: &str| {
        rsx!(
            img {
                class: "profile-portrait-lg",
                src: "{img_src}"
            }
        )
    };

    let image_data = match image_data {
        Some(PreviewImageData::DataUrl(ref data)) => image_element(data),
        Some(PreviewImageData::Remote(ref url)) => image_element(url),
        None => rsx!( div {"No image uploaded"})
    };

    rsx!(
        div {
            class: "flex flex-row justify-center",
            {image_data}
        }
    )
}

#[component]
pub fn EmailInput(page_state: Signal<PageState>) -> Element {
    rsx!(
        div {
            label {
                r#for: "email",
                div {
                    class: "flex flex-row justify-between",
                    span {"Email Address"}
                }
            }
            input {
                class: "input-field",
                id: "email",
                placeholder: "Email address",
                value: "{page_state.read().email}",
                oninput: move |ev| {
                    if let Err(e) = Email::try_new(&ev.value()) {
                        page_state.with_mut(|state| state.form_error.set("Bad email", e.to_string()));
                    } else {
                        page_state.with_mut(|state| state.form_error.remove("Bad email"));
                    }
                    page_state.with_mut(|state| state.email = ev.value());
                }
            }

        }
    )
}

pub fn EditProfile() -> Element {
    let page_state = use_signal(|| PageState::default());
    rsx!(
        form {
            class: "flex flex-col gap-3 w-full",
            ImagePreview {
                page_state: page_state
            }
            ImageInput {
                page_state: page_state
            }
            EmailInput {
                page_state: page_state
            
            }
            // Error notifications component
            KeyedNotificationsBox {
                legend: "Form errors",
                notification: page_state.read().form_error.clone()
            }
            div {
                class: "flex flex-row gap-3 justify-end",
                button {
                    class: "btn",
                    onclick: move |_| navigator().go_back(),
                    "Cancel"
                }
                button {
                    class: "btn",
                    r#type: "submit",
                    "Submit"
                }
            }
        }
    )
}