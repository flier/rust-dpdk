#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "gen")] {
        include!(concat!(env!("OUT_DIR"), "/config.rs"));
        include!(concat!(env!("OUT_DIR"), "/raw.rs"));
    } else {
        include!("config.rs");
        include!("raw.rs");
    }
}
