#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/auto_enums_derive/0.5.10")]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::use_self)]

extern crate proc_macro;

#[macro_use]
mod utils;

mod derive;
mod enum_derive;

use proc_macro::TokenStream;

/// An attribute macro like a wrapper of `#[derive]`, implementing
/// the supported traits and passing unsupported traits to `#[derive]`.
#[proc_macro_attribute]
pub fn enum_derive(args: TokenStream, input: TokenStream) -> TokenStream {
    TokenStream::from(self::enum_derive::attribute(args.into(), input.into()))
}
