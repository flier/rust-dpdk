#pragma once

#include <rte_bitmap.h>
#include <rte_spinlock.h>
#include <rte_mbuf.h>

/**
 * Seed the pseudo-random generator.
 *
 * The generator is automatically seeded by the EAL init with a timer
 * value. It may need to be re-seeded by the user with a real random
 * value.
 *
 * @param seedval
 *   The value of the seed.
 */
void
_rte_srand(uint64_t seedval);

/**
 * Get a pseudo-random value.
 *
 * This function generates pseudo-random numbers using the linear
 * congruential algorithm and 48-bit integer arithmetic, called twice
 * to generate a 64-bit value.
 *
 * @return
 *   A pseudo-random value between 0 and (1<<64)-1.
 */
uint64_t
_rte_rand(void);

/**
 * Bitmap initialization
 *
 * @param n_bits
 *   Number of pre-allocated bits in array2.
 * @param mem
 *   Base address of array1 and array2.
 * @param mem_size
 *   Minimum expected size of bitmap.
 * @return
 *   Handle to bitmap instance.
 */
struct rte_bitmap *
_rte_bitmap_init(uint32_t n_bits, uint8_t *mem, uint32_t mem_size);

/**
 * Bitmap free
 *
 * @param bmp
 *   Handle to bitmap instance
 * @return
 *   0 upon success, error code otherwise
 */
int _rte_bitmap_free(struct rte_bitmap *bmp);

/**
 * Bitmap reset
 *
 * @param bmp
 *   Handle to bitmap instance
 */
void
_rte_bitmap_reset(struct rte_bitmap *bmp);

/**
 * Bitmap location prefetch into CPU L1 cache
 *
 * @param bmp
 *   Handle to bitmap instance
 * @param pos
 *   Bit position
 * @return
 *   0 upon success, error code otherwise
 */
void
_rte_bitmap_prefetch0(struct rte_bitmap *bmp, uint32_t pos);

/**
 * Bitmap bit get
 *
 * @param bmp
 *   Handle to bitmap instance
 * @param pos
 *   Bit position
 * @return
 *   0 when bit is cleared, non-zero when bit is set
 */
uint64_t
_rte_bitmap_get(struct rte_bitmap *bmp, uint32_t pos);

/**
 * Bitmap bit set
 *
 * @param bmp
 *   Handle to bitmap instance
 * @param pos
 *   Bit position
 */
void
_rte_bitmap_set(struct rte_bitmap *bmp, uint32_t pos);

/**
 * Bitmap slab set
 *
 * @param bmp
 *   Handle to bitmap instance
 * @param pos
 *   Bit position identifying the array2 slab
 * @param slab
 *   Value to be assigned to the 64-bit slab in array2
 */
void
_rte_bitmap_set_slab(struct rte_bitmap *bmp, uint32_t pos, uint64_t slab);

/**
 * Bitmap bit clear
 *
 * @param bmp
 *   Handle to bitmap instance
 * @param pos
 *   Bit position
 */
void
_rte_bitmap_clear(struct rte_bitmap *bmp, uint32_t pos);

/**
 * Bitmap scan (with automatic wrap-around)
 *
 * @param bmp
 *   Handle to bitmap instance
 * @param pos
 *   When function call returns 1, pos contains the position of the next set
 *   bit, otherwise not modified
 * @param slab
 *   When function call returns 1, slab contains the value of the entire 64-bit
 *   slab where the bit indicated by pos is located. Slabs are always 64-bit
 *   aligned, so the position of the first bit of the slab (this bit is not
 *   necessarily set) is pos / 64. Once a slab has been returned by the bitmap
 *   scan operation, the internal pointers of the bitmap are updated to point
 *   after this slab, so the same slab will not be returned again if it
 *   contains more than one bit which is set. When function call returns 0,
 *   slab is not modified.
 * @return
 *   0 if there is no bit set in the bitmap, 1 otherwise
 */
int
_rte_bitmap_scan(struct rte_bitmap *bmp, uint32_t *pos, uint64_t *slab);

/**
 * Bitmap memory footprint calculation
 *
 * @param n_bits
 *   Number of bits in the bitmap
 * @return
 *   Bitmap memory footprint measured in bytes on success, 0 on error
 */
uint32_t _rte_bitmap_get_memory_footprint(uint32_t n_bits);

/**
 * Initialize the spinlock to an unlocked state.
 *
 * @param sl
 *   A pointer to the spinlock.
 */
void
_rte_spinlock_init(rte_spinlock_t *sl);

/**
 * Take the spinlock.
 *
 * @param sl
 *   A pointer to the spinlock.
 */
void
_rte_spinlock_lock(rte_spinlock_t *sl);

/**
 * Release the spinlock.
 *
 * @param sl
 *   A pointer to the spinlock.
 */
void
_rte_spinlock_unlock(rte_spinlock_t *sl);

/**
 * Try to take the lock.
 *
 * @param sl
 *   A pointer to the spinlock.
 * @return
 *   1 if the lock is successfully taken; 0 otherwise.
 */
int
_rte_spinlock_trylock(rte_spinlock_t *sl);

/**
 * Test if hardware transactional memory (lock elision) is supported
 *
 * @return
 *   1 if the hardware transactional memory is supported; 0 otherwise.
 */
int
_rte_tm_supported(void);

/**
 * Try to execute critical section in a hardware memory transaction,
 * if it fails or not available take the spinlock.
 *
 * NOTE: An attempt to perform a HW I/O operation inside a hardware memory
 * transaction always aborts the transaction since the CPU is not able to
 * roll-back should the transaction fail. Therefore, hardware transactional
 * locks are not advised to be used around rte_eth_rx_burst() and
 * rte_eth_tx_burst() calls.
 *
 * @param sl
 *   A pointer to the spinlock.
 */
void
_rte_spinlock_lock_tm(rte_spinlock_t *sl);

/**
 * Try to execute critical section in a hardware memory transaction,
 * if it fails or not available try to take the lock.
 *
 * NOTE: An attempt to perform a HW I/O operation inside a hardware memory
 * transaction always aborts the transaction since the CPU is not able to
 * roll-back should the transaction fail. Therefore, hardware transactional
 * locks are not advised to be used around rte_eth_rx_burst() and
 * rte_eth_tx_burst() calls.
 *
 * @param sl
 *   A pointer to the spinlock.
 * @return
 *   1 if the hardware memory transaction is successfully started
 *   or lock is successfully taken; 0 otherwise.
 */
int
_rte_spinlock_trylock_tm(rte_spinlock_t *sl);

/**
 * Commit hardware memory transaction or release the spinlock if
 * the spinlock is used as a fall-back
 *
 * @param sl
 *   A pointer to the spinlock.
 */
void
_rte_spinlock_unlock_tm(rte_spinlock_t *sl);

