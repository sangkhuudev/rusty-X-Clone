#![allow(non_snake_case)]

use crate::page::Route;
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        main {
            class: "max-w-[var(--content-max-width)]
            min-w-[var(--content-min-width)]
            mt-[var(--appbar-height)]
            mb-[var(--navbar-height)]
            mx-auto
            p-4",

            Router::<Route> {
                config: || RouterConfig::default().history(WebHistory::default()),
            }
        }
    }
}
