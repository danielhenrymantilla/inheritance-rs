#![cfg_attr(feature = "external_doc",
    feature(external_doc),
    doc(include = "../README.md"),
)]

pub use proc_macro::*; #[cfg(any())]
mod proc_macro;
