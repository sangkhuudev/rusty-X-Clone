#![allow(clippy::redundant_closure_call)]
#![allow(clippy::await_holding_refcell_ref)]
#![allow(clippy::drop_non_drop)]
#![allow(non_snake_case)]

pub mod util;
pub mod app;
pub mod page;

use app::App;
use dioxus::prelude::*;
use tracing::Level;


pub const ROOT_API_URL: &str = "http://127.0.0.1:8080/";



fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}



mod prelude {
    pub use crate::page;
    pub use crate::util::{async_handler, sync_handler};
}
