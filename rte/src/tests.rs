extern crate num_cpus;
extern crate pretty_env_logger;

use std::os::raw::c_void;
use std::sync::{Arc, Mutex};

use cfile;
use log::Level::Debug;

use ffi;

use eal;
use launch;
use lcore;
use mbuf;
use memory::AsMutRef;
use mempool::{self, MemoryPool, MemoryPoolDebug, MemoryPoolFlags};

#[test]
fn test_eal() {
    let _ = pretty_env_logger::try_init_timed();

    assert_eq!(
        eal::init(&vec![
            String::from("test"),
            String::from("-c"),
            format!("{:x}", (1 << num_cpus::get()) - 1),
            String::from("--log-level"),
            String::from("8")
        ]).unwrap(),
        4
    );

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
    let eal_cfg = eal::config();

    assert_eq!(eal_cfg.master_lcore(), 0);
    assert_eq!(eal_cfg.lcore_count(), num_cpus::get());
    assert_eq!(eal_cfg.process_type(), eal::ProcType::Primary);
    assert_eq!(
        eal_cfg.lcore_roles(),
        &[
            lcore::Role::Rte,
            lcore::Role::Rte,
            lcore::Role::Rte,
            lcore::Role::Rte
        ]
    );

    let mem_cfg = eal_cfg.memory_config();

    assert_eq!(mem_cfg.nchannel(), 0);
    assert_eq!(mem_cfg.nrank(), 0);

    let memzones = mem_cfg.memzones();

    assert!(memzones.len() > 0);
}

fn test_lcore() {
    assert_eq!(lcore::current().unwrap(), 0);

    let lcore_id = lcore::current().unwrap();

    assert_eq!(lcore_id.role(), lcore::Role::Rte);
    assert_eq!(lcore_id.socket_id(), 0);
    assert!(lcore_id.is_enabled());

    assert_eq!(lcore::master(), 0);
    assert_eq!(lcore::count(), num_cpus::get());
    assert_eq!(lcore::enabled().len(), num_cpus::get());

    assert_eq!(lcore::index(256), None);
    assert_eq!(lcore::Id::any().index(), 0);
    assert_eq!(lcore::id(0).index(), 0);
}

fn test_launch() {
    fn slave_main(mutex: Option<Arc<Mutex<usize>>>) -> i32 {
        debug!("lcore {} is running", lcore::current().unwrap());

        let mutex = mutex.unwrap();
        let mut data = mutex.lock().unwrap();

        *data += 1;

        debug!(
            "lcore {} finished, data={}",
            lcore::current().unwrap(),
            *data
        );

        0
    }

    let mutex = Arc::new(Mutex::new(0));
    let slave_id = lcore::id(1);

    assert_eq!(slave_id.state(), lcore::State::Wait);

    {
        let data = mutex.lock().unwrap();

        assert_eq!(*data, 0);

        debug!("remote launch lcore {}", slave_id);

        launch::remote_launch(slave_main, Some(mutex.clone()), slave_id).unwrap();

        assert_eq!(slave_id.state(), lcore::State::Running);
    }

    debug!("waiting lcore {} ...", slave_id);

    assert!(launch::wait_lcore(slave_id));

    {
        let data = mutex.lock().unwrap();

        assert_eq!(*data, 1);

        debug!("remote lcore {} finished", slave_id);

        assert_eq!(slave_id.state(), lcore::State::Wait);
    }

    {
        let _ = mutex.lock().unwrap();

        debug!("remote launch lcores");

        launch::mp_remote_launch(slave_main, Some(mutex.clone()), true).unwrap();
    }

    launch::mp_wait_lcore();

    {
        let data = mutex.lock().unwrap();

        debug!("remote lcores finished");

        assert_eq!(*data, num_cpus::get());
    }
}

fn test_mempool() {
    let p = mempool::create::<c_void, c_void>(
        "test",
        16,
        128,
        0,
        32,
        None,
        None,
        None,
        None,
        ffi::SOCKET_ID_ANY,
        MemoryPoolFlags::MEMPOOL_F_SP_PUT | MemoryPoolFlags::MEMPOOL_F_SC_GET,
    ).as_mut_ref()
    .unwrap();

    assert_eq!(p.name(), "test");
    assert_eq!(p.size, 16);
    assert_eq!(p.cache_size, 0);
    assert_eq!(p.elt_size, 128);
    assert_eq!(p.header_size, 64);
    assert_eq!(p.trailer_size, 0);
    assert_eq!(p.private_data_size, 64);

    assert_eq!(p.avail_count(), 16);
    assert_eq!(p.in_use_count(), 0);
    assert!(p.is_full());
    assert!(!p.is_empty());

    p.audit();

    if log_enabled!(Debug) {
        let stdout = cfile::tmpfile().unwrap();

        p.dump(&stdout);
    }

    let mut elements: Vec<(u32, *mut c_void)> = Vec::new();

    fn walk_element(
        _pool: mempool::RawMemoryPoolPtr,
        elements: Option<&mut Vec<(u32, *mut c_void)>>,
        obj: *mut c_void,
        obj_index: u32,
    ) {
        elements.unwrap().push((obj_index, obj));
    }

    assert_eq!(p.walk(walk_element, Some(&mut elements)), 4);

    assert_eq!(elements.len(), 4);

    let raw_ptr = p as mempool::RawMemoryPoolPtr;

    assert_eq!(raw_ptr, mempool::lookup("test").unwrap());

    let mut pools: Vec<mempool::RawMemoryPoolPtr> = Vec::new();

    fn walk_mempool(
        pool: mempool::RawMemoryPoolPtr,
        pools: Option<&mut Vec<mempool::RawMemoryPoolPtr>>,
    ) {
        pools.unwrap().push(pool);
    }

    mempool::walk(walk_mempool, Some(&mut pools));

    assert!(pools.contains(&raw_ptr));

    if log_enabled!(Debug) {
        let stdout = cfile::tmpfile().unwrap();

        mempool::list_dump(&stdout);
    }
}

fn test_mbuf() {
    const NB_MBUF: u32 = 1024;
    const CACHE_SIZE: u32 = 32;
    const PRIV_SIZE: u16 = 0;
    const MBUF_SIZE: u16 = 128;

    let p = mbuf::pktmbuf_pool_create(
        "mbuf_pool",
        NB_MBUF,
        CACHE_SIZE,
        PRIV_SIZE,
        mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
        eal::socket_id(),
    ).as_mut_ref()
    .unwrap();

    assert_eq!(p.name(), "mbuf_pool");
    assert_eq!(p.size, NB_MBUF);
    assert_eq!(p.cache_size, CACHE_SIZE);
    assert_eq!(
        p.elt_size,
        (mbuf::RTE_MBUF_DEFAULT_BUF_SIZE + PRIV_SIZE + MBUF_SIZE) as u32
    );
    assert_eq!(p.header_size, 64);
    assert_eq!(p.trailer_size, 0);
    assert_eq!(p.private_data_size, 64);

    assert_eq!(p.avail_count(), NB_MBUF as usize);
    assert_eq!(p.in_use_count(), 0);
    assert!(p.is_full());
    assert!(!p.is_empty());

    p.audit();
}
