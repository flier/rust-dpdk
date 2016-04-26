use std::env;

fn main() {
    // make install T=x86_64-native-linuxapp-gcc CONFIG_RTE_BUILD_COMBINE_LIBS=y EXTRA_CFLAGS="-fPIC" -j 4

    let root_dir = env::var("RTE_SDK")
                       .expect("RTE_SDK - Points to the DPDK installation directory.");
    let target = env::var("RTE_TARGET")
                     .expect("RTE_TARGET - Points to the DPDK target environment directory.");
    let base_dir = format!("{}/{}", root_dir, target);
    let include_dir = format!("{}/include", base_dir);
    let lib_dir = format!("{}/lib", base_dir);

    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:include={}", include_dir);
}