/**
 * Initialize the recursive spinlock to an unlocked state.
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 */
void
_rte_spinlock_recursive_init(rte_spinlock_recursive_t *slr);

/**
 * Take the recursive spinlock.
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 */
void
_rte_spinlock_recursive_lock(rte_spinlock_recursive_t *slr);

/**
 * Release the recursive spinlock.
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 */
void
_rte_spinlock_recursive_unlock(rte_spinlock_recursive_t *slr);

/**
 * Try to take the recursive lock.
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 * @return
 *   1 if the lock is successfully taken; 0 otherwise.
 */
int
_rte_spinlock_recursive_trylock(rte_spinlock_recursive_t *slr);

/**
 * Try to execute critical section in a hardware memory transaction,
 * if it fails or not available take the recursive spinlocks
 *
 * NOTE: An attempt to perform a HW I/O operation inside a hardware memory
 * transaction always aborts the transaction since the CPU is not able to
 * roll-back should the transaction fail. Therefore, hardware transactional
 * locks are not advised to be used around rte_eth_rx_burst() and
 * rte_eth_tx_burst() calls.
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 */
void
_rte_spinlock_recursive_lock_tm(rte_spinlock_recursive_t *slr);

/**
 * Commit hardware memory transaction or release the recursive spinlock
 * if the recursive spinlock is used as a fall-back
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 */
void
_rte_spinlock_recursive_unlock_tm(rte_spinlock_recursive_t *slr);

/**
 * Try to execute critical section in a hardware memory transaction,
 * if it fails or not available try to take the recursive lock
 *
 * NOTE: An attempt to perform a HW I/O operation inside a hardware memory
 * transaction always aborts the transaction since the CPU is not able to
 * roll-back should the transaction fail. Therefore, hardware transactional
 * locks are not advised to be used around rte_eth_rx_burst() and
 * rte_eth_tx_burst() calls.
 *
 * @param slr
 *   A pointer to the recursive spinlock.
 * @return
 *   1 if the hardware memory transaction is successfully started
 *   or lock is successfully taken; 0 otherwise.
 */
int
_rte_spinlock_recursive_trylock_tm(rte_spinlock_recursive_t *slr);

/**
 * Return the Application thread ID of the execution unit.
 *
 * Note: in most cases the lcore id returned here will also correspond
 *   to the processor id of the CPU on which the thread is pinned, this
 *   will not be the case if the user has explicitly changed the thread to
 *   core affinities using --lcores EAL argument e.g. --lcores '(0-3)@10'
 *   to run threads with lcore IDs 0, 1, 2 and 3 on physical core 10..
 *
 * @return
 *  Logical core ID (in EAL thread) or LCORE_ID_ANY (in non-EAL thread)
 */
unsigned
_rte_lcore_id(void);

/**
 * Error number value, stored per-thread, which can be queried after
 * calls to certain functions to determine why those functions failed.
 *
 * Uses standard values from errno.h wherever possible, with a small number
 * of additional possible values for RTE-specific conditions.
 */
int
_rte_errno(void);

/**
 * Return the number of TSC cycles since boot
 *
  * @return
 *   the number of cycles
 */
uint64_t
_rte_get_tsc_cycles(void);

/**
 * Get the number of cycles since boot from the default timer.
 *
 * @return
 *   The number of cycles
 */
uint64_t
_rte_get_timer_cycles(void);

/**
 * Get the number of cycles in one second for the default timer.
 *
 * @return
 *   The number of cycles in one second.
 */
uint64_t
_rte_get_timer_hz(void);

/**
 * Wait at least ms milliseconds.
 *
 * @param ms
 *   The number of milliseconds to wait.
 */
void
_rte_delay_ms(unsigned ms);

uint64_t
_rte_rdtsc(void);

uint64_t
_rte_rdtsc_precise(void);

uint64_t
_rte_get_tsc_cycles(void);

/**
 * Return a pointer to the mempool owning this object.
 *
 * @param obj
 *   An object that is owned by a pool. If this is not the case,
 *   the behavior is undefined.
 * @return
 *   A pointer to the mempool structure.
 */
struct rte_mempool *
_rte_mempool_from_obj(void *obj);

/**
 * Return the IO address of elt, which is an element of the pool mp.
 *
 * @param elt
 *   A pointer (virtual address) to the element of the pool.
 * @return
 *   The IO address of the elt element.
 *   If the mempool was created with MEMPOOL_F_NO_IOVA_CONTIG, the
 *   returned value is RTE_BAD_IOVA.
 */
rte_iova_t
_rte_mempool_virt2iova(const void *elt);

/**
 * Return a pointer to the private data in an mempool structure.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @return
 *   A pointer to the private data.
 */
void *
_rte_mempool_get_priv(struct rte_mempool *mp);

/**
 * Flush a user-owned mempool cache to the specified mempool.
 *
 * @param cache
 *   A pointer to the mempool cache.
 * @param mp
 *   A pointer to the mempool.
 */
void
_rte_mempool_cache_flush(struct rte_mempool_cache *cache, struct rte_mempool *mp);

/**
 * Get a pointer to the per-lcore default mempool cache.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param lcore_id
 *   The logical core id.
 * @return
 *   A pointer to the mempool cache or NULL if disabled or non-EAL thread.
 */
struct rte_mempool_cache *
_rte_mempool_default_cache(struct rte_mempool *mp, unsigned lcore_id);

/**
 * Put several objects back in the mempool.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the mempool from the obj_table.
 * @param cache
 *   A pointer to a mempool cache structure. May be NULL if not needed.
 */
void
_rte_mempool_generic_put(struct rte_mempool *mp, void * const *obj_table,
			unsigned int n, struct rte_mempool_cache *cache);

/**
 * Put several objects back in the mempool.
 *
 * This function calls the multi-producer or the single-producer
 * version depending on the default behavior that was specified at
 * mempool creation time (see flags).
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects).
 * @param n
 *   The number of objects to add in the mempool from obj_table.
 */
void
_rte_mempool_put_bulk(struct rte_mempool *mp, void * const *obj_table, unsigned int n);

/**
 * Put one object back in the mempool.
 *
 * This function calls the multi-producer or the single-producer
 * version depending on the default behavior that was specified at
 * mempool creation time (see flags).
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param obj
 *   A pointer to the object to be added.
 */
void
_rte_mempool_put(struct rte_mempool *mp, void *obj);

/**
 * Get several objects from the mempool.
 *
 * If cache is enabled, objects will be retrieved first from cache,
 * subsequently from the common pool. Note that it can return -ENOENT when
 * the local cache and common pool are empty, even if cache from other
 * lcores are full.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to get from mempool to obj_table.
 * @param cache
 *   A pointer to a mempool cache structure. May be NULL if not needed.
 * @return
 *   - 0: Success; objects taken.
 *   - -ENOENT: Not enough entries in the mempool; no object is retrieved.
 */
