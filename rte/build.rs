#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate cc;

extern crate rte_build;

use std::path::Path;

use rte_build::*;

fn rte_config(rte_sdk_dir: &Path) -> cc::Build {
    let mut build = cc::Build::new();

    build
        .include(rte_sdk_dir.join("include"))
        .flag("-march=native")
        .cargo_metadata(true);

    for flag in gen_cpu_features() {
        build.flag(&flag);
    }

    build
}

fn main() {
    pretty_env_logger::init();

    let rte_sdk_dir = RTE_SDK.join(RTE_TARGET.as_str());

    info!("using DPDK @ {:?}", rte_sdk_dir);

    rte_config(&rte_sdk_dir)
        .file("src/rte_helpers.c")
        .compile("librte_helpers.a");

    rte_config(&rte_sdk_dir)
        .file("examples/l2fwd/l2fwd_core.c")
        .compile("libl2fwd_core.a");

    rte_config(&rte_sdk_dir)
        .file("examples/kni/kni_core.c")
        .compile("libkni_core.a");

    gen_cargo_config(
        &rte_sdk_dir,
        RTE_CORE_LIBS
            .iter()
            .chain(RTE_PMD_LIBS.iter())
            .chain(RTE_DEPS_LIBS.iter()),
    );

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    }
}
