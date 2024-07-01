use dioxus::prelude::*;
use crate::page::Route;


pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}