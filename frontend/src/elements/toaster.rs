#![allow(non_snake_case)]

use crate::TOASTER;
use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use std::collections::HashMap;

//----------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind {
    Error,
    Info,
    Success,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub message: String,
    pub expires: DateTime<Utc>,
    pub kind: ToastKind,
}

//----------------------------------------------------------------------------------

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Toaster {
    toasts: HashMap<usize, Toast>,
    next_id: usize,
}

impl Toaster {
    fn increment_id(&mut self) {
        self.next_id += 1;
    }

    pub fn push(&mut self, toast: Toast) {
        self.toasts.insert(self.next_id, toast);
        self.increment_id();
    }

    pub fn remove(&mut self, id: usize) {
        self.toasts.remove(&id);
    }

    pub fn success<T: Into<String>>(&mut self, message: T, duration: Duration) {
        let toast = Toast {
            message: message.into(),
            expires: Utc::now() + duration,
            kind: ToastKind::Success,
        };
        self.push(toast);
    }

    pub fn info<T: Into<String>>(&mut self, message: T, duration: Duration) {
        let toast = Toast {
            message: message.into(),
            expires: Utc::now() + duration,
            kind: ToastKind::Info,
        };
        self.push(toast);
    }

    pub fn error<T: Into<String>>(&mut self, message: T, duration: Duration) {
        let toast = Toast {
            message: message.into(),
            expires: Utc::now() + duration,
            kind: ToastKind::Error,
        };
        self.push(toast);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, usize, Toast> {
        self.toasts.iter()
    }
}

//----------------------------------------------------------------------------------

#[component]
pub fn ToastRoot() -> Element {
    let toasters = TOASTER.read();
    let ToastElements = toasters.iter().map(|(&id, toast)| {
        let toast_style = match toast.kind {
            ToastKind::Info => "bg-slate-200 border-slate-300",
            ToastKind::Error => "bg-rose-300 border-rose-400",
            ToastKind::Success => "bg-emerald-200 border-emerald-300",
        };

        rsx! {
            div {
                key: "{id}",
                class: "{toast_style} p-3 border border-solid rounded cursor-pointer",
                onclick: move |_| {
                    TOASTER.write().remove(id);
                },
                "{toast.message}"
            }
        }
    });

    let total_toasts = TOASTER.signal();

    // use_future will run the future
    let _remove_ids = use_resource(move || async move {
        loop {
            if total_toasts.read().toasts.len() == 0 {
                break;
            }

            let expired_ids = TOASTER
                .read()
                .iter()
                .filter_map(|(&id, toast)| {
                    if Utc::now() > toast.expires {
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<usize>>();
            info!("The loop will be break after removing toasts ");

            expired_ids
                .iter()
                .for_each(|&id| TOASTER.write().remove(id));

            if total_toasts.read().toasts.len() == 0 {
                break;
            }

            gloo_timers::future::TimeoutFuture::new(200_u32).await
        }
    });

    rsx! {
        div {
            class: "fixed bottom-[var(--navbar-height)]
                w-screen max-w-[var(--content-max-width)]",
            div {
                class: "flex flex-col px-5 mb-5 gap-5",
                {ToastElements}
            }
        }
    }
}