int
_rte_mempool_generic_get(struct rte_mempool *mp, void **obj_table,
			unsigned int n, struct rte_mempool_cache *cache);

/**
 * Get several objects from the mempool.
 *
 * This function calls the multi-consumers or the single-consumer
 * version, depending on the default behaviour that was specified at
 * mempool creation time (see flags).
 *
 * If cache is enabled, objects will be retrieved first from cache,
 * subsequently from the common pool. Note that it can return -ENOENT when
 * the local cache and common pool are empty, even if cache from other
 * lcores are full.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param obj_table
 *   A pointer to a table of void * pointers (objects) that will be filled.
 * @param n
 *   The number of objects to get from the mempool to obj_table.
 * @return
 *   - 0: Success; objects taken
 *   - -ENOENT: Not enough entries in the mempool; no object is retrieved.
 */
int
_rte_mempool_get_bulk(struct rte_mempool *mp, void **obj_table, unsigned int n);

/**
 * Get one object from the mempool.
 *
 * This function calls the multi-consumers or the single-consumer
 * version, depending on the default behavior that was specified at
 * mempool creation (see flags).
 *
 * If cache is enabled, objects will be retrieved first from cache,
 * subsequently from the common pool. Note that it can return -ENOENT when
 * the local cache and common pool are empty, even if cache from other
 * lcores are full.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param obj_p
 *   A pointer to a void * pointer (object) that will be filled.
 * @return
 *   - 0: Success; objects taken.
 *   - -ENOENT: Not enough entries in the mempool; no object is retrieved.
 */
int
_rte_mempool_get(struct rte_mempool *mp, void **obj_p);


/**
 * @warning
 * @b EXPERIMENTAL: this API may change without prior notice.
 *
 * Get a contiguous blocks of objects from the mempool.
 *
 * If cache is enabled, consider to flush it first, to reuse objects
 * as soon as possible.
 *
 * The application should check that the driver supports the operation
 * by calling rte_mempool_ops_get_info() and checking that `contig_block_size`
 * is not zero.
 *
 * @param mp
 *   A pointer to the mempool structure.
 * @param first_obj_table
 *   A pointer to a pointer to the first object in each block.
 * @param n
 *   The number of blocks to get from mempool.
 * @return
 *   - 0: Success; blocks taken.
 *   - -ENOBUFS: Not enough entries in the mempool; no object is retrieved.
 *   - -EOPNOTSUPP: The mempool driver does not support block dequeue
 */
int __rte_experimental
_rte_mempool_get_contig_blocks(struct rte_mempool *mp, void **first_obj_table, unsigned int n);

/**
 * Prefetch the first part of the mbuf
 *
 * The first 64 bytes of the mbuf corresponds to fields that are used early
 * in the receive path. If the cache line of the architecture is higher than
 * 64B, the second part will also be prefetched.
 *
 * @param m
 *   The pointer to the mbuf.
 */
void
_rte_mbuf_prefetch_part1(struct rte_mbuf *m);

/**
 * Prefetch the second part of the mbuf
 *
 * The next 64 bytes of the mbuf corresponds to fields that are used in the
 * transmit path. If the cache line of the architecture is higher than 64B,
 * this function does nothing as it is expected that the full mbuf is
 * already in cache.
 *
 * @param m
 *   The pointer to the mbuf.
 */
void
_rte_mbuf_prefetch_part2(struct rte_mbuf *m);

/**
 * Return the IO address of the beginning of the mbuf data
 *
 * @param mb
 *   The pointer to the mbuf.
 * @return
 *   The IO address of the beginning of the mbuf data
 */
rte_iova_t
_rte_mbuf_data_iova(const struct rte_mbuf *mb);

/**
 * Return the default IO address of the beginning of the mbuf data
 *
 * This function is used by drivers in their receive function, as it
 * returns the location where data should be written by the NIC, taking
 * the default headroom in account.
 *
 * @param mb
 *   The pointer to the mbuf.
 * @return
 *   The IO address of the beginning of the mbuf data
 */
rte_iova_t
_rte_mbuf_data_iova_default(const struct rte_mbuf *mb);

/**
 * Return the mbuf owning the data buffer address of an indirect mbuf.
 *
 * @param mi
 *   The pointer to the indirect mbuf.
 * @return
 *   The address of the direct mbuf corresponding to buffer_addr.
 */
struct rte_mbuf *
_rte_mbuf_from_indirect(struct rte_mbuf *mi);

/**
 * Return the buffer address embedded in the given mbuf.
 *
 * @param md
 *   The pointer to the mbuf.
 * @return
 *   The address of the data buffer owned by the mbuf.
 */
char *
_rte_mbuf_to_baddr(struct rte_mbuf *md);

/**
 * Return the starting address of the private data area embedded in
 * the given mbuf.
 *
 * Note that no check is made to ensure that a private data area
 * actually exists in the supplied mbuf.
 *
 * @param m
 *   The pointer to the mbuf.
 * @return
 *   The starting address of the private data area of the given mbuf.
 */
void * __rte_experimental
_rte_mbuf_to_priv(struct rte_mbuf *m);

/**
 * Reads the value of an mbuf's refcnt.
 * @param m
 *   Mbuf to read
 * @return
 *   Reference count number.
 */
uint16_t
_rte_mbuf_refcnt_read(const struct rte_mbuf *m);

/**
 * Sets an mbuf's refcnt to a defined value.
 * @param m
 *   Mbuf to update
 * @param new_value
 *   Value set
 */
void
_rte_mbuf_refcnt_set(struct rte_mbuf *m, uint16_t new_value);

/**
 * Adds given value to an mbuf's refcnt and returns its new value.
 * @param m
 *   Mbuf to update
 * @param value
 *   Value to add/subtract
 * @return
 *   Updated value
 */
uint16_t
_rte_mbuf_refcnt_update(struct rte_mbuf *m, int16_t value);

/**
 * Reads the refcnt of an external buffer.
 *
 * @param shinfo
 *   Shared data of the external buffer.
 * @return
 *   Reference count number.
 */
uint16_t
_rte_mbuf_ext_refcnt_read(const struct rte_mbuf_ext_shared_info *shinfo);

/**
 * Set refcnt of an external buffer.
 *
 * @param shinfo
 *   Shared data of the external buffer.
 * @param new_value
 *   Value set
 */
void
_rte_mbuf_ext_refcnt_set(struct rte_mbuf_ext_shared_info *shinfo, uint16_t new_value);

/**
 * Add given value to refcnt of an external buffer and return its new
 * value.
 *
 * @param shinfo
 *   Shared data of the external buffer.
 * @param value
 *   Value to add/subtract
 * @return
 *   Updated value
 */
uint16_t
_rte_mbuf_ext_refcnt_update(struct rte_mbuf_ext_shared_info *shinfo, int16_t value);

