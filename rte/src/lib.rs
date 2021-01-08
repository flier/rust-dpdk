#![allow(
    deprecated,
    unused,
    clippy::useless_attribute,
    clippy::not_unsafe_ptr_arg_deref,
    clippy::trivially_copy_pass_by_ref,
    clippy::many_single_char_names
)]

extern crate anyhow;
#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate cfile;
extern crate errno;
extern crate itertools;
extern crate libc;
extern crate rand;
extern crate time;
#[macro_use]
extern crate num_derive;
extern crate num_traits;

extern crate rte_sys;

pub mod ffi;

#[macro_use]
pub mod errors;
#[macro_use]
pub mod macros;
#[macro_use]
mod common;
#[macro_use]
pub mod utils;

pub mod mbuf;
pub mod mempool;
pub mod ring;

pub mod bond;
pub mod ethdev;
pub mod kni;
pub mod pci;

pub mod arp;
pub mod ether;
pub mod ip;

#[macro_use]
pub mod cmdline;

pub use self::common::*;
pub use self::errors::{ErrorKind, RteError};
pub use self::ethdev::PortId;
pub use self::ethdev::QueueId;

#[cfg(test)]
mod tests;
