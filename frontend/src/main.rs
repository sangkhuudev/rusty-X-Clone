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
use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use page::{PostManager, Toaster};
use util::ApiClient;

pub const ROOT_API_URL: &str = "http://127.0.0.1:8000/";

pub static TOASTER: GlobalSignal<Toaster> = Signal::global(|| Toaster::default());
pub static POSTMANAGER: GlobalSignal<PostManager> = Signal::global(|| PostManager::default());

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");

    info!("Logger initialized!");

    ApiClient::init();
    launch(App);
}

mod prelude {
    // pub use crate::page;
    pub use crate::util::{async_handler, maybe_class, sync_handler};
}
