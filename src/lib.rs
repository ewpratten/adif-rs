//! `adif` is an [Amateur Data Interchange Format](http://adif.org/) library for Rust.

pub mod data;
pub mod parser;

pub use data::{AdifFile, AdifHeader, AdifRecord, AdifType};
pub use parser::parse_adif;
