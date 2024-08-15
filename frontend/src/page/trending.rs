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
        // gloo_timers::future::TimeoutFuture::new(300).await;
        // Define a timeout duration and start fetching data
        match fetch_json!(<TrendingPostOk>, api_client, TrendingPost) {
            Ok(res) => {
                POSTMANAGER.write().populate(res.posts.clone().into_iter());
                TOASTER
                    .write()
                    .info("Retrieving trending posts", Duration::milliseconds(1000));
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

    rsx!({ trending_posts.into_iter() })
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
