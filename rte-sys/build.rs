#[macro_use]
extern crate log;
extern crate pretty_env_logger;

#[cfg(feature = "gen")]
extern crate bindgen;

extern crate rte_build;

use std::path::Path;

use rte_build::*;

#[cfg(feature = "gen")]
fn gen_rte_binding(rte_sdk_dir: &Path, dest_path: &Path) {
    let rte_header = "src/rte.h";

    info!("generating RTE binding file base on \"{}\"", rte_header);

    let rte_sdk_inc_dir = rte_sdk_dir.join("include");
    let cflags = vec!["-march=native", "-I", rte_sdk_inc_dir.to_str().unwrap()];

    bindgen::Builder::default()
        .header(rte_header)
        .generate_comments(true)
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_args(
            cflags
                .into_iter()
                .map(|s| s.to_owned())
                .chain(gen_cpu_features()),
        ).rustfmt_bindings(true)
        .time_phases(true)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(dest_path)
        .expect("Couldn't write bindings!");
}

#[cfg(not(feature = "gen"))]
fn gen_rte_binding(_rte_sdk_dir: &Path, dest_path: &Path) {
    use std::fs;

    info!("coping RTE binding file");

    fs::copy("src/raw.rs", dest_path).expect("copy binding file");
}

fn main() {
    pretty_env_logger::init();

    let rte_sdk_dir = RTE_SDK.join(RTE_TARGET.as_str());

    info!("using DPDK @ {:?}", rte_sdk_dir);

    if !rte_sdk_dir.exists() || !rte_sdk_dir.join("lib/libdpdk.a").exists() {
        build_dpdk(RTE_SDK.as_path(), RTE_TARGET.as_str());
    }

    gen_rte_config(&rte_sdk_dir, &OUT_DIR.join("config.rs"));

    gen_rte_binding(&rte_sdk_dir, &OUT_DIR.join("raw.rs"));

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
