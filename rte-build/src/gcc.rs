use std::path::Path;

use cc;

use crate::gen_cpu_features;

pub fn gcc_rte_config<S: AsRef<Path>>(rte_include_dir: &Vec<S>) -> cc::Build {
    let mut build = cc::Build::new();

    for dir in rte_include_dir {
        build.include(dir);
    }

    build.flag("-march=native").cargo_metadata(true);

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
