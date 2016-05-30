#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate nix;

#[macro_use]
extern crate rte;

use std::env;

use nix::sys::signal;

use rte::*;

// Main function, does initialisation and calls the per-lcore functions
fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    // init EAL
    eal::init(&args).expect("Cannot init EAL");
}
