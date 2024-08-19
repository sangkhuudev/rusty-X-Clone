#![allow(non_snake_case)]

use crate::prelude::post::actionbar::Actionbar;
use crate::prelude::post::content::Content;
use crate::prelude::Route::ViewProfile;
use crate::prelude::*;
use dioxus::prelude::*;
use indexmap::IndexMap;
use uchat_domain::PostId;
use uchat_endpoint::post::types::PublicPost;

mod actionbar;
mod content;
mod quick_response;

#[derive(Default, Clone)]
pub struct PostManager {
    pub posts: IndexMap<PostId, PublicPost>,
}

impl PostManager {
    pub fn update<F>(&mut self, post_id: PostId, mut update_fn: F) -> bool
    where
        F: FnMut(&mut PublicPost),
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
        T: Iterator<Item = PublicPost>,
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

    pub fn all_to_public(&self) -> Vec<Option<VNode>> {
        self.posts
            .iter()
            .map(|(&id, _)| {
                rsx!(
                    div {
                        PublicPostEntry {
                            post_id: id
                        }
                    }
                )
            })
            .collect()
    }
}
#[component]
pub fn ProfileImage(post: PublicPost) -> Element {
    let poster_info = &post.by_user;

    let profile_img_src = &poster_info
        .profile_image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or_else(|| "");

    rsx!(
        div {
            img {
                class: "profile-portrait cursor-pointer",
                onclick: move |_| {
                    navigator().push(ViewProfile { user_id: post.by_user.id.to_string()});
                },
                src: "{profile_img_src}",
            }
        }
    )
}

#[component]
pub fn Header(post: PublicPost) -> Element {
    let (posted_date, posted_time) = {
        let date = post.time_posted.format("%Y-%m-%d");
        let time = post.time_posted.format("%H:%m:%s");
        (date, time)
    };

    let display_name = match &post.by_user.display_name {
        Some(name) => name.as_ref(),
        None => "",
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
    let this_post = match post_manager.get(&post_id) {
        Some(post) => post,
        None => {
            return rsx!(div { "Post not found" });
        }
    };

    rsx!(
        div {
            key: "{this_post.id.to_string()}",
            class: "grid grid-cols-[50px_1fr] gap-2 mb-4",
            ProfileImage { post: this_post.clone()}
            div {
                class: "flex flex-col gap-3",
                // header
                Header { post: this_post.clone()},
                // reply to
                // content
                Content { post: this_post.clone()},
                // action bar
                Actionbar {
                    post_id: this_post.id
                }
                hr {}
            }
        }
    )
}
