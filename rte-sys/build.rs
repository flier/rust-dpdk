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
    let stub_header = "src/stub.h";

    info!("generating RTE binding file base on \"{}\"", rte_header);

    let rte_sdk_inc_dir = rte_sdk_dir.join("include");
    let cflags = vec!["-march=native", "-I", rte_sdk_inc_dir.to_str().unwrap()];

    bindgen::Builder::default()
        .header(rte_header)
        .header(stub_header)
        .generate_comments(true)
        .generate_inline_functions(true)
        .whitelist_type(r"(rte|cmdline|ether|eth|arp|vlan|vxlan)_.*")
        .whitelist_function(r"(_rte|rte|cmdline|lcore|ether|eth|arp|is)_.*")
        .whitelist_var(
            r"(RTE|CMDLINE|ETHER|ARP|VXLAN|BONDING|LCORE|MEMPOOL|ARP|PKT|EXT_ATTACHED|IND_ATTACHED|lcore|rte|cmdline|per_lcore)_.*",
        )
        .derive_copy(true)
        .derive_debug(true)
        .derive_default(true)
        .derive_partialeq(true)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .clang_arg("-fkeep-inline-functions")
        .clang_args(
            cflags
                .into_iter()
                .map(|s| s.to_owned())
                .chain(gen_cpu_features().map(|(name, value)| {
                    if let Some(value) = value {
                        format!("-D{}={}", name, value)
                    } else {
                        format!("-D{}", name)
                    }
                })),
        )
        .rustfmt_bindings(true)
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
        apply_patches(RTE_SDK.as_path());

        build_dpdk(RTE_SDK.as_path(), RTE_TARGET.as_str());
    }

    if cfg!(feature = "gen") {
        gen_rte_config(&rte_sdk_dir, &OUT_DIR.join("config.rs"));

        let binding_file = OUT_DIR.join("raw.rs");

        gen_rte_binding(&rte_sdk_dir, &binding_file);
    }

    gcc_rte_config(&rte_sdk_dir)
        .file("src/stub.c")
        .include("src")
        .compile("rte_stub");

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
