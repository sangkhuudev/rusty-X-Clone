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
                class: "w-full border-2 rounded border-slate-400",
                placeholder: "User name",
                oninput: move |ev| oninput.call(ev),

            }
        }
    }
}

pub fn Register() -> Element {
    let page_state = PageState::new();
    let page_state = use_signal(|| page_state);
    let username_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        page_state.with_mut(|state| state.username.set(ev.value().clone()));
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
            button {
                class: "px-4 py-72 rounded text-sm font-semibold bg-slate-600 shadow-sm text-white",
                r#type: "submit",
                "Signup",
            }
        }
    }
}