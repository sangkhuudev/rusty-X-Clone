#![allow(non_snake_case)]

use crate::elements::keyed_notifications_box::{KeyedNotifications, KeyedNotificationsBox};
use crate::util::ApiClient;
use crate::{fetch_json, prelude::*};
use dioxus::prelude::*;
use uchat_domain::{Password, Username};
use uchat_endpoint::user::endpoint::{CreateUser, CreateUserOk};


pub struct PageState {
    pub username: Signal<String>,
    pub password: Signal<String>,
    pub form_error: KeyedNotifications,
}

impl PageState {
    pub fn new() -> Self {
        Self {
            username: use_signal(String::new),
            password: use_signal(String::new),
            form_error: KeyedNotifications::default(),
        }
    }
    pub fn can_submit(&self) -> bool {
        !(self.form_error.has_message()
            || self.username.read().is_empty()
            || self.password.read().is_empty())
    }
}

#[component]
pub fn UsernameInput(state: Signal<String>, oninput: EventHandler<FormEvent>) -> Element {
    rsx! {
        div { class: "flex flex-col",
            label { r#for: "username", "Username" }
            input {
                id: "username",
                name: "username",
                class: "input-field",
                placeholder: "User name",
                value: "{state.read()}",
                oninput: move |ev| oninput.call(ev)
            }
        }
    }
}

#[component]
pub fn PasswordInput(state: Signal<String>, oninput: EventHandler<FormEvent>) -> Element {
    rsx! {
        div { class: "flex flex-col",
            label { r#for: "password", "Password" }
            input {
                id: "password",
                r#type: "password",
                name: "password",
                class: "input-field",
                placeholder: "Password",
                value: "{state.read()}",
                oninput: move |ev| oninput.call(ev)
            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    let api_client = ApiClient::global();
    let page_state = PageState::new();
    let page_state = use_signal(|| page_state);

    let form_onsubmit = async_handler!([api_client, page_state], move |_| async move {
        let request_data = CreateUser {
            username: Username::try_new(page_state.with(|state| state.username.read().to_string()))
                .unwrap(),
            password: Password::try_new(page_state.with(|state| state.password.read().to_string()))
                .unwrap(),
        };
        let response = fetch_json!(<CreateUserOk>, api_client, request_data);
    
        match response {
            Ok(_resp) => tracing::info!("Form submitted successfully"),
            Err(_err) => tracing::error!("Error submitting form: {:?}", _err),
        }
    });
    
    let username_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        if let Err(e) = Username::try_new(&ev.value()) {
            page_state.with_mut(|state| state.form_error.set("Bad username", e.to_string()));
        } else {
            page_state.with_mut(|state| state.form_error.remove("Bad username"));
        }
        page_state.with_mut(|state| state.username.set(ev.value()));
    });

    let password_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        if let Err(e) = Password::try_new(&ev.value()) {
            page_state.with_mut(|state| state.form_error.set("Bad password", e.to_string()));
        } else {
            page_state.with_mut(|state| state.form_error.remove("Bad password"));
        }
        page_state.with_mut(|state| state.password.set(ev.value()));
    });
    let btn_submit_style = match page_state.with(|state| state.can_submit()) {
        false => "btn-disabled",
        true => "",
    };
    rsx! {
        form {
            class: "flex flex-col gap-5",
            prevent_default: "onsubmit",
            onsubmit: form_onsubmit,

            // Username input component
            UsernameInput { state: page_state.with(|state| state.username), oninput: username_oninput }

            // Password input component
            PasswordInput { state: page_state.with(|state| state.password), oninput: password_oninput }

            // Error notifications component
            KeyedNotificationsBox {
                legend: "Form errors",
                notification: page_state.with(|state| state.form_error.clone())
            }

            // Submit button
            button {
                class: "btn {btn_submit_style}",
                r#type: "submit",
                disabled: !page_state.with(|state| state.can_submit()),
                "Signup"
            }
        }
    }
}
