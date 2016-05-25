extern crate env_logger;
extern crate num_cpus;

use std::mem;
use std::sync::{Arc, Mutex};
use std::os::raw::c_void;

use log::LogLevel::Debug;
use cfile::CFile;

use ffi;

use super::*;
use super::mempool::{MemoryPool, MemoryPoolDebug};

#[test]
fn test_eal() {
    let _ = env_logger::init();

    assert_eq!(eal::init(&vec![String::from("test"),
                               String::from("-c"),
                               format!("{:x}", (1 << num_cpus::get()) - 1),
                               String::from("--log-level"),
                               String::from("8")])
                   .unwrap(),
               4);

    assert_eq!(eal::process_type(), eal::ProcType::Primary);
    assert!(!eal::primary_proc_alive());
    assert!(eal::has_hugepages());
    assert_eq!(eal::socket_id(), 0);

    test_config();

    test_lcore();

    test_launch();

    test_mempool();

    test_mbuf();
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
    assert!(lcore::is_enabled(lcore_id));
    assert_eq!(lcore::enabled_lcores().len(), num_cpus::get());

    assert_eq!(lcore::index(256), None);
    assert_eq!(lcore::index(-1), Some(lcore_id));
    assert_eq!(lcore::index(0), Some(lcore_id));
}

fn test_launch() {
    extern "C" fn slave_main(mutex: *const Arc<Mutex<usize>>) -> i32 {
        debug!("lcore {} is running", lcore::id().unwrap());

        let mut data = unsafe { (*mutex).lock().unwrap() };

        *data += 1;

        debug!("lcore {} finished, data={}", lcore::id().unwrap(), *data);

        0
    }

    let mutex = Arc::new(Mutex::new(0));
    let slave_id: u32 = 1;

    assert_eq!(launch::LcoreState::Wait, launch::get_lcore_state(slave_id));

    {
        let data = mutex.lock().unwrap();

        assert_eq!(*data, 0);

        debug!("remote launch lcore {}", slave_id);

        launch::remote_launch(slave_main, Some(&mutex.clone()), slave_id).unwrap();

        assert_eq!(launch::LcoreState::Running,
                   launch::get_lcore_state(slave_id));
    }

    debug!("waiting lcore {} ...", slave_id);

    assert!(launch::wait_lcore(slave_id));

    {
        let data = mutex.lock().unwrap();

        assert_eq!(*data, 1);

        debug!("remote lcore {} finished", slave_id);

        assert_eq!(launch::LcoreState::Wait, launch::get_lcore_state(slave_id));
    }

    {
        let _ = mutex.lock().unwrap();

        debug!("remote launch lcores");

        launch::mp_remote_launch(slave_main, Some(&mutex.clone()), true).unwrap();
    }

    launch::mp_wait_lcore();

    {
        let data = mutex.lock().unwrap();

        debug!("remote lcores finished");

        assert_eq!(*data, num_cpus::get());
    }
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
                                              mempool::MEMPOOL_F_SP_PUT |
                                              mempool::MEMPOOL_F_SC_GET) // flags
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

fn test_mbuf() {
    const NB_MBUF: u32 = 1024;
    const CACHE_SIZE: u32 = 32;
    const PRIV_SIZE: u16 = 0;
    const MBUF_SIZE: u16 = 128;

    let p = mbuf::pktmbuf_pool_create("mbuf_pool",
                                      NB_MBUF,
                                      CACHE_SIZE,
                                      PRIV_SIZE,
                                      mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
                                      eal::socket_id())
        .unwrap();

    assert_eq!(p.name(), "mbuf_pool");
    assert_eq!(p.size(), NB_MBUF);
    assert!(p.phys_addr() != 0);
    assert_eq!(p.cache_size(), CACHE_SIZE);
    assert_eq!(p.cache_flushthresh(), 48);
    assert_eq!(p.elt_size(),
               (mbuf::RTE_MBUF_DEFAULT_BUF_SIZE + PRIV_SIZE + MBUF_SIZE) as u32);
    assert_eq!(p.header_size(), 64);
    assert_eq!(p.trailer_size(), 0);
    assert_eq!(p.private_data_size(), 64);
    assert_eq!((p.elt_va_end() - p.elt_va_start()) as u32,
               (p.header_size() + p.elt_size()) * p.size());
    assert_eq!(p.elt_pa().len(), 1);

    assert_eq!(p.count(), NB_MBUF);
    assert_eq!(p.free_count(), 0);
    assert!(p.full());
    assert!(!p.empty());

    p.audit();
}
