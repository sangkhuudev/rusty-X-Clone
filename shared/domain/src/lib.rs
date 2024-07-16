#[cfg(feature = "query")]
#[macro_use]
extern crate diesel_derive_newtype;

pub mod id;
pub mod user;

pub use user::{Password, Username};
pub use id::*;