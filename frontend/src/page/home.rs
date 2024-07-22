#![allow(non_snake_case)]

use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {

    rsx! {
        div {
            h1 { "This is home page" }
        }
    }
}

