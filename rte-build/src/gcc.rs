use std::path::Path;

use cc;
use gen_cpu_features;

pub fn gcc_rte_config(rte_sdk_dir: &Path) -> cc::Build {
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
