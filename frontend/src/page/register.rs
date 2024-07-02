#![allow(non_snake_case)]

use dioxus::prelude::*;
use uchat_domain::Username;
use crate::elements::keyed_notifications_box::{
    KeyedNotifications, KeyedNotificationsBox
}; 
use crate::prelude::*;

pub struct PageState {
    pub username: Signal<String>,
    pub password: Signal<String>,
    pub form_error: KeyedNotifications
}

impl PageState {
    pub fn new() -> Self {
        Self {
            username: use_signal(String::new),
            password: use_signal(String::new),
            form_error: KeyedNotifications::default()
        }
    }
}

#[component]
pub fn UsernameInput (
    state: Signal<String>,
    oninput: EventHandler<FormEvent>
) -> Element {
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
                value: "{state.read().clone()}",
                oninput: move |ev| oninput.call(ev),

            }
        }
    }
}

#[component]
pub fn PasswordInput (
    state: Signal<String>,
    oninput: EventHandler<FormEvent>
) -> Element {
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
                value: "{state.read().clone()}",
                oninput: move |ev| oninput.call(ev),

            }
        }
    }
}

pub fn Register() -> Element {
    let page_state = PageState::new();
    let page_state = use_signal(|| page_state);
    let username_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        if let Err(e) = Username::new(&ev.value()) {
            page_state.with_mut(|state| state.form_error.set("Bad username", e.to_string()));
        } else {

            page_state.with_mut(|state| state.form_error.remove("Bad username"));
        }
        page_state.with_mut(|state| state.username.set(ev.value()));
    });

    let password_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        page_state.with_mut(|state| state.password.set(ev.value()));
    });

    rsx! {
        form {
            class: "flex flex-col gap-5",
            prevent_default: "onsubmit",
            onsubmit: move |_| {

            },
            UsernameInput {
                state: page_state.with(|state| state.username.clone()),
                oninput: username_oninput
            },

            PasswordInput {
                state: page_state.with(|state| state.password.clone()),
                oninput: password_oninput
            },
            KeyedNotificationsBox {
                legend: "Form errors",
                notification: page_state.with(|state| state.form_error.clone())
            }
            button {
                class: "btn",
                r#type: "submit",
                "Signup",
            }
        }
    }
}