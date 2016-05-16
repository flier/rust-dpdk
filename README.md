# rust-dpdk

Rust-Dpdk is a prototype to wrap [DPDK](http://dpdk.org/) API with Rust language.

## Build

First, please follow [the official document](http://dpdk.org/doc/guides/linux_gsg/build_dpdk.html) to setup a DPDK development envrionment.

And build rust-dpdk with `RTE_SDK` envrionment variable:

```
$ RTE_SDK=<rte_path> cargo build
```

## Examples

```
$ sudo RTE_SDK=<rte_path> cargo run --example l2fwd -- --log-level 8 -v -c f -- -p f
```

Please check [l2fwd](rte/examples/l2fwd.rs) example for details.