/**
 * Allocate an uninitialized mbuf from mempool *mp*.
 *
 * This function can be used by PMDs (especially in RX functions) to
 * allocate an uninitialized mbuf. The driver is responsible of
 * initializing all the required fields. See rte_pktmbuf_reset().
 * For standard needs, prefer rte_pktmbuf_alloc().
 *
 * The caller can expect that the following fields of the mbuf structure
 * are initialized: buf_addr, buf_iova, buf_len, refcnt=1, nb_segs=1,
 * next=NULL, pool, priv_size. The other fields must be initialized
 * by the caller.
 *
 * @param mp
 *   The mempool from which mbuf is allocated.
 * @return
 *   - The pointer to the new mbuf on success.
 *   - NULL if allocation failed.
 */
struct rte_mbuf *
_rte_mbuf_raw_alloc(struct rte_mempool *mp);

/**
 * Put mbuf back into its original mempool.
 *
 * The caller must ensure that the mbuf is direct and properly
 * reinitialized (refcnt=1, next=NULL, nb_segs=1), as done by
 * rte_pktmbuf_prefree_seg().
 *
 * This function should be used with care, when optimization is
 * required. For standard needs, prefer rte_pktmbuf_free() or
 * rte_pktmbuf_free_seg().
 *
 * @param m
 *   The mbuf to be freed.
 */
void
_rte_mbuf_raw_free(struct rte_mbuf *m);

/**
 * Get the data room size of mbufs stored in a pktmbuf_pool
 *
 * The data room size is the amount of data that can be stored in a
 * mbuf including the headroom (RTE_PKTMBUF_HEADROOM).
 *
 * @param mp
 *   The packet mbuf pool.
 * @return
 *   The data room size of mbufs stored in this mempool.
 */
uint16_t
_rte_pktmbuf_data_room_size(struct rte_mempool *mp);

/**
 * Get the application private size of mbufs stored in a pktmbuf_pool
 *
 * The private size of mbuf is a zone located between the rte_mbuf
 * structure and the data buffer where an application can store data
 * associated to a packet.
 *
 * @param mp
 *   The packet mbuf pool.
 * @return
 *   The private size of mbufs stored in this mempool.
 */
uint16_t
_rte_pktmbuf_priv_size(struct rte_mempool *mp);

/**
 * Reset the data_off field of a packet mbuf to its default value.
 *
 * The given mbuf must have only one segment, which should be empty.
 *
 * @param m
 *   The packet mbuf's data_off field has to be reset.
 */
void
_rte_pktmbuf_reset_headroom(struct rte_mbuf *m);

/**
 * Reset the fields of a packet mbuf to their default values.
 *
 * The given mbuf must have only one segment.
 *
 * @param m
 *   The packet mbuf to be resetted.
 */
void
_rte_pktmbuf_reset(struct rte_mbuf *m);

/**
 * Allocate a new mbuf from a mempool.
 *
 * This new mbuf contains one segment, which has a length of 0. The pointer
 * to data is initialized to have some bytes of headroom in the buffer
 * (if buffer size allows).
 *
 * @param mp
 *   The mempool from which the mbuf is allocated.
 * @return
 *   - The pointer to the new mbuf on success.
 *   - NULL if allocation failed.
 */
struct rte_mbuf *
_rte_pktmbuf_alloc(struct rte_mempool *mp);

/**
 * Allocate a bulk of mbufs, initialize refcnt and reset the fields to default
 * values.
 *
 *  @param pool
 *    The mempool from which mbufs are allocated.
 *  @param mbufs
 *    Array of pointers to mbufs
 *  @param count
 *    Array size
 *  @return
 *   - 0: Success
 *   - -ENOENT: Not enough entries in the mempool; no mbufs are retrieved.
 */
int
_rte_pktmbuf_alloc_bulk(struct rte_mempool *pool, struct rte_mbuf **mbufs, unsigned count);

/**
 * Initialize shared data at the end of an external buffer before attaching
 * to a mbuf by ``rte_pktmbuf_attach_extbuf()``. This is not a mandatory
 * initialization but a helper function to simply spare a few bytes at the
 * end of the buffer for shared data. If shared data is allocated
 * separately, this should not be called but application has to properly
 * initialize the shared data according to its need.
 *
 * Free callback and its argument is saved and the refcnt is set to 1.
 *
 * @warning
 * The value of buf_len will be reduced to RTE_PTR_DIFF(shinfo, buf_addr)
 * after this initialization. This shall be used for
 * ``rte_pktmbuf_attach_extbuf()``
 *
 * @param buf_addr
 *   The pointer to the external buffer.
 * @param [in,out] buf_len
 *   The pointer to length of the external buffer. Input value must be
 *   larger than the size of ``struct rte_mbuf_ext_shared_info`` and
 *   padding for alignment. If not enough, this function will return NULL.
 *   Adjusted buffer length will be returned through this pointer.
 * @param free_cb
 *   Free callback function to call when the external buffer needs to be
 *   freed.
 * @param fcb_opaque
 *   Argument for the free callback function.
 *
 * @return
 *   A pointer to the initialized shared data on success, return NULL
 *   otherwise.
 */
struct rte_mbuf_ext_shared_info *
_rte_pktmbuf_ext_shinfo_init_helper(void *buf_addr, uint16_t *buf_len,
	rte_mbuf_extbuf_free_callback_t free_cb, void *fcb_opaque);


