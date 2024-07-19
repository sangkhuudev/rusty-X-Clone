#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct NavButtonProps {
    img: &'static str,
    label: &'static str,
    onclick: EventHandler<MouseEvent>,
    highlight: Option<bool>,
    children: Element,
}


pub const BUTTON_CLASS: &str = "grid grid-cols-[20px_1fr] gap-4 pl-4
    justify-center items-center w-full h-12 border-y navbar-border-color";

#[component]
pub fn NewPostPopup(hide: Signal<bool>) -> Element {
    let hide_class = maybe_class!("hidden", *hide.read());
    rsx!(
        div {
            class: "flex flex-col absolute right-0 bottom-[var(--navbar-height)]
                w-28 items-center {hide_class} text-white text-sm navbar-bg-color",
            div {
                class: BUTTON_CLASS,
                onclick: move |_| {},
                img {
                    class: "invert",
                    src: "icon-poll.svg",
                }
                "Poll"
            } 
            div {
                class: BUTTON_CLASS,
                onclick: move |_| {},
                img {
                    class: "invert",
                    src: "icon-image.svg",
                }
                "Image"
            }
            div {
                class: BUTTON_CLASS,
                onclick: move |_| {},
                img {
                    class: "invert",
                    src: "icon-messages.svg",
                }
                "Chat"
            }
        }
    )
}

pub fn NavButton(props: NavButtonProps) -> Element {
    let selected_bg_color = maybe_class!("bg-slate-500", matches!(props.highlight, Some(true)));

    rsx!(
        button {
            class: "cursor-pointer flex flex-col justify-center
            h-full items-center {selected_bg_color}",
            onclick: move |ev| props.onclick.call(ev),
            img {
                class: "invert",
                src: props.img,
                width: "25px",
                height: "25px",
            },
            div {
                class: "text-sm text-white",
                {props.label}
            },
            {&props.children}

        }
    )
}

pub fn Navbar() -> Element {
    let mut hide_new_post_popup = use_signal(|| true);

    rsx!(
        nav {
            class: "max-w-[var(--content-max-width)] h-[var(--navbar-height)]
                fixed bottom-0 left-0 right-0 mx-auto
                border-t navbar-bg-color navbar-border-color",
            div {
                class: "grid grid-cols-3 justify-around w-full h-
                full items-center shadow-inner",

                NavButton {
                    img: "icon-home.svg",
                    label: "Home",
                    onclick: move |_| {}
                }

                NavButton {
                    img: "icon-trending.svg",
                    label: "Trending",
                    onclick: move |_| {}
                }

                NavButton {
                    img: "icon-new-post.svg",
                    label: "Post",
                    onclick: move |_| {
                        let is_hidden = *hide_new_post_popup.read();
                        hide_new_post_popup.set(!is_hidden);
                    },
                    NewPostPopup { hide: hide_new_post_popup.clone() }
                }
            }
        }
    )
}
