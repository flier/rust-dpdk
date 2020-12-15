#[macro_use]
extern crate log;

extern crate rte_build;

use rte_build::*;

fn main() {
    pretty_env_logger::init();

    // let rte_sdk_dir = RTE_SDK.join("");

    // info!("using DPDK @ {:?}", rte_sdk_dir);

    gcc_rte_config(&RTE_INCLUDE_DIR)
        .file("examples/l2fwd/l2fwd_core.c")
        .compile("libl2fwd_core.a");
    gcc_rte_config(&RTE_INCLUDE_DIR)
        .file("examples/kni/kni_core.c")
        .compile("libkni_core.a");

    gen_cargo_config(
        RTE_LIB_DIR.iter(),
        RTE_INCLUDE_DIR.iter(),
        RTE_CORE_LIBS.iter().chain(RTE_PMD_LIBS.iter()),
        RTE_DEPS_LIBS.iter(),
    );

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    }
    println!("cargo:rustc-link-lib=static=rte_net_bond" );
    println!("cargo:rustc-link-lib=static=rte_bus_vdev" );
}
