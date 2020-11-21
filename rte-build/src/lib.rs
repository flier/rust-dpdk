#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate cc;
extern crate itertools;
extern crate num_cpus;
extern crate pkg_config;
extern crate raw_cpuid;

mod cargo;
mod cpu;
mod gcc;
mod rte;

pub use crate::cargo::{gen_cargo_config, OUT_DIR};
pub use crate::cpu::gen_cpu_features;
pub use crate::gcc::gcc_rte_config;
pub use crate::rte::*;
