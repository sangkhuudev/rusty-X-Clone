#![allow(clippy::redundant_closure_call)]
#![allow(clippy::await_holding_refcell_ref)]
#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]

pub mod app;
pub mod elements;
pub mod page;
pub mod util;
pub mod icon;

use app::App;
use dioxus::prelude::*;
use util::ApiClient;
use dioxus_logger::tracing::{Level, info};


pub const ROOT_API_URL: &str = "http://127.0.0.1:8000/";

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");

    info!("Logger initialized!");

    ApiClient::init();
    launch(App);
}

mod prelude {
    // pub use crate::page;
    pub use crate::util::{async_handler, sync_handler, maybe_class};
}
