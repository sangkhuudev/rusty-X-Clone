#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::{page::post::content::Content, POSTMANAGER};
use uchat_domain::PostId;
use uchat_endpoint::post::types::PublicPost;
use crate::page::post::actionbar::Actionbar;
use indexmap::IndexMap;


pub mod content;
pub mod actionbar;

#[derive(Default)]
pub struct PostManager {
    pub posts: IndexMap<PostId, PublicPost>
}

impl PostManager {
    pub fn update<F>(&mut self, post_id: PostId, mut update_fn: F) -> bool 
    where
        F: FnMut(&mut PublicPost)
    {
        if let Some(post) = self.posts.get_mut(&post_id) {
            update_fn(post);
            true
        } else {
            false
        }
    }

    pub fn populate<T>(&mut self, posts: T)
    where
        T: Iterator<Item = PublicPost>
    {
        self.posts.clear();
        for post in posts {
            self.posts.insert(post.id, post);
        }
    }

    pub fn clear(&mut self) {
        self.posts.clear()
    }

    pub fn get(&self, post_id: &PostId) -> Option<&PublicPost> {
        self.posts.get(post_id)
    }

    pub fn remove(&mut self, post_id: &PostId) {
        self.posts.shift_remove(post_id);
    }
}   



#[component]
pub fn Header(post: PublicPost) -> Element {
    let (posted_date, posted_time) = {
        let date = post.time_posted.format("%Y-%m-%d");
        let time = post.time_posted.format("%H:%m:%s");
        (date, time)
    };
    
    let display_name = match &post.by_user.dislay_name {
        Some(name) => name.as_ref(),
        None => ""
    };

    let handle = &post.by_user.handle;

    rsx!(
        div {
            class: "flex flex-row justify-between",
            div {
                class: "cursor-pointer",
                onclick: move |_| {},
                div {
                    "{display_name}"
                }
                div {
                    class: "font-light",
                    "{handle}"
                },
            }
            div {
                class: "text-right",
                "{posted_date}",
                "{posted_time}"
            }
        }
    )
}

#[component]
pub fn PublicPostEntry(post_id: PostId) -> Element {
    let post_manager = POSTMANAGER.read();
    let this_post = post_manager.get(&post_id).unwrap();
    // let this_post = POSTMANAGER.signal().read().get(&post_id).unwrap();

    rsx!(
        div {
            key: "{this_post.id.to_string()}",
            class: "grid grid-cols-[50px_1fr] gap-2 mb-4",
            div { /*profile image */},
            div {
                class: "flex flex-col gap-3",
                // header
                Header { post: this_post.clone()},
                // reply to
                // content
                Content{ post: this_post.clone()},
                // action bar
                Actionbar {
                    post_id: this_post.id
                }
                hr {}
            }
        }
    )
}