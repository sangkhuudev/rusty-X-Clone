// #![allow(non_snake_case)]

// use crate::prelude::*;
// use chrono::Duration;
// use dioxus::prelude::*;
// use dioxus_logger::tracing;
// use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

// pub fn Trending() -> Element {
//     let api_client = ApiClient::global();

//     // Initialize tracing
//     tracing::info!("Initializing Trending component.");

//     // Fetch trending posts asynchronously
//     // let _fetch_posts = use_resource(move || async move {
//     //     tracing::info!("Starting request to fetch trending posts.");

//     //     // Define a timeout duration and start fetching data
//     //     match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
//     //         Ok(data) => {
//     //             tracing::info!("Successfully retrieved trending posts.");
//     //             POSTMANAGER.write().populate(data.posts.into_iter());

//     //             TOASTER
//     //                 .write()
//     //                 .info("Retrieving trending posts", Duration::milliseconds(600));
//     //         }
//     //         Err(err) => {
//     //             tracing::error!("Failed to fetch trending posts: {:?}", err);
//     //             TOASTER.write().error(
//     //                 format!("Failed to retrieve posts : {err}"),
//     //                 Duration::milliseconds(600),
//     //             );
//     //         }
//     //     }
//     // });

//     let fetch_posts = use_resource(move || async move {
//         tracing::info!("Starting request to fetch trending posts.");

//         // Define a timeout duration and start fetching data
//         match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
//             Ok(data) => {
//                 tracing::info!("Successfully retrieved trending posts.");
//                 POSTMANAGER.write().populate(data.posts.clone().into_iter());

//                 TOASTER
//                     .write()
//                     .info("Retrieving trending posts", Duration::milliseconds(600));

//                 Ok(data)
//             }
//             Err(err) => {
//                 tracing::error!("Failed to fetch trending posts: {:?}", err);
//                 TOASTER.write().error(
//                     format!("Failed to retrieve posts : {err}"),
//                     Duration::milliseconds(600),
//                 );
//                 Err(err)
//             }
//         }
//     });

//     match fetch_posts() {
//         Some(Ok(_data)) => {
//             let post_manager = POSTMANAGER.read();
//             let trending_posts = post_manager.all_to_public();

//             rsx!({ trending_posts.into_iter() })
//         },
//         Some(Err(_)) => {
//             rsx!( div { "Something wrong"})
//         }
//         None => rsx!( div { "Loading posts"})
//     }
//     // let post_manager = POSTMANAGER.read();
//     // let trending_posts = post_manager.all_to_public();

//     // rsx!({ trending_posts.into_iter() })
// }

#![allow(non_snake_case)]

use crate::prelude::*;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use uchat_endpoint::post::endpoint::{TrendingPost, TrendingPostOk};

pub fn Trending() -> Element {
    let api_client = ApiClient::global();

    // Fetch trending posts asynchronously using use_resource
    let _fetch_posts = use_resource(move || async move {
        tracing::info!("Starting request to fetch trending posts.");
        POSTMANAGER.write().clear();
        // Define a timeout duration and start fetching data
        match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
            Ok(res) => {
                POSTMANAGER.write().populate(res.posts.clone().into_iter());
                TOASTER
                    .write()
                    .info("Retrieving trending posts", Duration::milliseconds(1200));
            }
            Err(err) => {
                tracing::error!("Failed to fetch trending posts: {:?}", err);
                TOASTER.write().error(
                    format!("Failed to retrieve posts : {err}"),
                    Duration::milliseconds(1000),
                );
            }
        }
    });
    let post_manager = POSTMANAGER.read();
    let trending_posts = post_manager.all_to_public();

    rsx!(
        Appbar {
            title: "Trending posts",
            AppbarImgButton {
                click_handler: move |_| {
                    navigator().go_back();
                },
                img: ICON_BACK,
                label: "Back",
                title: "Go to the previous page",
            }
        }
        {trending_posts.into_iter()}
    )
    // // Render the UI based on the state of the resource
    // match *fetch_posts.state().read() {
    //     UseResourceState::Pending => rsx!(div { "Loading posts..." }),
    //     UseResourceState::Ready => {
    //         let post_manager = POSTMANAGER.read();
    //         let trending_posts = post_manager.all_to_public();

    //         rsx!({ trending_posts.into_iter() })
    //     }
    //     // UseResourceState::Failed => rsx!(div { "Failed to load posts." }),
    //     _ => rsx!(div { "Unknown state." }),
    // }
}

// #![allow(non_snake_case)]

// use crate::prelude::*;
// use chrono::Duration;
// use dioxus::prelude::*;
// use dioxus_logger::tracing;
// use uchat_endpoint::post::{endpoint::{TrendingPost, TrendingPostOk}, types::PublicPost};

// #[component]
// pub fn Trending() -> Element {
//     let api_client = ApiClient::global();
//     let mut trending_posts = use_signal(Vec::new);
//     let mut loading = use_signal(|| true);
//     let mut error = use_signal(|| None::<String>);

//     // Fetch trending posts asynchronously
//     use_effect(
//         move || {
//             // let api_client = api_client.clone();
//             // let trending_posts = trending_posts.clone();
//             // let loading = loading.clone();
//             // let error = error.clone();

//             spawn(async move {
//                 tracing::info!("Starting request to fetch trending posts.");
//                 *loading.write() = true;

//                 match fetch_trending_posts(&api_client).await {
//                     Ok(posts) => {
//                         *trending_posts.write() = posts;
//                         TOASTER
//                             .write()
//                             .info("Trending posts retrieved", Duration::milliseconds(1000));
//                     }
//                     Err(err) => {
//                         let err_msg = format!("Failed to retrieve trending posts: {err}");
//                         tracing::error!("{err_msg}");
//                         *error.write() = Some(err_msg.clone());
//                         TOASTER.write().error(err_msg, Duration::milliseconds(1000));
//                     }
//                 }

//                 *loading.write() = false;
//             });

//         }
//     );

//     let post_manager = POSTMANAGER.read();

//     // Render the UI based on loading and error state
//     if *loading.read() {
//         rsx!(div { "Loading posts..." })
//     } else if let Some(error) = &*error.read() {
//         rsx!(div { "Error: {error}" })
//     } else {
//         let posts = post_manager.all_to_public();
//         rsx!({posts.into_iter()})
//     }
// }

// async fn fetch_trending_posts(api_client: &ApiClient) -> Result<Vec<PublicPost>, String> {
//     match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
//         Ok(res) => Ok(res.posts),
//         Err(err) => Err(format!("{:?}", err)),
//     }
// }
