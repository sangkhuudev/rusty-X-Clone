#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{error, info};
use std::str::FromStr;
use uchat_domain::UserId;
use uchat_endpoint::user::{
    endpoint::{ViewProfile, ViewProfileOk},
    types::PublicUserProfile,
};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;

#[wasm_bindgen(inline_js = "
    export function now() {
        return performance.now();
    }
")]
extern "C" {
    fn now() -> f64;
}

#[component]
pub fn ViewProfile(user_id: ReadOnlySignal<String>) -> Element {
    let api_client = ApiClient::global();
    let mut profile = use_signal(|| None);

    // Fetch and populate profile and posts data
    let _ = use_resource(move || async move {
        let start_time = now();
        tracing::info!("Starting fetch for profile: {}", user_id.read());

        fetch_and_populate_profile(&user_id, &api_client, &mut profile).await;

        let elapsed_time = now() - start_time;
        tracing::info!(
            "Completed fetch for profile: {} in {} ms",
            user_id.read(),
            elapsed_time
        );
    });

    let ProfileSection = {
        match profile.with(|profile| profile.clone()) {
            Some(profile) => render_profile_section(profile),
            None => rsx!(div { "Loading profile..." }),
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
    user_id: &ReadOnlySignal<String>,
    api_client: &ApiClient,
    profile: &mut Signal<Option<PublicUserProfile>>,
) {
    POSTMANAGER.write().clear();
    let request_data = ViewProfile {
        for_user: UserId::from_str(&user_id.read()).unwrap(),
    };

    let start_time = now();
    let response = fetch_json!(<ViewProfileOk>, api_client, request_data);

    match response {
        Ok(res) => {
            profile.with_mut(|profile| *profile = Some(res.profile));
            POSTMANAGER.write().populate(res.posts.into_iter());

            TOASTER
                .write()
                .info("Retrieving profile", Duration::milliseconds(600));

            tracing::info!(
                "Profile and posts fetched successfully in {} ms",
                now() - start_time
            );
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

fn render_profile_section(profile: PublicUserProfile) -> Element {
    let display_name = profile
        .display_name
        .map(|name| name.into_inner())
        .unwrap_or_else(|| "(None)".to_string());

    let profile_image = profile
        .profile_image
        .map(|url| url.to_string())
        .unwrap_or_else(|| "".to_string());

    let follow_button_text = if profile.am_following {
        "Unfollow"
    } else {
        "Follow"
    };

    rsx!(
        div {
            class: "flex flex-col gap-3",
            div {
                class: "flex flex-row justify-center",
                img {
                    class: "profile-portrait-lg",
                    src: "{profile_image}"
                }
            },
            div { "Handle: {profile.handle}" }
            div { "Name: {display_name}" }
            button {
                class: "btn",
                onclick: move |_| {},
                "{follow_button_text}"
            }
        }
    )
}
