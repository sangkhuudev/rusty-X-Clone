#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn NewChat() -> Element {
    rsx!(
        form {
            class: "flex flex-col gap-4",
            onsubmit: move |_| (),
            // Message input
            // Headline input
            button {
                class: "btn",
                r#type: "submit",
                disabled: true,
                "Post"
            }
        }
    )
}