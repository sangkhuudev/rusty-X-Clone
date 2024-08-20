mod edit_profile;
mod home;
mod login;
mod new_post;
mod not_found;
mod register;
mod trending;
mod view_profile;

pub use crate::elements::*;
use crate::Init;
use dioxus::prelude::*;
pub use edit_profile::EditProfile;
pub use home::{bookmarked::HomeBookmarked, liked::HomeLiked, Home};
pub use login::Login;
pub use new_post::*;
pub use not_found::PageNotFound;
pub use register::Register;
pub use trending::Trending;
pub use view_profile::ViewProfile;

#[derive(Routable, Clone, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Main)]
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

        #[route("/profile/view/:user_id")]
        ViewProfile {
            user_id: String,
        },
    #[end_layout]
    #[route("/:..route")]
    PageNotFound {
        route: Vec<String>,
    },
}

#[component]
pub fn Main() -> Element {
    rsx!(
        div {
            Init {},
            Sidebar {}
            ToastRoot {}
            Navbar {}
        }
        Outlet::<Route> {}
    )
}
