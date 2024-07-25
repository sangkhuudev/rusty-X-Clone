
#![allow(non_snake_case)]

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;

use crate::TOASTER;


//----------------------------------------------------------------------------------

#[derive(Clone)]
pub enum ToastKind {
    Error,
    Info,
    Success
}

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub expires: DateTime<Utc>,
    pub kind: ToastKind,
}


//----------------------------------------------------------------------------------

#[derive(Default, Clone)]
pub struct Toaster {
    toasts: HashMap<usize, Toast>,
    next_id: usize
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
            kind: ToastKind::Success
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
            kind: ToastKind::Error
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
    let ToastElements = toasters
        .iter()
        .map(|(&id, toast)| {
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
    
    rsx!{
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