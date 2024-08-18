#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

pub const BUTTON_SELECTED: &str = "border-slate-600";
//-------------------------------------------------------------------------------------------

#[derive(Props, PartialEq, Clone)]
pub struct AppbarImgButtonProps {
    append_class: Option<String>,
    click_handler: Option<EventHandler<MouseEvent>>,
    disabled: Option<bool>,
    img: String,
    label: String,
    title: Option<String>,
}

pub fn AppbarImgButton(props: AppbarImgButtonProps) -> Element {
    let append_class = props.append_class.unwrap_or(String::new());
    rsx!(
        button {
            class: "flex flex-col h-14 w-10 justify-end items-center
            border-slate-200 border-b-4 {append_class}",
            disabled: props.disabled.unwrap_or_default(),
            onclick: move |ev| {
                if let Some(callback) = props.click_handler {
                    callback.call(ev)
                }
            },
            title: props.title,
            img {
                class: "h-6 w-6",
                src: "{props.img}"
            },
            span {
                class: "text-sm",
                "{props.label}"
            }
        }
    )
}
//-------------------------------------------------------------------------------------------
#[derive(Props, PartialEq, Clone)]
pub struct AppbarProps {
    pub title: String,
    pub children: Element,
}

pub fn Appbar(props: AppbarProps) -> Element {
    let local_profile = LOCAL_PROFILE.read();
    let profile_image_src = local_profile
        .image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or_else(|| "");

    rsx!(
        div {
            class: "max-w-[var(--content-max-width)] h-[var(--appbar-height)]
                    fixed top-0 right-0 left-0 mx-auto z-50",
            div {
                class: "flex flex-row items-center w-full h-full pr-5 gap-8",
                div {
                    class: "cursor-pointer",
                    onclick: move |_| {
                        SIDEBAR.write().open();
                    },
                    img {
                        class: "profile-portrait",
                        src: "{profile_image_src}"
                    }
                }
                div {
                    class: "text-xl mr-auto fold-bold",
                    {props.title}
                }
                {props.children}
            }
        }
    )
}
