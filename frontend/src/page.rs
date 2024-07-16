pub mod home;
pub mod not_found;
pub mod register;
pub mod login;

pub use home::{Blog, Home};
pub use not_found::PageNotFound;
pub use register::Register;
pub use login::Login;
// pub use route::*;

use dioxus::prelude::*;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/blog/:id")]
    Blog { id: i32 },

    #[route("/account/register")]
    Register,

    #[route("/account/login")]
    Login,

    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}
