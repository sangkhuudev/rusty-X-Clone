#![allow(non_snake_case)]

use crate::page::Route;
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }    
}
