#![allow(clippy::redundant_closure_call)]
#![allow(clippy::await_holding_refcell_ref)]
#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]

pub mod app;
pub mod elements;
pub mod page;
pub mod util;

use app::App;
use dioxus::prelude::*;
use util::ApiClient;
// use cfg_if::cfg_if;
// use dioxus_logger::{Trace, Level};
use dioxus_logger::tracing::{Level, info};


pub const ROOT_API_URL: &str = "http://127.0.0.1:8000/";

// cfg_if! {
//     if #[cfg(feature = "console_log")] {
//         fn init_log() {
//             use log::Level;
//             console_log::init_with_level(Level::Trace).expect("error initializing log");
//         }
//     } else {
//         fn init_log() {}
//     }
// }

fn main() {
    // Init logger
    // wasm_logger::init(wasm_logger::Config::default());
    // 
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");

    info!("Logger initialized!");

    ApiClient::init();
    launch(App);
}

mod prelude {
    // pub use crate::page;
    pub use crate::util::{async_handler, sync_handler};
}
