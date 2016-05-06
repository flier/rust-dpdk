#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate errno;
extern crate cfile;

extern crate rte_sys as ffi;

mod errors;
pub mod common;
pub mod config;
pub mod memzone;
pub mod mempool;
pub mod mbuf;
pub mod ethdev;
pub mod lcore;
pub mod eal;

pub use errors::{Error, Result};
pub use ffi::consts::*;

#[cfg(test)]
mod tests;
