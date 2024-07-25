#![allow(non_snake_case)]

use crate::page::{Route, ToastRoot};
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
        ToastRoot { }
    }    
}
