# rust-dpdk

Rust-Dpdk is a prototype to wrap [DPDK](http://dpdk.org/) API with Rust language.

## Build

First, please follow [the official document](http://dpdk.org/doc/guides/linux_gsg/build_dpdk.html) to setup a DPDK development envrionment.

compiling problem solve:
before compiling dpdk:

1、if on x86_64 platform:
    export EXTRA_CFLAGS='-fPIC'  #to generate relocaltion opcode
    
    
2、asm patch:
illegal instructions "i"  in rte_rtm.h:56

void rte_xabort(const unsigned int status)
{
     -- asm volatile(".byte 0xc6,0xf8,%P0" :: "i" (status) : "memory");
     ++ asm volatile("": : : "memory");
     ++ _xabort(status);
     ++ asm volatile("": : : "memory");
}

3、change your cc command to clang-3.8 
 alias cc='/usr/bin/clang-3.8'
 without this , _xabort is illegal in gcc

after these configure ,you could compiling dpdk safety.

And build rust-dpdk with `RTE_SDK` envrionment variable:

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
