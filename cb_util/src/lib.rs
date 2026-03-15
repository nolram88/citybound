// TODO: remove once https://github.com/rust-lang/rust/issues/54726 is resolved
#![feature(custom_inner_attributes)]
extern crate rand;
extern crate fnv;
extern crate uuid;
extern crate arrayvec;
extern crate kay;
extern crate cb_time;
#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;
extern crate serde;

pub extern crate compact;
#[macro_use]
extern crate compact_macros;
#[macro_use]
extern crate serde_derive;

pub mod async_counter;
pub mod random;
pub mod config_manager;
pub mod log;
