#![feature(const_trait_impl)]
#![feature(decl_macro)]
#![feature(let_chains)]

pub mod version;
pub mod util;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");