/**
 * Attach an external buffer to a mbuf.
 *
 * User-managed anonymous buffer can be attached to an mbuf. When attaching
 * it, corresponding free callback function and its argument should be
 * provided via shinfo. This callback function will be called once all the
 * mbufs are detached from the buffer (refcnt becomes zero).
 *
 * The headroom for the attaching mbuf will be set to zero and this can be
 * properly adjusted after attachment. For example, ``rte_pktmbuf_adj()``
 * or ``rte_pktmbuf_reset_headroom()`` might be used.
 *
 * More mbufs can be attached to the same external buffer by
 * ``rte_pktmbuf_attach()`` once the external buffer has been attached by
 * this API.
 *
 * Detachment can be done by either ``rte_pktmbuf_detach_extbuf()`` or
 * ``rte_pktmbuf_detach()``.
 *
 * Memory for shared data must be provided and user must initialize all of
 * the content properly, escpecially free callback and refcnt. The pointer
 * of shared data will be stored in m->shinfo.
 * ``rte_pktmbuf_ext_shinfo_init_helper`` can help to simply spare a few
 * bytes at the end of buffer for the shared data, store free callback and
 * its argument and set the refcnt to 1. The following is an example:
 *
 *   struct rte_mbuf_ext_shared_info *shinfo =
 *          rte_pktmbuf_ext_shinfo_init_helper(buf_addr, &buf_len,
 *                                             free_cb, fcb_arg);
 *   rte_pktmbuf_attach_extbuf(m, buf_addr, buf_iova, buf_len, shinfo);
 *   rte_pktmbuf_reset_headroom(m);
 *   rte_pktmbuf_adj(m, data_len);
 *
 * Attaching an external buffer is quite similar to mbuf indirection in
 * replacing buffer addresses and length of a mbuf, but a few differences:
 * - When an indirect mbuf is attached, refcnt of the direct mbuf would be
 *   2 as long as the direct mbuf itself isn't freed after the attachment.
 *   In such cases, the buffer area of a direct mbuf must be read-only. But
 *   external buffer has its own refcnt and it starts from 1. Unless
 *   multiple mbufs are attached to a mbuf having an external buffer, the
 *   external buffer is writable.
 * - There's no need to allocate buffer from a mempool. Any buffer can be
 *   attached with appropriate free callback and its IO address.
 * - Smaller metadata is required to maintain shared data such as refcnt.
 *
 * @warning
 * @b EXPERIMENTAL: This API may change without prior notice.
 * Once external buffer is enabled by allowing experimental API,
 * ``RTE_MBUF_DIRECT()`` and ``RTE_MBUF_INDIRECT()`` are no longer
 * exclusive. A mbuf can be considered direct if it is neither indirect nor
 * having external buffer.
 *
 * @param m
 *   The pointer to the mbuf.
 * @param buf_addr
 *   The pointer to the external buffer.
 * @param buf_iova
 *   IO address of the external buffer.
 * @param buf_len
 *   The size of the external buffer.
 * @param shinfo
 *   User-provided memory for shared data of the external buffer.
 */
void __rte_experimental
_rte_pktmbuf_attach_extbuf(struct rte_mbuf *m, void *buf_addr,
	rte_iova_t buf_iova, uint16_t buf_len,
	struct rte_mbuf_ext_shared_info *shinfo);

/**
 * Attach packet mbuf to another packet mbuf.
 *
 * If the mbuf we are attaching to isn't a direct buffer and is attached to
 * an external buffer, the mbuf being attached will be attached to the
 * external buffer instead of mbuf indirection.
 *
 * Otherwise, the mbuf will be indirectly attached. After attachment we
 * refer the mbuf we attached as 'indirect', while mbuf we attached to as
 * 'direct'.  The direct mbuf's reference counter is incremented.
 *
 * Right now, not supported:
 *  - attachment for already indirect mbuf (e.g. - mi has to be direct).
 *  - mbuf we trying to attach (mi) is used by someone else
 *    e.g. it's reference counter is greater then 1.
 *
 * @param mi
 *   The indirect packet mbuf.
 * @param m
 *   The packet mbuf we're attaching to.
 */
void
_rte_pktmbuf_attach(struct rte_mbuf *mi, struct rte_mbuf *m);

/**
 * Detach a packet mbuf from external buffer or direct buffer.
 *
 *  - decrement refcnt and free the external/direct buffer if refcnt
 *    becomes zero.
 *  - restore original mbuf address and length values.
 *  - reset pktmbuf data and data_len to their default values.
 *
 * All other fields of the given packet mbuf will be left intact.
 *
 * @param m
 *   The indirect attached packet mbuf.
 */
void
_rte_pktmbuf_detach(struct rte_mbuf *m);

/**
 * Decrease reference counter and unlink a mbuf segment
 *
 * This function does the same than a free, except that it does not
 * return the segment to its pool.
 * It decreases the reference counter, and if it reaches 0, it is
 * detached from its parent for an indirect mbuf.
 *
 * @param m
 *   The mbuf to be unlinked
 * @return
 *   - (m) if it is the last reference. It can be recycled or freed.
 *   - (NULL) if the mbuf still has remaining references on it.
 */
struct rte_mbuf *
_rte_pktmbuf_prefree_seg(struct rte_mbuf *m);

/**
 * Free a segment of a packet mbuf into its original mempool.
 *
 * Free an mbuf, without parsing other segments in case of chained
 * buffers.
 *
 * @param m
 *   The packet mbuf segment to be freed.
 */
void
_rte_pktmbuf_free_seg(struct rte_mbuf *m);

/**
 * Free a packet mbuf back into its original mempool.
 *
 * Free an mbuf, and all its segments in case of chained buffers. Each
 * segment is added back into its original mempool.
 *
 * @param m
 *   The packet mbuf to be freed. If NULL, the function does nothing.
 */
void
_rte_pktmbuf_free(struct rte_mbuf *m);

/**
 * Creates a "clone" of the given packet mbuf.
 *
 * Walks through all segments of the given packet mbuf, and for each of them:
 *  - Creates a new packet mbuf from the given pool.
 *  - Attaches newly created mbuf to the segment.
 * Then updates pkt_len and nb_segs of the "clone" packet mbuf to match values
 * from the original packet mbuf.
 *
 * @param md
 *   The packet mbuf to be cloned.
 * @param mp
 *   The mempool from which the "clone" mbufs are allocated.
 * @return
 *   - The pointer to the new "clone" mbuf on success.
 *   - NULL if allocation fails.
 */
struct rte_mbuf *
_rte_pktmbuf_clone(struct rte_mbuf *md, struct rte_mempool *mp);

/**
 * Adds given value to the refcnt of all packet mbuf segments.
 *
 * Walks through all segments of given packet mbuf and for each of them
 * invokes rte_mbuf_refcnt_update().
 *
 * @param m
 *   The packet mbuf whose refcnt to be updated.
 * @param v
 *   The value to add to the mbuf's segments refcnt.
 */
void
_rte_pktmbuf_refcnt_update(struct rte_mbuf *m, int16_t v);

/**
 * Get the headroom in a packet mbuf.
 *
 * @param m
 *   The packet mbuf.
 * @return
 *   The length of the headroom.
 */
uint16_t
_rte_pktmbuf_headroom(const struct rte_mbuf *m);

/**
 * Get the tailroom of a packet mbuf.
 *
 * @param m
 *   The packet mbuf.
 * @return
 *   The length of the tailroom.
 */
uint16_t
_rte_pktmbuf_tailroom(const struct rte_mbuf *m);

/**
 * Get the last segment of the packet.
 *
 * @param m
 *   The packet mbuf.
 * @return
 *   The last segment of the given mbuf.
 */
struct rte_mbuf *
_rte_pktmbuf_lastseg(struct rte_mbuf *m);

/**
 * Prepend len bytes to an mbuf data area.
 *
 * Returns a pointer to the new
 * data start address. If there is not enough headroom in the first
 * segment, the function will return NULL, without modifying the mbuf.
 *
 * @param m
 *   The pkt mbuf.
 * @param len
 *   The amount of data to prepend (in bytes).
 * @return
 *   A pointer to the start of the newly prepended data, or
 *   NULL if there is not enough headroom space in the first segment
 */
