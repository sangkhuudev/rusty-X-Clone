#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

//-------------------------------------------------------------------------------------------

#[derive(Props, PartialEq, Clone)]
pub struct AppbarImgButtonProps {
    append_class: Option<String>,
    click_handler: Option<EventHandler<MouseEvent>>,
    disabled: Option<bool>,
    img: String,
    lable: String,
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
                    callback(ev);
                }
            },
            title: props.title,
            img {
                class: "h-6 w-6",
                src: "{props.img}"
            },
            span {
                class: "text-sm",
                "{props.lable}"
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
    rsx!(
        div {
            class: "max-w-[var(--content-max-width)] h-[var(--appbar-height)]
                    fixed top-0 right-0 mx-auto z-50 bg-slate-200",
            div {
                class: "flex flex-row items-center w-full h-full pr-5 gap-8",
                div {
                    class: "cursor-pointer",
                    onclick: move |_| {},
                    img {
                        class: "profile-portrait",
                        src: ""
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
