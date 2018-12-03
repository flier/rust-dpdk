use std::os::raw::{c_int, c_uchar, c_uint};

use libc::{size_t, uint16_t};

pub use rte_sys::*;

extern "C" {
    #[link_name = "\u{1}_rte_lcore_id"]
    pub fn rte_lcore_id() -> u32;

    /// Error number value, stored per-thread, which can be queried after
    /// calls to certain functions to determine why those functions failed.
    #[link_name = "\u{1}_rte_errno"]
    pub fn rte_errno() -> i32;
}

extern "C" {
    #[link_name = "\u{1}_rte_rdtsc"]
    pub fn rte_rdtsc() -> u64;

    #[link_name = "\u{1}_rte_rdtsc_precise"]
    pub fn rte_rdtsc_precise() -> u64;
}

extern "C" {
    #[link_name = "\u{1}_rte_spinlock_lock"]
    pub fn rte_spinlock_lock(sl: *mut rte_spinlock_t);

    #[link_name = "\u{1}_rte_spinlock_unlock"]
    pub fn rte_spinlock_unlock(sl: *mut rte_spinlock_t);

    #[link_name = "\u{1}_rte_spinlock_trylock"]
    pub fn rte_spinlock_trylock(sl: *mut rte_spinlock_t) -> c_int;

    #[link_name = "\u{1}_rte_tm_supported"]
    pub fn rte_tm_supported() -> c_int;

    #[link_name = "\u{1}_rte_spinlock_lock_tm"]
    pub fn rte_spinlock_lock_tm(sl: *mut rte_spinlock_t);

    #[link_name = "\u{1}_rte_spinlock_unlock_tm"]
    pub fn rte_spinlock_unlock_tm(sl: *mut rte_spinlock_t);

    #[link_name = "\u{1}_rte_spinlock_trylock_tm"]
    pub fn rte_spinlock_trylock_tm(sl: *mut rte_spinlock_t) -> c_int;

    #[link_name = "\u{1}_rte_spinlock_recursive_lock"]
    pub fn rte_spinlock_recursive_lock(sl: *mut rte_spinlock_recursive_t);

    #[link_name = "\u{1}_rte_spinlock_recursive_unlock"]
    pub fn rte_spinlock_recursive_unlock(sl: *mut rte_spinlock_recursive_t);

    #[link_name = "\u{1}_rte_spinlock_recursive_trylock"]
    pub fn rte_spinlock_recursive_trylock(sl: *mut rte_spinlock_recursive_t) -> c_int;

    #[link_name = "\u{1}_rte_spinlock_recursive_lock_tm"]
    pub fn rte_spinlock_recursive_lock_tm(sl: *mut rte_spinlock_recursive_t);

    #[link_name = "\u{1}_rte_spinlock_recursive_unlock_tm"]
    pub fn rte_spinlock_recursive_unlock_tm(sl: *mut rte_spinlock_recursive_t);

    #[link_name = "\u{1}_rte_spinlock_recursive_trylock_tm"]
    pub fn rte_spinlock_recursive_trylock_tm(sl: *mut rte_spinlock_recursive_t) -> c_int;
}

extern "C" {
    /// Bitmap memory footprint calculation
    #[link_name = "\u{1}_rte_bitmap_get_memory_footprint"]
    pub fn rte_bitmap_get_memory_footprint(n_bits: u32) -> u32;

    /// Bitmap initialization
    #[link_name = "\u{1}_rte_bitmap_init"]
    pub fn rte_bitmap_init(n_bits: u32, mem: *mut u8, mem_size: u32) -> *mut rte_bitmap;

    /// Bitmap free
    #[link_name = "\u{1}_rte_bitmap_free"]
    pub fn rte_bitmap_free(bmp: *mut rte_bitmap) -> i32;

    /// Bitmap reset
    #[link_name = "\u{1}_rte_bitmap_reset"]
    pub fn rte_bitmap_reset(bmp: *mut rte_bitmap);

    /// Bitmap location prefetch into CPU L1 cache
    #[link_name = "\u{1}_rte_bitmap_prefetch0"]
    pub fn rte_bitmap_prefetch0(bmp: *mut rte_bitmap, pos: u32);

    /// Bitmap bit get
    #[link_name = "\u{1}_rte_bitmap_get"]
    pub fn rte_bitmap_get(bmp: *mut rte_bitmap, pos: u32) -> u64;

    /// Bitmap bit set
    #[link_name = "\u{1}_rte_bitmap_set"]
    pub fn rte_bitmap_set(bmp: *mut rte_bitmap, pos: u32);

    /// Bitmap slab set
    #[link_name = "\u{1}_rte_bitmap_set_slab"]
    pub fn rte_bitmap_set_slab(bmp: *mut rte_bitmap, pos: u32, slab: u64);

    /// Bitmap bit clear
    #[link_name = "\u{1}_rte_bitmap_clear"]
    pub fn rte_bitmap_clear(bmp: *mut rte_bitmap, pos: u32);

    /// Bitmap scan (with automatic wrap-around)
    #[link_name = "\u{1}_rte_bitmap_scan"]
    pub fn rte_bitmap_scan(bmp: *mut rte_bitmap, pos: *mut u32, slab: *mut u64) -> i32;
}

