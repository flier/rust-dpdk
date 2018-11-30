use std::env;
use std::path::Path;
use std::process::Command;

use num_cpus;

pub fn build_dpdk(rte_sdk: &Path, rte_target: &str) {
    let debug_mode = env::var("DEBUG")
        .map(|s| s.parse().unwrap_or_default())
        .unwrap_or_default();

    info!(
        "building {} mode DPDK {} @ {:?}",
        if debug_mode { "debug" } else { "release" },
        rte_target,
        rte_sdk
    );

    Command::new("make")
        .arg("install")
        .arg(format!("T={}", rte_target))
        .args(&["-j", &num_cpus::get().to_string()])
        .env("CONFIG_RTE_BUILD_COMBINE_LIBS", "y")
        .env(
            "EXTRA_CFLAGS",
            if debug_mode {
                "-fPIC -O0 -g -ggdb"
            } else {
                "-fPIC -O"
            },
        ).current_dir(rte_sdk)
        .status()
        .unwrap_or_else(|e| panic!("failed to build DPDK: {}", e));
}
