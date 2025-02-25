

#![recursion_limit = "512"]
#![allow(deprecated)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate mazzekey as keylib;
extern crate tempdir;

#[macro_use]
mod config_macro;
pub mod accounts;
pub mod archive;
pub mod common;
pub mod configuration;
pub mod full;
pub mod light;
pub mod rpc;

/// Used in Genesis author to indicate test-net/main-net version.
/// Increased for every test-net/main-net release with reset.
const GENESIS_VERSION: &str = "1949000000000000000000000000000000001001";
