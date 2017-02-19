#[macro_use]
extern crate log;
extern crate env_logger;
#[cfg(feature = "gen")]
extern crate bindgen;

#[cfg(not(feature = "gen"))]
use std::fs;
use std::env;
use std::env::consts::*;
use std::path::{Path, PathBuf};
#[cfg(feature = "dpdk")]
use std::process::Command;

#[cfg(feature = "gen")]
fn gen_binding(base_dir: &Path, out_file: &Path) {
    let _ = bindgen::builder()
        .clang_arg("-xc++")
        .clang_arg("--std=c++11")
        .clang_arg(if cfg!(feature = "avx") {
            "-mavx"
        } else if cfg!(feature = "sse42") {
            "-msse4.2"
        } else {
            "-march=native"
        })
        .clang_arg("-I")
        .clang_arg(base_dir.join("include").to_string_lossy())
        .header("src/rte.h")
        .no_unstable_rust()
        .disable_name_namespacing()
        .derive_debug(true)
        .derive_default(true)
        .whitelisted_type("^\\w+_hdr$")
        .whitelisted_type("^(rte|pci|cmdline)_.*")
        .whitelisted_function("^(rte|cmdline|mp|pciinitfn|vdrvinitfn|tailqinitfn)_.*")
        .whitelisted_var("^(lcore|cmdline|RTE|SOCKET|CMDLINE|BONDING|ETHER|PKT|ARP)_.*")
        .link_static("rte_pmd_af_packet")
        .generate()
        .unwrap()
        .write_to_file(out_file)
        .expect("fail to write bindings");
}

#[cfg(not(feature = "gen"))]
fn gen_binding(_: &Path, out_file: &Path) {
    fs::copy("src/raw.rs", out_file).expect("fail to copy bindings");
}

#[cfg(feature = "dpdk")]
fn build_dpdk(base_dir: &PathBuf) {
    let target = base_dir.file_name().unwrap().to_str().unwrap();

    debug!("building DPDK for target {} @ {}",
           target,
           base_dir.to_str().unwrap());

    Command::new("make")
        .args(&["install",
                format!("T={}", target).as_str(),
                "CONFIG_RTE_BUILD_COMBINE_LIBS=y",
                if cfg!(feature = "debug") {
                    "EXTRA_CFLAGS='-fPIC -g -ggdb'"
                } else {
                    "EXTRA_CFLAGS='-fPIC -O3 -ggdb'"
                }])
        .current_dir(base_dir.parent().unwrap())
        .status()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
}

#[cfg(not(feature = "dpdk"))]
fn build_dpdk(base_dir: &PathBuf) {
    if !base_dir.is_dir() {
        panic!("DPDK build not ready at {}", base_dir.to_str().unwrap());
    }
}

fn gen_cargo_config(base_dir: &PathBuf) {
    let libs = vec!["rte_acl",
                    "rte_cfgfile",
                    "rte_cmdline",
                    "rte_cryptodev",
                    "rte_distributor",
                    "rte_eal",
                    "rte_ethdev",
                    "rte_hash",
                    "rte_ip_frag",
                    "rte_jobstats",
                    "rte_kni",
                    "rte_kvargs",
                    "rte_lpm",
                    "rte_mbuf",
                    "rte_mempool",
                    "rte_meter",
                    "rte_net",
                    "rte_pdump",
                    "rte_pipeline",
                    "rte_port",
                    "rte_power",
                    "rte_reorder",
                    "rte_ring",
                    "rte_sched",
                    "rte_table",
                    "rte_timer",
                    "rte_vhost",
                    "rte_pmd_af_packet",
                    "rte_pmd_bond",
                    "rte_pmd_cxgbe",
                    "rte_pmd_e1000",
                    "rte_pmd_ena",
                    "rte_pmd_enic",
                    "rte_pmd_fm10k",
                    "rte_pmd_i40e",
                    "rte_pmd_ixgbe",
                    "rte_pmd_null",
                    "rte_pmd_null_crypto",
                    "rte_pmd_qede",
                    "rte_pmd_ring",
                    "rte_pmd_vhost",
                    "rte_pmd_virtio",
                    "rte_pmd_vmxnet3_uio"];

    for lib in libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    println!("cargo:rustc-link-search=native={}",
             base_dir.join("lib").to_str().unwrap());
    println!("cargo:include={}",
             base_dir.join("include").to_str().unwrap());
}

fn main() {
    env_logger::init().unwrap();

    let root_dir = env::var("RTE_SDK")
        .expect("RTE_SDK - Points to the DPDK installation directory.");
    let target = env::var("RTE_TARGET")
        .unwrap_or(String::from(format!("{}-native-{}app-gcc", ARCH, OS)));

    let base_dir = PathBuf::from(root_dir).join(target);

    build_dpdk(&base_dir);

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("raw.rs");

    gen_binding(&base_dir, &out_file);

    gen_cargo_config(&base_dir);
}