extern "C" {
    /// Retrieve a burst of input packets from a receive queue of an Ethernet device.
    /// The retrieved packets are stored in *rte_mbuf* structures
    /// whose pointers are supplied in the *rx_pkts* array.
    #[link_name = "\u{1}_rte_eth_rx_burst"]
    pub fn rte_eth_rx_burst(
        port_id: uint16_t,
        queue_id: uint16_t,
        rx_pkts: *mut *mut rte_mbuf,
        nb_pkts: uint16_t,
    ) -> uint16_t;

    /// Send a burst of output packets on a transmit queue of an Ethernet device.
    #[link_name = "\u{1}_rte_eth_tx_burst"]
    pub fn rte_eth_tx_burst(
        port_id: uint16_t,
        queue_id: uint16_t,
        tx_pkts: *mut *mut rte_mbuf,
        nb_pkts: uint16_t,
    ) -> uint16_t;
}

extern "C" {
    /// Allocate a new mbuf from a mempool.
    #[link_name = "\u{1}_rte_pktmbuf_alloc"]
    pub fn rte_pktmbuf_alloc(mp: *mut rte_mempool) -> *mut rte_mbuf;

    /// Free a packet mbuf back into its original mempool.
    #[link_name = "\u{1}_rte_pktmbuf_free"]
    pub fn rte_pktmbuf_free(m: *mut rte_mbuf);

    /// Allocate a bulk of mbufs, initialize refcnt and reset the fields to default values.
    #[link_name = "\u{1}_rte_pktmbuf_alloc_bulk"]
    pub fn rte_pktmbuf_alloc_bulk(
        mp: *mut rte_mempool,
        mbufs: *mut *mut rte_mbuf,
        count: c_uint,
    ) -> c_int;

    /// Creates a "clone" of the given packet mbuf.
    #[link_name = "\u{1}_rte_pktmbuf_clone"]
    pub fn rte_pktmbuf_clone(md: *mut rte_mbuf, mp: *mut rte_mempool) -> *mut rte_mbuf;

    /// Prepend len bytes to an mbuf data area.
    #[link_name = "\u{1}_rte_pktmbuf_prepend"]
    pub fn rte_pktmbuf_prepend(m: *mut rte_mbuf, len: uint16_t) -> *mut c_uchar;

    /// Append len bytes to an mbuf.
    #[link_name = "\u{1}_rte_pktmbuf_append"]
    pub fn rte_pktmbuf_append(m: *mut rte_mbuf, len: uint16_t) -> *mut c_uchar;

    /// Remove len bytes at the beginning of an mbuf.
    #[link_name = "\u{1}_rte_pktmbuf_adj"]
    pub fn rte_pktmbuf_adj(m: *mut rte_mbuf, len: uint16_t) -> *mut c_uchar;

    /// Remove len bytes of data at the end of the mbuf.
    #[link_name = "\u{1}_rte_pktmbuf_trim"]
    pub fn rte_pktmbuf_trim(m: *mut rte_mbuf, len: uint16_t) -> c_int;
}

extern "C" {
    /// Extract VLAN tag information into mbuf
    #[link_name = "\u{1}_rte_vlan_strip"]
    pub fn rte_vlan_strip(m: *mut rte_mbuf) -> libc::c_int;

    /// Insert VLAN tag into mbuf.
    #[link_name = "\u{1}_rte_vlan_insert"]
    pub fn rte_vlan_insert(m: *mut *mut rte_mbuf) -> libc::c_int;
}
