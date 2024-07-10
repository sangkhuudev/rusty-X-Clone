pub mod home;
pub mod not_found;
pub mod register;

pub use home::{Blog, Home};
pub use not_found::PageNotFound;
pub use register::Register;
// pub use route::*;

use dioxus::prelude::*;

// pub mod route {
//     pub const ACCOUNT_REGISTER: &str = "/account/register";
// }

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/account/register")]
    Register,
    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}
