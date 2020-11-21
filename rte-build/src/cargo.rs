use std::env;
use std::path::PathBuf;

lazy_static! {
    pub static ref OUT_DIR: PathBuf = env::var("OUT_DIR").unwrap().into();
}

pub fn gen_cargo_config<S: AsRef<str>>(
    lib_dirs: impl Iterator<Item = S>,
    include_dirs: impl Iterator<Item = S>,
    static_libs: impl Iterator<Item = S>,
    shared_libs: impl Iterator<Item = S>,
) {
    for lib in static_libs {
        println!("cargo:rustc-link-lib=static={}", lib.as_ref());
    }

    for lib in shared_libs {
        println!("cargo:rustc-link-lib={}", lib.as_ref());
    }

    for dir in lib_dirs{
        println!( "cargo:rustc-link-search=native={}", dir.as_ref());
    }

    for dir in include_dirs{
        println!("cargo:include={}", dir.as_ref());
    }
}
