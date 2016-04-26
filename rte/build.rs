extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    let root_dir = env::var("RTE_SDK")
                       .expect("RTE_SDK - Points to the DPDK installation directory.");
    let target = env::var("RTE_TARGET")
                     .expect("RTE_TARGET - Points to the DPDK target environment directory.");

    let mut include_dir = PathBuf::from(root_dir);

    include_dir.push(target);
    include_dir.push("include");

    let mut config = gcc::Config::new();

    config.include(include_dir);
    config.file("src/rte_helpers.c").compile("librte_helpers.a");
}
