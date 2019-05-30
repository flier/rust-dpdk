use std::path::Path;

use cc;

use crate::gen_cpu_features;

pub fn gcc_rte_config(rte_sdk_dir: &Path) -> cc::Build {
    let mut build = cc::Build::new();

    build
        .include(rte_sdk_dir.join("include"))
        .flag("-march=native")
        .cargo_metadata(true);

    for (name, value) in gen_cpu_features() {
        let define = if let Some(value) = value {
            format!("-D{}={}", name, value)
        } else {
            format!("-D{}", name)
        };

        build.flag(&define);
    }

    build
}
