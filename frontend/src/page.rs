pub mod home;
pub mod not_found;
pub mod register;
pub mod login;

pub use home::Home;
pub use not_found::PageNotFound;
pub use register::Register;
pub use login::Login;


use dioxus::prelude::*;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/home")]
    // #[redirect("/account/login", || Route::Home {})]
    Home,

    #[route("/account/register")]
    Register,

    #[route("/account/login")]
    // #[redirect("/account/login", || Route::Home {})]
    Login,

    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}
