#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use std::str::FromStr;
use uchat_domain::UserId;
use uchat_endpoint::user::{
    endpoint::{FollowUser, FollowUserOk, ViewProfile, ViewProfileOk},
    types::{FollowAction, PublicUserProfile},
};

#[component]
pub fn ViewProfile(user_id: ReadOnlySignal<String>) -> Element {
    let api_client = ApiClient::global();
    let mut profile: Signal<Option<PublicUserProfile>> = use_signal(|| None);
    let follow_onclick = async_handler!([api_client], move |_| async move {
        let am_following = match profile.read().as_ref() {
            Some(profile) => profile.am_following,
            None => false,
        };

        let request_data = FollowUser {
            user_id: UserId::from_str(&user_id.read()).unwrap(),
            action: match am_following {
                true => FollowAction::Unfollow,
                false => FollowAction::Follow,
            },
        };
        match fetch_json!(<FollowUserOk>, api_client, request_data) {
            Ok(res) => {
                profile.with_mut(|profile| {
                    profile.as_mut().map(|p| p.am_following = res.status.into())
                });
            }
            Err(e) => TOASTER.write().error(
                format!("Failed to update follow status : {e}"),
                Duration::milliseconds(600),
            ),
        }
    });

    // Fetch and populate profile and posts data
    let _ = use_resource(move || async move {
        tracing::info!("Starting fetch for profile: {}", user_id);
        fetch_and_populate_profile(user_id, &api_client, &mut profile).await;
    });

    let ProfileSection = {
        match profile.with(|profile| profile.clone()) {
            Some(profile) => {
                let display_name = profile
                    .display_name
                    .map(|name| name.into_inner())
                    .unwrap_or_else(|| "(None)".to_string());
                let profile_image = profile
                    .profile_image
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| "".to_string());

                let follow_button_text = match profile.am_following {
                    true => "Unfollow",
                    false => "Follow",
                };

                rsx! {
                    div {
                        class: "flex flex-col gap-3",
                        div {
                            class: "flex flex-row justify-center",
                            img {
                                class: "profile-portrait-lg",
                                src: "{profile_image}",
                            }
                        },
                        div { "Handle: {profile.handle}" },
                        div { "Name: {display_name} "},
                        button {
                            class: "btn",
                            onclick: follow_onclick,
                            "{follow_button_text}"
                        }
                    }
                }
            }
            None => rsx! { "Loading profile..." },
        }
    };

    let post_manager = POSTMANAGER.read();
    let Posts = post_manager.all_to_public();

    rsx!(
        Appbar {
            title: "View profile",
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().go_back();
                },
                img: ICON_BACK,
                label: "Back",
                title: "Go to the previous page",
            }
        }
        {ProfileSection}
        div {
            class: "font-bold text-center my-6",
            "Posts"
        }
        hr {
            class: "h-px my-6 bg-gray-200 border-0"
        }
        {Posts.into_iter()}
    )
}

async fn fetch_and_populate_profile(
    user_id: ReadOnlySignal<String>,
    api_client: &ApiClient,
    profile: &mut Signal<Option<PublicUserProfile>>,
) {
    POSTMANAGER.write().clear();
    let request_data = ViewProfile {
        for_user: UserId::from_str(&user_id.read()).unwrap(),
    };

    let response = fetch_json!(<ViewProfileOk>, api_client, request_data);

    match response {
        Ok(res) => {
            profile.with_mut(|profile| *profile = Some(res.profile));
            POSTMANAGER.write().populate(res.posts.into_iter());

            TOASTER
                .write()
                .info("Retrieving profile", Duration::milliseconds(600));
        }
        Err(err) => {
            tracing::error!("Failed to fetch profiles: {:?}", err);
            TOASTER.write().error(
                format!("Failed to retrieve posts : {err}"),
                Duration::milliseconds(600),
            );
        }
    }
}
