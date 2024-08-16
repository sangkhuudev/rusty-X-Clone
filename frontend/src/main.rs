#![allow(clippy::redundant_closure_call)]
#![allow(clippy::await_holding_refcell_ref)]
#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]

pub mod app;
pub mod elements;
pub mod icon;
pub mod page;
pub mod util;

use app::App;
use chrono::Duration;
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use page::{local_profile::LocalProfile, PostManager, Route, Toaster};
use uchat_endpoint::user::endpoint::{GetMyProfile, GetMyProfileOk};
use util::ApiClient;

pub const ROOT_API_URL: &str = uchat_endpoint::app_url::API_URL;

pub static TOASTER: GlobalSignal<Toaster> = Signal::global(|| Toaster::default());
pub static POSTMANAGER: GlobalSignal<PostManager> = Signal::global(|| PostManager::default());
pub static LOCAL_PROFILE: GlobalSignal<LocalProfile> = Signal::global(|| LocalProfile::default());

pub fn Init() -> Element {
    let api_client = ApiClient::global();
    let _fetch_local_profile = use_resource(move || async move {
        tracing::info!("Starting request to fetch profiles");

        // Define a timeout duration and start fetching data
        match fetch_json!(<GetMyProfileOk>, api_client, GetMyProfile) {
            Ok(data) => {
                tracing::info!("Successfully retrieved trending posts.");
                LOCAL_PROFILE.write().image = data.profile_image;
                LOCAL_PROFILE.write().user_id = Some(data.user_id);
            }
            Err(err) => {
                TOASTER.write().error(
                    format!("Please login or create an account to continue: {err}"),
                    Duration::milliseconds(600),
                );
                router().push(Route::Login {});
            }
        }
    });

    None
}

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");

    info!("Logger initialized!");

    ApiClient::init();
    launch(App);
}

mod prelude {
    pub use crate::elements::appbar::{self, Appbar, AppbarImgButton};
    // pub use crate::elements::post::PublicPostEntry;
    pub use crate::icon::*;
    pub use crate::page::*;
    pub use crate::util::api_client::fetch_json;
    pub use crate::util::{async_handler, maybe_class, sync_handler, ApiClient};
    pub use crate::{LOCAL_PROFILE, POSTMANAGER, TOASTER};
}
