#![allow(non_snake_case)]

use chrono::Duration;
use dioxus::prelude::*;

use crate::TOASTER;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            h1 { "This is home page" }
        }
        button {
            onclick: move |_| {
                TOASTER.write().success("Success", Duration::seconds(5));
                TOASTER.write().info("Info", Duration::seconds(5));
                TOASTER.write().error("Error", Duration::seconds(5));
            },
            "Toast"
        }
    }
}
