# rust-dpdk

Rust-Dpdk is an experimental prototype to wrap [DPDK](http://dpdk.org/) API with Rust language.

## Build

First, please follow [the official document](http://dpdk.org/doc/guides/linux_gsg/build_dpdk.html) to setup a DPDK development envrionment.

```
$ CONFIG_RTE_BUILD_COMBINE_LIBS=y EXTRA_CFLAGS="-fPIC -O0 -g -ggdb" make install T=x86_64-native-linuxapp-gcc -j 4
```

And build `rust-dpdk` with `RTE_SDK` envrionment variable:

```
$ RTE_SDK=<rte_path> cargo build
```

## Examples

```rust
extern crate rte;

use std::env;
use std::ptr;
use std::os::raw::c_void;

use rte::*;

extern "C" fn lcore_hello(_: *const c_void) -> i32 {
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
    lcore_hello(ptr::null());

    launch::mp_wait_lcore();
}
```

Please check [l2fwd](rte/examples/l2fwd/l2fwd.rs) example for details.

```
$ sudo RTE_SDK=<rte_path> cargo run --example l2fwd -- --log-level 8 -v -c f -- -p f
```
