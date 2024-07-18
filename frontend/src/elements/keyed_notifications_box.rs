use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct KeyedNotifications {
    pub inner: HashMap<String, String>,
}

impl KeyedNotifications {
    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.insert(key.into(), value.into());
    }

    pub fn remove<K: AsRef<str> + ?Sized>(&mut self, key: &K) {
        self.inner.remove(key.as_ref());
    }

    pub fn message(&self) -> std::collections::hash_map::Values<'_, String, String> {
        self.inner.values()
    }
    pub fn has_message(&self) -> bool {
        !self.inner.is_empty()
    }
}

#[derive(Clone, PartialEq, Props)]
pub struct KeyedNotificationsProps {
    legend: Option<&'static str>,
    notification: KeyedNotifications,
}

pub fn KeyedNotificationsBox(props: KeyedNotificationsProps) -> Element {
    let notifications = props.notification.message().map(|msg| {
        rsx! {
            li { "{msg}" }
        }
    });

    let legend = props.legend.unwrap_or("Error");

    match props.notification.has_message() {
        true => {
            rsx! {
                fieldset { class: "fieldset border-red-300 rounded",
                    legend { class: "border-red-300 px-4", "{legend}" }
                    ul { class: "list-disc ml-4", {notifications} }
                }
            }
        }

        false => None,
    }
}
