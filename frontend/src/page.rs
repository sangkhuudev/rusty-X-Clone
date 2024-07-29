pub mod home;
pub mod not_found;
pub mod register;
pub mod login;
pub mod new_post;
pub mod trending;

pub use home::Home;
pub use not_found::PageNotFound;
pub use register::Register;
pub use login::Login;
pub use trending::Trending;
pub use new_post::*;
pub use crate::elements::*;


use dioxus::prelude::*;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
        #[route("/home")]
        Home {},

        #[route("/account/register")]
        Register {},

        #[route("/account/login")]
        Login {},

        #[route("/post/new_chat")]
        NewChat {},

        #[route("/posts/trending")]
        Trending {},

    #[end_layout]
    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}
