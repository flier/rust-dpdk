use std::env;
use std::path::PathBuf;

lazy_static! {
    pub static ref OUT_DIR: PathBuf = env::var("OUT_DIR").unwrap().into();
}

pub fn gen_cargo_config<S: AsRef<str>>(rte_sdk_dir: &PathBuf, libs: impl Iterator<Item = S>) {
    for lib in libs {
        println!("cargo:rustc-link-lib=static={}", lib.as_ref());
    }

    println!(
        "cargo:rustc-link-search=native={}",
        rte_sdk_dir.join("lib").to_str().unwrap()
    );
    println!(
        "cargo:include={}",
        rte_sdk_dir.join("include").to_str().unwrap()
    );
}
