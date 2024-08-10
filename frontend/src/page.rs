pub mod edit_profile;
pub mod home;
pub mod login;
pub mod new_post;
pub mod not_found;
pub mod register;
pub mod trending;

pub use crate::elements::*;
pub use edit_profile::EditProfile;
pub use home::{bookmarked::HomeBookmarked, liked::HomeLiked, Home};
pub use login::Login;
pub use new_post::*;
pub use not_found::PageNotFound;
pub use register::Register;
pub use trending::Trending;

use dioxus::prelude::*;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
        #[route("/home")]
        Home {},

        #[route("/home/liked")]
        HomeLiked {},

        #[route("/home/bookmarked")]
        HomeBookmarked {},

        #[route("/account/register")]
        Register {},

        #[route("/account/login")]
        Login {},

        #[route("/post/new_chat")]
        NewChat {},

        #[route("/post/new_image")]
        NewImage {},

        #[route("/post/new_poll")]
        NewPoll {},

        #[route("/posts/trending")]
        Trending {},

        #[route("/profile/edit")]
        EditProfile {},
    #[end_layout]
    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}
