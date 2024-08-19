#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use keyed_notifications_box::KeyedNotifications;
use uchat_domain::{
    user::{DisplayName, Email},
    Password,
};
use uchat_endpoint::{
    user::endpoint::{GetMyProfile, GetMyProfileOk, UpdateProfile, UpdateProfileOk},
    Update,
};
use web_sys::HtmlInputElement;

#[derive(Debug, Clone)]
pub enum PreviewImageData {
    DataUrl(String),
    Remote(String),
}

#[derive(Debug, Clone, Default)]
pub struct PageState {
    pub display_name: String,
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub profile_image: Option<PreviewImageData>,

    pub form_error: KeyedNotifications,
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

            if let Some(files) = element_html.files() {
                if let Some(file) = files.get(0) {
                    let file: File = file.into();
                    
                    match read_as_data_url(&file).await {
                        Ok(data) => page_state
                            .with_mut(|state| state.profile_image = Some(PreviewImageData::DataUrl(data))),
                        Err(e) => TOASTER.write().error(
                            format!("Failed to load file: {}", e),
                            Duration::milliseconds(600),
                        ),
                    }
                    
                } else {
                    TOASTER
                        .write()
                        .error("No file selected", Duration::milliseconds(600));
                }
            } else {
                TOASTER
                    .write()
                    .error("Failed to access files", Duration::milliseconds(600));
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
        rsx!(img {
            class: "profile-portrait-lg",
            src: "{img_src}"
        })
    };

    let image_data = match image_data {
        Some(PreviewImageData::DataUrl(ref data)) => image_element(data),
        Some(PreviewImageData::Remote(ref url)) => image_element(url),
        None => rsx!( div {"No image uploaded"}),
    };

    rsx!(
        div {
            class: "flex flex-row justify-center",
            {image_data}
        }
    )
}

#[component]
pub fn PasswordInput(page_state: Signal<PageState>) -> Element {
    let mut check_password_matched = move || {
        let password_matches =
            page_state.with(|state| state.password == state.password_confirmation);

        match password_matches {
            true => page_state.with_mut(|state| state.form_error.remove("Password-mismatch")),
            false => page_state.with_mut(|state| {
                state
                    .form_error
                    .set("Password-mismatch", "Password must match".to_string())
            }),
        }
    };

    rsx!(
        fieldset {
            class: "fieldset",
            legend {"Set new password"}
            div {
                class: "flex flex-row gap-2 w-full",
                div {
                    label {
                        r#for: "password",
                        "Password"
                    }
                    input {
                        id: "password",
                        class: "input-field",
                        r#type: "password",
                        placeholder: "Password",
                        value: "{page_state.read().password}",
                        oninput: move |ev| {
                            match Password::try_new(&ev.value()) {
                                Ok(_) => page_state.with_mut(|state| state.form_error.remove("Bad password")),
                                Err(e) => page_state.with_mut(|state| state.form_error.set("Bad password", e.to_string())),
                            }

                            page_state.with_mut(|state| state.password = ev.value());
                            page_state.with_mut(|state| state.password_confirmation = "".to_string());

                            if page_state.with(|state| state.password.is_empty()) {
                                page_state.with_mut(|state| state.form_error.remove("Bad password"));
                                page_state.with_mut(|state| state.form_error.remove("Password-mismatch"));
                            } else {
                                check_password_matched();
                            }
                        }
                    }
                }

                div {
                    label {
                        r#for: "password-confirm",
                        "Confirm password"
                    }
                    input {
                        id: "password-confirm",
                        class: "input-field",
                        r#type: "password",
                        placeholder: "Confirm password",
                        value: "{page_state.read().password_confirmation}",
                        oninput: move |ev| {
                            page_state.with_mut(|state| state.password_confirmation = ev.value());
                            check_password_matched();
                        }
                    }
                }
            }
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
                    if !&ev.value().is_empty() {
                        match Email::try_new(&ev.value()) {
                            Ok(_) => {
                                page_state.with_mut(|state| state.form_error.remove("bad-email"));
                            }
                            Err(e) => {
                                page_state.with_mut(|state| state.form_error.set("bad-email", e.to_string()));
                            }
                        }
                    } else {
                        page_state.with_mut(|state| state.form_error.remove("bad-email"));
                    }

                    page_state.with_mut(|state| state.email = ev.value());
                }
            }

        }
    )
}

