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
pub mod memzone;
pub mod mempool;
pub mod mbuf;
mod eal;

pub use errors::{Error, Result};
pub use eal::*;
pub use ffi::consts::*;
