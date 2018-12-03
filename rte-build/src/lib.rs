#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate cc;
extern crate itertools;
extern crate num_cpus;
extern crate raw_cpuid;

mod build;
mod cargo;
mod cpu;
mod gcc;
mod rte;

pub use build::build_dpdk;
pub use cargo::{gen_cargo_config, OUT_DIR};
pub use cpu::gen_cpu_features;
pub use gcc::gcc_rte_config;
pub use rte::*;
