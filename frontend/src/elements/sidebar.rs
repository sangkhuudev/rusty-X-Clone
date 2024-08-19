#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Utc;
use dioxus::prelude::*;
use uchat_domain::SessionId;

#[derive(Default)]
pub struct SidebarManager {
    pub is_open: bool,
}

impl SidebarManager {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

#[component]
pub fn Sidebar() -> Element {
    let sidebar_width = if SIDEBAR.read().is_open() {
        "w-[var(--sidebar-width)]"
    } else {
        "w-0"
    };

    let overlay_class = if SIDEBAR.read().is_open() {
        "w-full opacity-80"
    } else {
        "w-0 opacity-0"
    };

    let Overlay = rsx! {
        div {
            class: "fixed top-0 left-0 h-full navbar-bg-color transition z-[99] {overlay_class}",
            onclick: move |_| SIDEBAR.write().close(),
        }
    };
    let navigator = use_navigator();
    let read_local_profile = LOCAL_PROFILE.read();
    let profile_img_src = read_local_profile
        .image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or_else(|| "");

    rsx!(
        { Overlay },
        div {
            class: "{sidebar_width} z-[100] fixed top-0 left-0 h-full
            overflow-x-hidden
            flex flex-col
            navbar-bg-color transition-[width] duration-300",

            a {
                class: "flex flex-row justify-center py-5 cursor-pointer",
                onclick: move |_| {
                    SIDEBAR.write().close();
                    if let Some(id) = LOCAL_PROFILE.read().user_id {
                        navigator.push(Route::ViewProfile { user_id: id.to_string() });
                    }
                },
                img {
                    class: "profile-portrait-lg",
                    src: "{profile_img_src}"
                }
            }
            a {
                class: "sidebar-navlink border-t",
                onclick: move |_| {
                    SIDEBAR.write().close();
                    navigator.push(Route::EditProfile {});
                },
                "Edit Profile"
            }
            a {
                class: "sidebar-navlink",
                onclick: move |_| {
                    SIDEBAR.write().close();
                    navigator.push(Route::HomeBookmarked {});
                },
                "Bookmarks"
            }
            a {
                class: "sidebar-navlink mb-auto",
                onclick: move |_| {
                    SIDEBAR.write().close();
                    navigator.push(Route::HomeLiked {});
                },
                "Liked"
            }
            a {
                class: "sidebar-navlink",
                onclick: move |_| {
                    crate::util::cookie::set_session("".to_string(), SessionId::new(), Utc::now());
                    SIDEBAR.write().close();
                    LOCAL_PROFILE.write().user_id = None;
                    LOCAL_PROFILE.write().image = None;
                    navigator.replace(Route::Login {});
                },
                "Logout"
            }

        }
    )
}
