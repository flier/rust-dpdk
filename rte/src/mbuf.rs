use std::ffi::CString;

use ffi::rte_pktmbuf_pool_create;

use errors::{Error, Result};
use mempool::RawMemoryPool;

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