char *
_rte_pktmbuf_prepend(struct rte_mbuf *m, uint16_t len);

/**
 * Append len bytes to an mbuf.
 *
 * Append len bytes to an mbuf and return a pointer to the start address
 * of the added data. If there is not enough tailroom in the last
 * segment, the function will return NULL, without modifying the mbuf.
 *
 * @param m
 *   The packet mbuf.
 * @param len
 *   The amount of data to append (in bytes).
 * @return
 *   A pointer to the start of the newly appended data, or
 *   NULL if there is not enough tailroom space in the last segment
 */
char *
_rte_pktmbuf_append(struct rte_mbuf *m, uint16_t len);

/**
 * Remove len bytes at the beginning of an mbuf.
 *
 * Returns a pointer to the start address of the new data area. If the
 * length is greater than the length of the first segment, then the
 * function will fail and return NULL, without modifying the mbuf.
 *
 * @param m
 *   The packet mbuf.
 * @param len
 *   The amount of data to remove (in bytes).
 * @return
 *   A pointer to the new start of the data.
 */
char *
_rte_pktmbuf_adj(struct rte_mbuf *m, uint16_t len);

/**
 * Remove len bytes of data at the end of the mbuf.
 *
 * If the length is greater than the length of the last segment, the
 * function will fail and return -1 without modifying the mbuf.
 *
 * @param m
 *   The packet mbuf.
 * @param len
 *   The amount of data to remove (in bytes).
 * @return
 *   - 0: On success.
 *   - -1: On error.
 */
int
_rte_pktmbuf_trim(struct rte_mbuf *m, uint16_t len);

/**
 * Test if mbuf data is contiguous.
 *
 * @param m
 *   The packet mbuf.
 * @return
 *   - 1, if all data is contiguous (one segment).
 *   - 0, if there is several segments.
 */
int
_rte_pktmbuf_is_contiguous(const struct rte_mbuf *m);

/**
 * Read len data bytes in a mbuf at specified offset.
 *
 * If the data is contiguous, return the pointer in the mbuf data, else
 * copy the data in the buffer provided by the user and return its
 * pointer.
 *
 * @param m
 *   The pointer to the mbuf.
 * @param off
 *   The offset of the data in the mbuf.
 * @param len
 *   The amount of bytes to read.
 * @param buf
 *   The buffer where data is copied if it is not contiguous in mbuf
 *   data. Its length should be at least equal to the len parameter.
 * @return
 *   The pointer to the data, either in the mbuf if it is contiguous,
 *   or in the user buffer. If mbuf is too small, NULL is returned.
 */
const void *
_rte_pktmbuf_read(const struct rte_mbuf *m, uint32_t off, uint32_t len, void *buf);

/**
 * Chain an mbuf to another, thereby creating a segmented packet.
 *
 * Note: The implementation will do a linear walk over the segments to find
 * the tail entry. For cases when there are many segments, it's better to
 * chain the entries manually.
 *
 * @param head
 *   The head of the mbuf chain (the first packet)
 * @param tail
 *   The mbuf to put last in the chain
 *
 * @return
 *   - 0, on success.
 *   - -EOVERFLOW, if the chain segment limit exceeded
 */
int
_rte_pktmbuf_chain(struct rte_mbuf *head, struct rte_mbuf *tail);

/**
 * Validate general requirements for Tx offload in mbuf.
 *
 * This function checks correctness and completeness of Tx offload settings.
 *
 * @param m
 *   The packet mbuf to be validated.
 * @return
 *   0 if packet is valid
 */
int
_rte_validate_tx_offload(const struct rte_mbuf *m);

/**
 * Linearize data in mbuf.
 *
 * This function moves the mbuf data in the first segment if there is enough
 * tailroom. The subsequent segments are unchained and freed.
 *
 * @param mbuf
 *   mbuf to linearize
 * @return
 *   - 0, on success
 *   - -1, on error
 */
int
_rte_pktmbuf_linearize(struct rte_mbuf *mbuf);

/**
 *
 * Retrieve a burst of input packets from a receive queue of an Ethernet
 * device. The retrieved packets are stored in *rte_mbuf* structures whose
 * pointers are supplied in the *rx_pkts* array.
 *
 * The rte_eth_rx_burst() function loops, parsing the RX ring of the
 * receive queue, up to *nb_pkts* packets, and for each completed RX
 * descriptor in the ring, it performs the following operations:
 *
 * - Initialize the *rte_mbuf* data structure associated with the
 *   RX descriptor according to the information provided by the NIC into
 *   that RX descriptor.
 *
 * - Store the *rte_mbuf* data structure into the next entry of the
 *   *rx_pkts* array.
 *
 * - Replenish the RX descriptor with a new *rte_mbuf* buffer
 *   allocated from the memory pool associated with the receive queue at
 *   initialization time.
 *
 * When retrieving an input packet that was scattered by the controller
 * into multiple receive descriptors, the rte_eth_rx_burst() function
 * appends the associated *rte_mbuf* buffers to the first buffer of the
 * packet.
 *
 * The rte_eth_rx_burst() function returns the number of packets
 * actually retrieved, which is the number of *rte_mbuf* data structures
 * effectively supplied into the *rx_pkts* array.
 * A return value equal to *nb_pkts* indicates that the RX queue contained
 * at least *rx_pkts* packets, and this is likely to signify that other
 * received packets remain in the input queue. Applications implementing
 * a "retrieve as much received packets as possible" policy can check this
 * specific case and keep invoking the rte_eth_rx_burst() function until
 * a value less than *nb_pkts* is returned.
 *
 * This receive method has the following advantages:
 *
 * - It allows a run-to-completion network stack engine to retrieve and
 *   to immediately process received packets in a fast burst-oriented
 *   approach, avoiding the overhead of unnecessary intermediate packet
 *   queue/dequeue operations.
 *
 * - Conversely, it also allows an asynchronous-oriented processing
 *   method to retrieve bursts of received packets and to immediately
 *   queue them for further parallel processing by another logical core,
 *   for instance. However, instead of having received packets being
 *   individually queued by the driver, this approach allows the caller
 *   of the rte_eth_rx_burst() function to queue a burst of retrieved
 *   packets at a time and therefore dramatically reduce the cost of
 *   enqueue/dequeue operations per packet.
 *
 * - It allows the rte_eth_rx_burst() function of the driver to take
 *   advantage of burst-oriented hardware features (CPU cache,
 *   prefetch instructions, and so on) to minimize the number of CPU
 *   cycles per packet.
 *
 * To summarize, the proposed receive API enables many
 * burst-oriented optimizations in both synchronous and asynchronous
 * packet processing environments with no overhead in both cases.
 *
 * The rte_eth_rx_burst() function does not provide any error
 * notification to avoid the corresponding overhead. As a hint, the
 * upper-level application might check the status of the device link once
 * being systematically returned a 0 value for a given number of tries.
 *
 * @param port_id
 *   The port identifier of the Ethernet device.
 * @param queue_id
 *   The index of the receive queue from which to retrieve input packets.
 *   The value must be in the range [0, nb_rx_queue - 1] previously supplied
 *   to rte_eth_dev_configure().
 * @param rx_pkts
 *   The address of an array of pointers to *rte_mbuf* structures that
 *   must be large enough to store *nb_pkts* pointers in it.
 * @param nb_pkts
 *   The maximum number of packets to retrieve.
 * @return
 *   The number of packets actually retrieved, which is the number
 *   of pointers to *rte_mbuf* structures effectively supplied to the
 *   *rx_pkts* array.
 */
