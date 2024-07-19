#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::elements::keyed_notifications_box::{KeyedNotifications, KeyedNotificationsBox};
use crate::page::Route;
use crate::util::ApiClient;
use crate::{fetch_json, prelude::*};
use uchat_domain::{Password, Username};
use uchat_endpoint::user::endpoint::{Login, LoginOk};
use dioxus_logger::tracing::{info, error};

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
        div {
            class: "flex flex-col",
            label {
                r#for: "username",
                "Username",
            },
            input {
                id: "username",
                name: "username",
                class: "input-field",
                placeholder: "User name",
                value: "{state.read()}",
                oninput: move |ev| oninput.call(ev),
            }
        }
    }
}

#[component]
pub fn PasswordInput(state: Signal<String>, oninput: EventHandler<FormEvent>) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            label {
                r#for: "password",
                "Password",
            },
            input {
                id: "password",
                r#type: "password",
                name: "password",
                class: "input-field",
                placeholder: "Password",
                value: "{state.read()}",
                oninput: move |ev| oninput.call(ev),
            }
        }
    }
}

#[component]
pub fn Login() -> Element {
    info!("Login component initialized!");

    let api_client = ApiClient::global();
    let page_state = PageState::new();
    let page_state = use_signal(|| page_state);
    let router = router();

    let form_onsubmit = async_handler!([api_client, page_state, router], move |_| async move {
        info!("Form submitted!");

        let request_data = Login {
            username: Username::try_new(page_state.with(|state| state.username.read().to_string()))
            .expect("Username is not valid!"),
            password: Password::try_new(page_state.with(|state| state.password.read().to_string()))
            .expect("There is somthing wrong with password"),
        };

        let response = fetch_json!(<LoginOk>, api_client, request_data);

        match response {
            Ok(res) => {
                info!("Login successfully!");
                crate::util::cookie::set_session(
                    res.session_signature,
                    res.session_id,
                    res.session_expires,
                );
                router.push(Route::Home);
            }
            Err(err) => {
                error!("Login failed: {:?}", err);
            }
        }
    });

    let username_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        info!("Username input changed: {}", ev.value());
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
            // prevent_default: "onsubmit",
            onsubmit: form_onsubmit,

            // Username input component
            UsernameInput { 
                state: page_state.with(|state| state.username), 
                oninput: username_oninput 
            }

            // Password input component
            PasswordInput { 
                state: page_state.with(|state| state.password), 
                oninput: password_oninput 
            }

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
                "Login",
            }
        }
    }
}