extern crate env_logger;
extern crate num_cpus;

use std::mem;
use std::os::raw::c_void;

use log::LogLevel::Debug;
use cfile::CFile;

use ffi;

use super::eal;
use super::lcore;
use super::mempool;
use super::mempool::{MemoryPool, MemoryPoolDebug};

#[test]
fn test_eal() {
    let _ = env_logger::init();

    assert!(eal::init(&vec![String::from("test")]));

    assert_eq!(eal::process_type(), eal::ProcType::Primary);
    assert!(!eal::primary_proc_alive());
    assert!(eal::has_hugepages());
    assert_eq!(eal::socket_id(), 0);

    test_config();

    test_lcore();

    test_mempool();
}

fn test_config() {
    let eal_cfg = eal::get_configuration();

    assert_eq!(eal_cfg.master_lcore(), 0);
    assert_eq!(eal_cfg.lcore_count(), num_cpus::get());
    assert_eq!(eal_cfg.process_type(), eal::ProcType::Primary);
    assert_eq!(eal_cfg.lcore_roles(),
               &[lcore::Role::Rte, lcore::Role::Rte, lcore::Role::Rte, lcore::Role::Rte]);

    let mem_cfg = eal_cfg.memory_config();

    assert_eq!(mem_cfg.nchannel(), 0);
    assert_eq!(mem_cfg.nrank(), 0);

    let memzones = mem_cfg.memzones();

    assert!(memzones.len() > 0);
}

fn test_lcore() {
    assert_eq!(lcore::id(), Some(0));

    let lcore_id = lcore::id().unwrap();

    assert_eq!(lcore::role(lcore_id), lcore::Role::Rte);
    assert_eq!(lcore::master(), 0);
    assert_eq!(lcore::count(), num_cpus::get());
    assert_eq!(lcore::socket_id(lcore_id), 0);
}

fn test_mempool() {
    let p = mempool::create::<c_void, c_void>("test", // name
                                              16, // nll
                                              128, // elt_size
                                              0, // cache_size
                                              32, // private_data_size
                                              None, // mp_init
                                              None, // mp_init_arg
                                              None, // obj_init
                                              None, // obj_init_arg
                                              ffi::SOCKET_ID_ANY, // socket_id
                                              mempool::MEMPOOL_F_SP_PUT | mempool::MEMPOOL_F_SC_GET) // flags
                    .unwrap();

    assert_eq!(p.name(), "test");
    assert_eq!(p.size(), 16);
    assert!(p.phys_addr() != 0);
    assert_eq!(p.cache_size(), 0);
    assert_eq!(p.cache_flushthresh(), 0);
    assert_eq!(p.elt_size(), 128);
    assert_eq!(p.header_size(), 64);
    assert_eq!(p.trailer_size(), 0);
    assert_eq!(p.private_data_size(), 64);
    assert_eq!((p.elt_va_end() - p.elt_va_start()) as u32,
               (p.header_size() + p.elt_size()) * p.size());
    assert_eq!(p.elt_pa().len(), 1);

    assert_eq!(p.count(), 16);
    assert_eq!(p.free_count(), 0);
    assert!(p.full());
    assert!(!p.empty());

    p.audit();

    if log_enabled!(Debug) {
        let stdout = CFile::open_tmpfile().unwrap();

        p.dump(&stdout);
    }

    let mut elements: Vec<(u32, usize)> = Vec::new();

    fn walk_element(elements: Option<&mut Vec<(u32, usize)>>,
                    obj_start: *mut c_void,
                    obj_end: *mut c_void,
                    obj_index: u32) {
        unsafe {
            let obj_addr: usize = mem::transmute(obj_start);
            let obj_end: usize = mem::transmute(obj_end);

            elements.unwrap()
                    .push((obj_index, obj_end - obj_addr));
        }
    }

    assert_eq!(p.walk(4, Some(walk_element), Some(&mut elements)), 4);

    assert_eq!(elements.len(), 4);

    assert_eq!(p, mempool::lookup("test").unwrap());

    let mut pools: Vec<mempool::RawMemoryPoolPtr> = Vec::new();

    fn walk_mempool(pool: mempool::RawMemoryPoolPtr,
                    pools: Option<&mut Vec<mempool::RawMemoryPoolPtr>>) {
        pools.unwrap().push(pool);
    }

    mempool::walk(Some(walk_mempool), Some(&mut pools));

    assert!(pools.iter().find(|pool| **pool == *p).is_some());

    if log_enabled!(Debug) {
        let stdout = CFile::open_tmpfile().unwrap();

        mempool::list_dump(&stdout);
    }
}
