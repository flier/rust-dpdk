extern crate rte;

use std::env;
use std::os::raw::c_void;

use rte::*;

extern "C" fn lcore_hello(_: Option<&c_void>) -> i32 {
    println!("hello from core {}", lcore::id().unwrap());

    0
}

fn main() {
    let args: Vec<String> = env::args().collect();

    eal::init(&args).expect("Cannot init EAL");

    // call lcore_hello() on every slave lcore
    lcore::foreach_slave(|lcore_id| {
        launch::remote_launch(lcore_hello, None, lcore_id).expect("Cannot launch task");
    });

    // call it on master lcore too
    lcore_hello(None);

    launch::mp_wait_lcore();
}