#[component]
pub fn DisplayNameInput(page_state: Signal<PageState>) -> Element {
    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().display_name.len() > DisplayName::MAX_CHARS
    );

    rsx!(
        div {
            label {
                r#for: "display-name",
                div {
                    class: "flex flex-row justify-between",
                    span {"Display Name"}
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().display_name.len()}/{DisplayName::MAX_CHARS}"
                    }
                }
            }
            input {
                class: "input-field",
                id: "display-name",
                placeholder: "Display name",
                value: "{page_state.read().display_name}",
                oninput: move |ev| {
                    match DisplayName::try_new(&ev.value()) {
                        Ok(_) => {
                            page_state.with_mut(|state| state.form_error.remove("bad-displayname"));
                        }
                        Err(e) => {
                            page_state.with_mut(|state| state.form_error.set("bad-displayname", e.to_string()));
                        }
                    }
                    page_state.with_mut(|state| state.display_name = ev.value());
                }
            }

        }
    )
}

pub fn EditProfile() -> Element {
    let api_client = ApiClient::global();
    let mut page_state = use_signal(|| PageState::default());
    let disabled_submit = page_state.with(|state| state.form_error.has_message());
    let submit_btn_style = maybe_class!("btn-disabled", disabled_submit);

    // Fetch profile posts asynchronously
    let _fetch_profile = use_resource(move || async move {
        tracing::info!("Starting request to fetch profiles");

        // Define a timeout duration and start fetching data
        match fetch_json!(<GetMyProfileOk>, api_client, GetMyProfile) {
            Ok(data) => {
                page_state.with_mut(|state| {
                    state.display_name = data.display_name.unwrap_or_default();
                    state.email = data.email.unwrap_or_default();
                    state.profile_image = data
                        .profile_image
                        .map(|image| PreviewImageData::Remote(image.to_string()));
                });

                TOASTER
                    .write()
                    .info("Retrieving  profiles...", Duration::milliseconds(1500));
            }
            Err(err) => {
                tracing::error!("Failed to fetch profiles: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {err}"),
                    Duration::milliseconds(1500),
                );
            }
        }
    });

    let form_onsubmit = async_handler!([api_client, page_state, router], move |_| async move {
        info!("Form submitted!");

        let request_data = UpdateProfile {
            display_name: {
                let name = page_state.with(|state| state.display_name.clone());
                if name.is_empty() {
                    Update::SetNull
                } else {
                    Update::Change(name)
                }
            },
            email: {
                let email = page_state.with(|state| state.email.clone());
                if email.is_empty() {
                    Update::SetNull
                } else {
                    Update::Change(email)
                }
            },
            profile_image: {
                let profile_image = page_state.with(|state| state.profile_image.clone());
                match profile_image {
                    Some(PreviewImageData::DataUrl(data)) => Update::Change(data),
                    Some(PreviewImageData::Remote(_)) => Update::NoChange,
                    None => Update::SetNull,
                }
            },
            password: {
                let password = page_state.with(|state| state.password.clone());

                if password.is_empty() {
                    Update::NoChange
                } else {
                    Update::Change(Password::try_new(password).unwrap())
                }
            },
        };

        let response = fetch_json!(<UpdateProfileOk>, api_client, request_data);

        match response {
            Ok(res) => {
                info!("Profile updated successfully!");
                LOCAL_PROFILE.write().image = res.profile_image;
                TOASTER
                    .write()
                    .success("Profile updated", Duration::milliseconds(600));
                router().replace(Route::Home {});
            }
            Err(err) => {
                error!("Login failed: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to update profile: {}", err),
                    Duration::milliseconds(600),
                );
            }
        }
    });

    rsx!(
        Appbar {
            title: "Edit profile",
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
            class: "flex flex-col gap-3 w-full",
            onsubmit: form_onsubmit,
            ImagePreview {
                page_state: page_state
            }
            ImageInput {
                page_state: page_state
            }
            DisplayNameInput {
                page_state: page_state
            }
            EmailInput {
                page_state: page_state
            }
            PasswordInput {
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
                    class: "btn {submit_btn_style}",
                    r#type: "submit",
                    disabled: disabled_submit,
                    "Submit"
                }
            }
        }
    )
}