uint16_t
_rte_eth_rx_burst(uint16_t port_id, uint16_t queue_id,
		 struct rte_mbuf **rx_pkts, const uint16_t nb_pkts);

/**
 * Get the number of used descriptors of a rx queue
 *
 * @param port_id
 *  The port identifier of the Ethernet device.
 * @param queue_id
 *  The queue id on the specific port.
 * @return
 *  The number of used descriptors in the specific queue, or:
 *     (-EINVAL) if *port_id* or *queue_id* is invalid
 *     (-ENOTSUP) if the device does not support this function
 */
int
_rte_eth_rx_queue_count(uint16_t port_id, uint16_t queue_id);

/**
 * Check if the DD bit of the specific RX descriptor in the queue has been set
 *
 * @param port_id
 *  The port identifier of the Ethernet device.
 * @param queue_id
 *  The queue id on the specific port.
 * @param offset
 *  The offset of the descriptor ID from tail.
 * @return
 *  - (1) if the specific DD bit is set.
 *  - (0) if the specific DD bit is not set.
 *  - (-ENODEV) if *port_id* invalid.
 *  - (-ENOTSUP) if the device does not support this function
 */
int
_rte_eth_rx_descriptor_done(uint16_t port_id, uint16_t queue_id, uint16_t offset);

/**
 * Check the status of a Rx descriptor in the queue
 *
 * It should be called in a similar context than the Rx function:
 * - on a dataplane core
 * - not concurrently on the same queue
 *
 * Since it's a dataplane function, no check is performed on port_id and
 * queue_id. The caller must therefore ensure that the port is enabled
 * and the queue is configured and running.
 *
 * Note: accessing to a random descriptor in the ring may trigger cache
 * misses and have a performance impact.
 *
 * @param port_id
 *  A valid port identifier of the Ethernet device which.
 * @param queue_id
 *  A valid Rx queue identifier on this port.
 * @param offset
 *  The offset of the descriptor starting from tail (0 is the next
 *  packet to be received by the driver).
 *
 * @return
 *  - (RTE_ETH_RX_DESC_AVAIL): Descriptor is available for the hardware to
 *    receive a packet.
 *  - (RTE_ETH_RX_DESC_DONE): Descriptor is done, it is filled by hw, but
 *    not yet processed by the driver (i.e. in the receive queue).
 *  - (RTE_ETH_RX_DESC_UNAVAIL): Descriptor is unavailable, either hold by
 *    the driver and not yet returned to hw, or reserved by the hw.
 *  - (-EINVAL) bad descriptor offset.
 *  - (-ENOTSUP) if the device does not support this function.
 *  - (-ENODEV) bad port or queue (only if compiled with debug).
 */
int
_rte_eth_rx_descriptor_status(uint16_t port_id, uint16_t queue_id, uint16_t offset);

/**
 * Check the status of a Tx descriptor in the queue.
 *
 * It should be called in a similar context than the Tx function:
 * - on a dataplane core
 * - not concurrently on the same queue
 *
 * Since it's a dataplane function, no check is performed on port_id and
 * queue_id. The caller must therefore ensure that the port is enabled
 * and the queue is configured and running.
 *
 * Note: accessing to a random descriptor in the ring may trigger cache
 * misses and have a performance impact.
 *
 * @param port_id
 *  A valid port identifier of the Ethernet device which.
 * @param queue_id
 *  A valid Tx queue identifier on this port.
 * @param offset
 *  The offset of the descriptor starting from tail (0 is the place where
 *  the next packet will be send).
 *
 * @return
 *  - (RTE_ETH_TX_DESC_FULL) Descriptor is being processed by the hw, i.e.
 *    in the transmit queue.
 *  - (RTE_ETH_TX_DESC_DONE) Hardware is done with this descriptor, it can
 *    be reused by the driver.
 *  - (RTE_ETH_TX_DESC_UNAVAIL): Descriptor is unavailable, reserved by the
 *    driver or the hardware.
 *  - (-EINVAL) bad descriptor offset.
 *  - (-ENOTSUP) if the device does not support this function.
 *  - (-ENODEV) bad port or queue (only if compiled with debug).
 */
int
_rte_eth_tx_descriptor_status(uint16_t port_id,	uint16_t queue_id, uint16_t offset);

/**
 * Send a burst of output packets on a transmit queue of an Ethernet device.
 *
 * The rte_eth_tx_burst() function is invoked to transmit output packets
 * on the output queue *queue_id* of the Ethernet device designated by its
 * *port_id*.
 * The *nb_pkts* parameter is the number of packets to send which are
 * supplied in the *tx_pkts* array of *rte_mbuf* structures, each of them
 * allocated from a pool created with rte_pktmbuf_pool_create().
 * The rte_eth_tx_burst() function loops, sending *nb_pkts* packets,
 * up to the number of transmit descriptors available in the TX ring of the
 * transmit queue.
 * For each packet to send, the rte_eth_tx_burst() function performs
 * the following operations:
 *
 * - Pick up the next available descriptor in the transmit ring.
 *
 * - Free the network buffer previously sent with that descriptor, if any.
 *
 * - Initialize the transmit descriptor with the information provided
 *   in the *rte_mbuf data structure.
 *
 * In the case of a segmented packet composed of a list of *rte_mbuf* buffers,
 * the rte_eth_tx_burst() function uses several transmit descriptors
 * of the ring.
 *
 * The rte_eth_tx_burst() function returns the number of packets it
 * actually sent. A return value equal to *nb_pkts* means that all packets
 * have been sent, and this is likely to signify that other output packets
 * could be immediately transmitted again. Applications that implement a
 * "send as many packets to transmit as possible" policy can check this
 * specific case and keep invoking the rte_eth_tx_burst() function until
 * a value less than *nb_pkts* is returned.
 *
 * It is the responsibility of the rte_eth_tx_burst() function to
 * transparently free the memory buffers of packets previously sent.
 * This feature is driven by the *tx_free_thresh* value supplied to the
 * rte_eth_dev_configure() function at device configuration time.
 * When the number of free TX descriptors drops below this threshold, the
 * rte_eth_tx_burst() function must [attempt to] free the *rte_mbuf*  buffers
 * of those packets whose transmission was effectively completed.
 *
 * If the PMD is DEV_TX_OFFLOAD_MT_LOCKFREE capable, multiple threads can
 * invoke this function concurrently on the same tx queue without SW lock.
 * @see rte_eth_dev_info_get, struct rte_eth_txconf::offloads
 *
 * @see rte_eth_tx_prepare to perform some prior checks or adjustments
 * for offloads.
 *
 * @param port_id
 *   The port identifier of the Ethernet device.
 * @param queue_id
 *   The index of the transmit queue through which output packets must be
 *   sent.
 *   The value must be in the range [0, nb_tx_queue - 1] previously supplied
 *   to rte_eth_dev_configure().
 * @param tx_pkts
 *   The address of an array of *nb_pkts* pointers to *rte_mbuf* structures
 *   which contain the output packets.
 * @param nb_pkts
 *   The maximum number of packets to transmit.
 * @return
 *   The number of output packets actually stored in transmit descriptors of
 *   the transmit ring. The return value can be less than the value of the
 *   *tx_pkts* parameter when the transmit ring is full or has been filled up.
 */
