use std::env;
use std::env::consts::*;
use std::path::Path;
use std::process::Command;

fn main() {
    let root_dir = env::var("RTE_SDK")
                       .expect("RTE_SDK - Points to the DPDK installation directory.");
    let target = env::var("RTE_TARGET")
                     .unwrap_or(String::from(format!("{}-native-{}app-gcc", ARCH, OS)));
    let base_dir = format!("{}/{}", root_dir, target);

    let include_dir = format!("{}/include", base_dir);
    let lib_dir = format!("{}/lib", base_dir);

    if !Path::new(&base_dir).exists() {
        Command::new("make")
            .args(&["install",
                    format!("T={}", target).as_str(),
                    "CONFIG_RTE_BUILD_COMBINE_LIBS=y",
                    "EXTRA_CFLAGS='-fPIC'",
                    "-j 4"])
            .current_dir(root_dir)
            .status()
            .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
    }

    let libs = vec!["rte_eal", "rte_mempool", "rte_ring"];

    for lib in libs {
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:include={}", include_dir);
}
