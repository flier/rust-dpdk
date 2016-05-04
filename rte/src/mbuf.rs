use std::ffi::CString;

use ffi::*;

use errors::{Error, Result};
use mempool::RawMemoryPool;


/**
 * Some NICs need at least 2KB buffer to RX standard Ethernet frame without
 * splitting it into multiple segments.
 * So, for mbufs that planned to be involved into RX/TX, the recommended
 * minimal buffer length is 2KB + RTE_PKTMBUF_HEADROOM.
 */
pub const RTE_MBUF_DEFAULT_DATAROOM: u32 = 2048;
pub const RTE_MBUF_DEFAULT_BUF_SIZE: u32 = RTE_MBUF_DEFAULT_DATAROOM + RTE_PKTMBUF_HEADROOM;


/// Create a mbuf pool.
///
/// This function creates and initializes a packet mbuf pool.
/// It is a wrapper to rte_mempool_create() with the proper packet constructor
/// and mempool constructor.
pub fn pktmbuf_pool_create(name: &str,
                           n: usize,
                           cache_size: usize,
                           priv_size: usize,
                           data_room_size: usize,
                           socket_id: usize)
                           -> Result<RawMemoryPool> {
    let name = try!(CString::new(name))
                   .as_bytes_with_nul()
                   .as_ptr() as *const i8;

    let p = unsafe {
        rte_pktmbuf_pool_create(name,
                                n as u32,
                                cache_size as u32,
                                priv_size as u16,
                                data_room_size as u16,
                                socket_id as i32)
    };

    if p.is_null() {
        Err(Error::rte_error())
    } else {
        Ok(RawMemoryPool::from_raw(p))
    }
}