uint16_t
_rte_eth_tx_burst(uint16_t port_id, uint16_t queue_id,
		 struct rte_mbuf **tx_pkts, uint16_t nb_pkts);


/**
 * Process a burst of output packets on a transmit queue of an Ethernet device.
 *
 * The rte_eth_tx_prepare() function is invoked to prepare output packets to be
 * transmitted on the output queue *queue_id* of the Ethernet device designated
 * by its *port_id*.
 * The *nb_pkts* parameter is the number of packets to be prepared which are
 * supplied in the *tx_pkts* array of *rte_mbuf* structures, each of them
 * allocated from a pool created with rte_pktmbuf_pool_create().
 * For each packet to send, the rte_eth_tx_prepare() function performs
 * the following operations:
 *
 * - Check if packet meets devices requirements for tx offloads.
 *
 * - Check limitations about number of segments.
 *
 * - Check additional requirements when debug is enabled.
 *
 * - Update and/or reset required checksums when tx offload is set for packet.
 *
 * Since this function can modify packet data, provided mbufs must be safely
 * writable (e.g. modified data cannot be in shared segment).
 *
 * The rte_eth_tx_prepare() function returns the number of packets ready to be
 * sent. A return value equal to *nb_pkts* means that all packets are valid and
 * ready to be sent, otherwise stops processing on the first invalid packet and
 * leaves the rest packets untouched.
 *
 * When this functionality is not implemented in the driver, all packets are
 * are returned untouched.
 *
 * @param port_id
 *   The port identifier of the Ethernet device.
 *   The value must be a valid port id.
 * @param queue_id
 *   The index of the transmit queue through which output packets must be
 *   sent.
 *   The value must be in the range [0, nb_tx_queue - 1] previously supplied
 *   to rte_eth_dev_configure().
 * @param tx_pkts
 *   The address of an array of *nb_pkts* pointers to *rte_mbuf* structures
 *   which contain the output packets.
 * @param nb_pkts
 *   The maximum number of packets to process.
 * @return
 *   The number of packets correct and ready to be sent. The return value can be
 *   less than the value of the *tx_pkts* parameter when some packet doesn't
 *   meet devices requirements with rte_errno set appropriately:
 *   - -EINVAL: offload flags are not correctly set
 *   - -ENOTSUP: the offload feature is not supported by the hardware
 *
 */
uint16_t
_rte_eth_tx_prepare(uint16_t port_id, uint16_t queue_id,
		struct rte_mbuf **tx_pkts, uint16_t nb_pkts);

/**
 * Send any packets queued up for transmission on a port and HW queue
 *
 * This causes an explicit flush of packets previously buffered via the
 * rte_eth_tx_buffer() function. It returns the number of packets successfully
 * sent to the NIC, and calls the error callback for any unsent packets. Unless
 * explicitly set up otherwise, the default callback simply frees the unsent
 * packets back to the owning mempool.
 *
 * @param port_id
 *   The port identifier of the Ethernet device.
 * @param queue_id
 *   The index of the transmit queue through which output packets must be
 *   sent.
 *   The value must be in the range [0, nb_tx_queue - 1] previously supplied
 *   to rte_eth_dev_configure().
 * @param buffer
 *   Buffer of packets to be transmit.
 * @return
 *   The number of packets successfully sent to the Ethernet device. The error
 *   callback is called for any packets which could not be sent.
 */
uint16_t
_rte_eth_tx_buffer_flush(uint16_t port_id, uint16_t queue_id, struct rte_eth_dev_tx_buffer *buffer);

/**
 * Buffer a single packet for future transmission on a port and queue
 *
 * This function takes a single mbuf/packet and buffers it for later
 * transmission on the particular port and queue specified. Once the buffer is
 * full of packets, an attempt will be made to transmit all the buffered
 * packets. In case of error, where not all packets can be transmitted, a
 * callback is called with the unsent packets as a parameter. If no callback
 * is explicitly set up, the unsent packets are just freed back to the owning
 * mempool. The function returns the number of packets actually sent i.e.
 * 0 if no buffer flush occurred, otherwise the number of packets successfully
 * flushed
 *
 * @param port_id
 *   The port identifier of the Ethernet device.
 * @param queue_id
 *   The index of the transmit queue through which output packets must be
 *   sent.
 *   The value must be in the range [0, nb_tx_queue - 1] previously supplied
 *   to rte_eth_dev_configure().
 * @param buffer
 *   Buffer used to collect packets to be sent.
 * @param tx_pkt
 *   Pointer to the packet mbuf to be sent.
 * @return
 *   0 = packet has been buffered for later transmission
 *   N > 0 = packet has been buffered, and the buffer was subsequently flushed,
 *     causing N packets to be sent, and the error callback to be called for
 *     the rest.
 */
uint16_t
_rte_eth_tx_buffer(uint16_t port_id, uint16_t queue_id,
		struct rte_eth_dev_tx_buffer *buffer, struct rte_mbuf *tx_pkt);

/**
 * Extract VLAN tag information into mbuf
 *
 * Software version of VLAN stripping
 *
 * @param m
 *   The packet mbuf.
 * @return
 *   - 0: Success
 *   - 1: not a vlan packet
 */
int
_rte_vlan_strip(struct rte_mbuf *m);

/**
 * Insert VLAN tag into mbuf.
 *
 * Software version of VLAN unstripping
 *
 * @param m
 *   The packet mbuf.
 * @return
 *   - 0: On success
 *   -EPERM: mbuf is is shared overwriting would be unsafe
 *   -ENOSPC: not enough headroom in mbuf
 */
int
_rte_vlan_insert(struct rte_mbuf **m);
