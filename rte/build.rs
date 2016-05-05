extern crate gcc;

use std::env;
use std::env::consts::*;
use std::path::PathBuf;

fn build_rte_helpers(base_dir: &PathBuf) {
    let mut config = gcc::Config::new();

    config.include(base_dir.join("include"));
    config.file("src/rte_helpers.c").compile("librte_helpers.a");
}

fn gen_cargo_config(base_dir: &PathBuf) {
    let libs = vec!["rte_pmd_af_packet",
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
    let root_dir = env::var("RTE_SDK")
                       .expect("RTE_SDK - Points to the DPDK installation directory.");
    let target = env::var("RTE_TARGET")
                     .unwrap_or(String::from(format!("{}-native-{}app-gcc", ARCH, OS)));

    let base_dir = PathBuf::from(root_dir).join(target);

    build_rte_helpers(&base_dir);

    gen_cargo_config(&base_dir);
}
