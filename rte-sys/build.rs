#[macro_use]
extern crate log;
extern crate env_logger;
extern crate gcc;
extern crate regex;

use std::env;
use std::env::consts::*;
use std::io::Cursor;
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

fn build_dpdk(base_dir: &PathBuf) {
    let target = base_dir.file_name().unwrap().to_str().unwrap();

    debug!("building DPDK for target {} @ {}",
           target,
           base_dir.to_str().unwrap());

    Command::new("make")
        .args(&["install",
                format!("T={}", target).as_str(),
                "CONFIG_RTE_BUILD_COMBINE_LIBS=y",
                "EXTRA_CFLAGS='-fPIC -g -ggdb'",
                "-j 4"])
        .current_dir(base_dir.parent().unwrap())
        .status()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));
}

fn gen_rte_config(base_dir: &PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR")
                                    .expect("environment variable `OUT_DIR` not defined"));
    let dest_path = out_dir.join("rte_config.rs");

    debug!("generating rte_config.rs @ {}", out_dir.to_str().unwrap());

    let mut cmd = gcc::Config::new()
                      .include(base_dir.join("include"))
                      .flag("-dM")
                      .flag("-E")
                      .get_compiler()
                      .to_command();

    cmd.arg("src/rte.h");

    let re_num = regex::Regex::new(r"^\(?(?:\([:word:]+\))?\(?(?P<value>-?[:digit:]+)(?P<suffix>[Uu]?[Ll]*)\)?$")
                     .unwrap();
    let re_hex = regex::Regex::new(r"^\(?(?:\([:word:]+\))?\(?(?P<value>0x[:xdigit:]+)(?P<suffix>[Uu]?[Ll]*)\)?$")
                     .unwrap();
    let re_shift = regex::Regex::new(r"^\(?(?P<value>[:digit:]+\s*<<\s*[:digit:]+)\)?$").unwrap();

    debug!("executing: {:?}", cmd);

    let output = cmd.output()
                    .unwrap_or_else(|err| panic!("failed to generate rte_config.rs, {}", err));

    let f = File::create(&dest_path).unwrap();

    fn value_types(sign: bool, long: bool) -> &'static str {
        match (sign, long) {
            (true, true) => "i64",
            (false, true) => "u64",
            (true, false) => "i32",
            (false, false) => "u32",
        }
    }

    for line in Cursor::new(output.stdout)
                    .lines()
                    .map(|r| r.unwrap())
                    .filter(|l| l.starts_with("#define RTE_")) {
        let vars: Vec<&str> = line.splitn(3, " ").collect();

        let name = vars[1];
        let mut value = vars[2];

        if let Some(value_type) = if value.starts_with("\"") && value.ends_with("\"") {
            Some("&'static str")
        } else if let Some(caps) = re_num.captures(value) {
            value = caps.name("value").expect("`value` not found");

            Some(value_types(value.starts_with("-"),
                             caps.name("suffix")
                                 .map_or_else(|| false, |suffix| suffix.to_uppercase() == "ULL")))
        } else if let Some(caps) = re_hex.captures(value) {
            value = caps.name("value").expect("`value` not found");

            Some(value_types(value.starts_with("-"),
                             caps.name("suffix")
                                 .map_or_else(|| false, |suffix| suffix.to_uppercase() == "ULL")))
        } else if let Some(caps) = re_shift.captures(value) {
            value = caps.name("value").expect("`value` not found");

            Some("u32")
        } else {
            None
        } {
            write!(&f,
                   "pub const {}: {} = {}; // {}\n",
                   name,
                   value_type,
                   value,
                   vars[2])
        } else {
            write!(&f, "// #define {} {}\n", name, value)
        }
        .unwrap();
    }
}

fn gen_cargo_config(base_dir: &PathBuf) {
    let libs = vec!["rte_eal", "rte_mempool", "rte_ring", "rte_mbuf"];

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

    let root_dir = PathBuf::from(env::var("RTE_SDK")
                                     .expect("RTE_SDK - Points to the DPDK installation \
                                              directory."));
    let target = env::var("RTE_TARGET")
                     .unwrap_or(String::from(format!("{}-native-{}app-gcc", ARCH, OS)));

    let base_dir = root_dir.join(target);

    if !base_dir.exists() {
        build_dpdk(&base_dir);
    }

    gen_rte_config(&base_dir);

    gen_cargo_config(&base_dir);
}
