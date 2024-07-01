#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::prelude::*;

pub struct PageState {
    pub username: Signal<String>,
    pub password: Signal<String>,
}

impl PageState {
    pub fn new() -> Self {
        Self {
            username: use_signal(String::new),
            password: use_signal(String::new)
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
                name: "password",
                class: "input-field",
                placeholder: "Password",
                oninput: move |ev| oninput.call(ev),

            }
        }
    }
}

pub fn Register() -> Element {
    let page_state = PageState::new();
    let page_state = use_signal(|| page_state);
    let username_oninput = sync_handler!([page_state], move |ev: FormEvent| {
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
            button {
                class: "button",
                r#type: "submit",
                "Signup",
            }
        }
    }
}