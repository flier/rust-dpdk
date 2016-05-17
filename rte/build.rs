extern crate gcc;

use std::env;
use std::env::consts::*;
use std::path::PathBuf;

fn rte_config(base_dir: &PathBuf) -> gcc::Config {
    let mut cfg = gcc::Config::new();

    cfg.include(base_dir.join("include"))
       .flag("-march=native")
       .flag("-DRTE_MACHINE_CPUFLAG_SSE")
       .flag("-DRTE_MACHINE_CPUFLAG_SSE2")
       .flag("-DRTE_MACHINE_CPUFLAG_SSE3")
       .flag("-DRTE_MACHINE_CPUFLAG_SSSE3")
       .flag("-DRTE_MACHINE_CPUFLAG_SSE4_1")
       .flag("-DRTE_MACHINE_CPUFLAG_SSE4_2")
       .flag("-DRTE_MACHINE_CPUFLAG_AES")
       .flag("-DRTE_MACHINE_CPUFLAG_PCLMULQDQ")
       .flag("-DRTE_MACHINE_CPUFLAG_AVX")
       .flag("-DRTE_MACHINE_CPUFLAG_RDRAND")
       .flag("-DRTE_MACHINE_CPUFLAG_FSGSBASE")
       .flag("-DRTE_MACHINE_CPUFLAG_F16C")
       .flag("-DRTE_MACHINE_CPUFLAG_AVX2")
       .flag("-DRTE_COMPILE_TIME_CPUFLAGS=RTE_CPUFLAG_SSE,RTE_CPUFLAG_SSE2,RTE_CPUFLAG_SSE3,\
              RTE_CPUFLAG_SSSE3,RTE_CPUFLAG_SSE4_1,RTE_CPUFLAG_SSE4_2,RTE_CPUFLAG_AES,\
              RTE_CPUFLAG_PCLMULQDQ,RTE_CPUFLAG_AVX,RTE_CPUFLAG_RDRAND,RTE_CPUFLAG_FSGSBASE,\
              RTE_CPUFLAG_F16C,RTE_CPUFLAG_AVX2");
    cfg
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

    rte_config(&base_dir)
        .file("src/rte_helpers.c")
        .compile("librte_helpers.a");

    gen_cargo_config(&base_dir);

    rte_config(&base_dir)
        .file("examples/l2fwd/l2fwd_core.c")
        .compile("libl2fwd_core.a");
}
