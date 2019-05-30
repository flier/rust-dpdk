#include "rte.h"

void
_rte_srand(uint64_t seedval) {
    rte_srand(seedval);
}

uint64_t
_rte_rand(void) {
    return rte_rand();
}

struct rte_bitmap *
_rte_bitmap_init(uint32_t n_bits, uint8_t *mem, uint32_t mem_size) {
    return rte_bitmap_init(n_bits, mem, mem_size);
}

int
_rte_bitmap_free(struct rte_bitmap *bmp) {
    return rte_bitmap_free(bmp);
}

void
_rte_bitmap_reset(struct rte_bitmap *bmp) {
    rte_bitmap_reset(bmp);
}

void
_rte_bitmap_prefetch0(struct rte_bitmap *bmp, uint32_t pos) {
    rte_bitmap_prefetch0(bmp, pos);
}

uint64_t
_rte_bitmap_get(struct rte_bitmap *bmp, uint32_t pos) {
    return rte_bitmap_get(bmp, pos);
}

void
_rte_bitmap_set(struct rte_bitmap *bmp, uint32_t pos) {
    rte_bitmap_set(bmp, pos);
}

void
_rte_bitmap_set_slab(struct rte_bitmap *bmp, uint32_t pos, uint64_t slab) {
    return rte_bitmap_set_slab(bmp, pos, slab);
}

void
_rte_bitmap_clear(struct rte_bitmap *bmp, uint32_t pos) {
    return rte_bitmap_clear(bmp, pos);
}

int
_rte_bitmap_scan(struct rte_bitmap *bmp, uint32_t *pos, uint64_t *slab) {
    return rte_bitmap_scan(bmp, pos, slab);
}

uint32_t
_rte_bitmap_get_memory_footprint(uint32_t n_bits) {
    return rte_bitmap_get_memory_footprint(n_bits);
}

void
_rte_spinlock_init(rte_spinlock_t *sl) {
    rte_spinlock_init(sl);
}

void
_rte_spinlock_lock(rte_spinlock_t *sl) {
    rte_spinlock_lock(sl);
}

void
_rte_spinlock_unlock(rte_spinlock_t *sl) {
    rte_spinlock_unlock(sl);
}

int
_rte_spinlock_trylock(rte_spinlock_t *sl) {
    return rte_spinlock_trylock(sl);
}

int
_rte_tm_supported(void) {
    return rte_tm_supported();
}

void
_rte_spinlock_lock_tm(rte_spinlock_t *sl) {
    rte_spinlock_lock_tm(sl);
}

int
_rte_spinlock_trylock_tm(rte_spinlock_t *sl) {
    return rte_spinlock_trylock_tm(sl);
}

void
_rte_spinlock_unlock_tm(rte_spinlock_t *sl) {
    rte_spinlock_unlock_tm(sl);
}

void
_rte_spinlock_recursive_init(rte_spinlock_recursive_t *slr) {
    rte_spinlock_recursive_init(slr);
}

void
_rte_spinlock_recursive_lock(rte_spinlock_recursive_t *slr) {
    rte_spinlock_recursive_lock(slr);
}

void
_rte_spinlock_recursive_unlock(rte_spinlock_recursive_t *slr) {
    rte_spinlock_recursive_unlock(slr);
}

int
_rte_spinlock_recursive_trylock(rte_spinlock_recursive_t *slr) {
    return rte_spinlock_recursive_trylock(slr);
}

void
_rte_spinlock_recursive_lock_tm(rte_spinlock_recursive_t *slr) {
    rte_spinlock_recursive_lock_tm(slr);
}

void
_rte_spinlock_recursive_unlock_tm(rte_spinlock_recursive_t *slr) {
    rte_spinlock_recursive_unlock_tm(slr);
}

int
_rte_spinlock_recursive_trylock_tm(rte_spinlock_recursive_t *slr) {
    return rte_spinlock_recursive_trylock_tm(slr);
}

unsigned
_rte_lcore_id(void) {
    return rte_lcore_id();
}

int
_rte_errno(void) {
    return rte_errno;
}

uint64_t
_rte_get_tsc_cycles(void) {
    return rte_get_tsc_cycles();
}

uint64_t
_rte_get_timer_cycles(void) {
    return rte_get_timer_cycles();
}

uint64_t
_rte_get_timer_hz(void) {
    return rte_get_timer_hz();
}

void
_rte_delay_ms(unsigned ms) {
    rte_delay_ms(ms);
}

uint64_t
_rte_rdtsc(void) {
    return rte_rdtsc();
}

uint64_t
_rte_rdtsc_precise(void) {
    return rte_rdtsc_precise();
}

struct rte_mempool *
_rte_mempool_from_obj(void *obj) {
    return rte_mempool_from_obj(obj);
}

rte_iova_t
_rte_mempool_virt2iova(const void *elt) {
    return rte_mempool_virt2iova(elt);
}

void *
_rte_mempool_get_priv(struct rte_mempool *mp) {
    return rte_mempool_get_priv(mp);
}

void
_rte_mempool_cache_flush(struct rte_mempool_cache *cache, struct rte_mempool *mp) {
    rte_mempool_cache_flush(cache, mp);
}

struct rte_mempool_cache *
_rte_mempool_default_cache(struct rte_mempool *mp, unsigned lcore_id) {
    return rte_mempool_default_cache(mp, lcore_id);
}

void
_rte_mempool_generic_put(struct rte_mempool *mp, void * const *obj_table,
			unsigned int n, struct rte_mempool_cache *cache) {
    rte_mempool_generic_put(mp, obj_table, n, cache);
}

void
_rte_mempool_put_bulk(struct rte_mempool *mp, void * const *obj_table, unsigned int n) {
    rte_mempool_put_bulk(mp, obj_table, n);
}

void
_rte_mempool_put(struct rte_mempool *mp, void *obj) {
    rte_mempool_put(mp, obj);
}

int
_rte_mempool_generic_get(struct rte_mempool *mp, void **obj_table,
			unsigned int n, struct rte_mempool_cache *cache) {
    return rte_mempool_generic_get(mp, obj_table, n, cache);
}

int
_rte_mempool_get_bulk(struct rte_mempool *mp, void **obj_table, unsigned int n) {
    return rte_mempool_get_bulk(mp, obj_table, n);
}

int
_rte_mempool_get(struct rte_mempool *mp, void **obj_p) {
    return rte_mempool_get(mp, obj_p);
}

int __rte_experimental
_rte_mempool_get_contig_blocks(struct rte_mempool *mp, void **first_obj_table, unsigned int n) {
    return rte_mempool_get_contig_blocks(mp, first_obj_table, n);
}

void
_rte_mbuf_prefetch_part1(struct rte_mbuf *m) {
    rte_mbuf_prefetch_part1(m);
}

void
_rte_mbuf_prefetch_part2(struct rte_mbuf *m) {
    rte_mbuf_prefetch_part2(m);
}

rte_iova_t
_rte_mbuf_data_iova(const struct rte_mbuf *mb) {
    return rte_mbuf_data_iova(mb);
}

rte_iova_t
_rte_mbuf_data_iova_default(const struct rte_mbuf *mb) {
    return rte_mbuf_data_iova_default(mb);
}

struct rte_mbuf *
_rte_mbuf_from_indirect(struct rte_mbuf *mi) {
    return rte_mbuf_from_indirect(mi);
}

char *
_rte_mbuf_to_baddr(struct rte_mbuf *md) {
    return rte_mbuf_to_baddr(md);
}

void *
_rte_mbuf_to_priv(struct rte_mbuf *m) {
    return rte_mbuf_to_priv(m);
}

uint16_t
_rte_mbuf_refcnt_read(const struct rte_mbuf *m) {
    return rte_mbuf_refcnt_read(m);
}

void
_rte_mbuf_refcnt_set(struct rte_mbuf *m, uint16_t new_value) {
    rte_mbuf_refcnt_set(m, new_value);
}

uint16_t
_rte_mbuf_refcnt_update(struct rte_mbuf *m, int16_t value) {
    return rte_mbuf_refcnt_update(m, value);
}

uint16_t
_rte_mbuf_ext_refcnt_read(const struct rte_mbuf_ext_shared_info *shinfo) {
    return rte_mbuf_ext_refcnt_read(shinfo);
}

void
_rte_mbuf_ext_refcnt_set(struct rte_mbuf_ext_shared_info *shinfo, uint16_t new_value) {
    rte_mbuf_ext_refcnt_set(shinfo, new_value);
}

uint16_t
_rte_mbuf_ext_refcnt_update(struct rte_mbuf_ext_shared_info *shinfo, int16_t value) {
    return rte_mbuf_ext_refcnt_update(shinfo, value);
}

struct rte_mbuf *
_rte_mbuf_raw_alloc(struct rte_mempool *mp) {
    return rte_mbuf_raw_alloc(mp);
}

void
_rte_mbuf_raw_free(struct rte_mbuf *m) {
    rte_mbuf_raw_free(m);
}

uint16_t
_rte_pktmbuf_data_room_size(struct rte_mempool *mp) {
    return rte_pktmbuf_data_room_size(mp);
}

uint16_t
_rte_pktmbuf_priv_size(struct rte_mempool *mp) {
    return rte_pktmbuf_priv_size(mp);
}

void
_rte_pktmbuf_reset_headroom(struct rte_mbuf *m) {
    rte_pktmbuf_reset_headroom(m);
}

void
_rte_pktmbuf_reset(struct rte_mbuf *m) {
    rte_pktmbuf_reset(m);
}

struct rte_mbuf *
_rte_pktmbuf_alloc(struct rte_mempool *mp) {
    return rte_pktmbuf_alloc(mp);
}

int
_rte_pktmbuf_alloc_bulk(struct rte_mempool *pool, struct rte_mbuf **mbufs, unsigned count) {
    return rte_pktmbuf_alloc_bulk(pool, mbufs, count);
}

struct rte_mbuf_ext_shared_info *
_rte_pktmbuf_ext_shinfo_init_helper(void *buf_addr, uint16_t *buf_len,
	rte_mbuf_extbuf_free_callback_t free_cb, void *fcb_opaque) {
    return rte_pktmbuf_ext_shinfo_init_helper(buf_addr, buf_len, free_cb, fcb_opaque);
}

void
_rte_pktmbuf_attach_extbuf(struct rte_mbuf *m, void *buf_addr,
	rte_iova_t buf_iova, uint16_t buf_len,
	struct rte_mbuf_ext_shared_info *shinfo) {
    rte_pktmbuf_attach_extbuf(m, buf_addr, buf_iova, buf_len, shinfo);
}

void
_rte_pktmbuf_attach(struct rte_mbuf *mi, struct rte_mbuf *m) {
    rte_pktmbuf_attach(mi, m);
}

void
_rte_pktmbuf_detach(struct rte_mbuf *m) {
    rte_pktmbuf_detach(m);
}

struct rte_mbuf *
_rte_pktmbuf_prefree_seg(struct rte_mbuf *m) {
    return rte_pktmbuf_prefree_seg(m);
}

void
_rte_pktmbuf_free_seg(struct rte_mbuf *m) {
    rte_pktmbuf_free_seg(m);
}

void
_rte_pktmbuf_free(struct rte_mbuf *m) {
    rte_pktmbuf_free(m);
}

struct rte_mbuf *
_rte_pktmbuf_clone(struct rte_mbuf *md, struct rte_mempool *mp) {
    return rte_pktmbuf_clone(md, mp);
}

void
_rte_pktmbuf_refcnt_update(struct rte_mbuf *m, int16_t v) {
    rte_pktmbuf_refcnt_update(m, v);
}

uint16_t
_rte_pktmbuf_headroom(const struct rte_mbuf *m) {
    return rte_pktmbuf_headroom(m);
}

uint16_t
_rte_pktmbuf_tailroom(const struct rte_mbuf *m) {
    return rte_pktmbuf_tailroom(m);
}

struct rte_mbuf *
_rte_pktmbuf_lastseg(struct rte_mbuf *m) {
    return rte_pktmbuf_lastseg(m);
}

char *
_rte_pktmbuf_prepend(struct rte_mbuf *m, uint16_t len) {
    return rte_pktmbuf_prepend(m, len);
}

char *
_rte_pktmbuf_append(struct rte_mbuf *m, uint16_t len) {
    return rte_pktmbuf_append(m, len);
}

char *
_rte_pktmbuf_adj(struct rte_mbuf *m, uint16_t len) {
    return rte_pktmbuf_adj(m, len);
}

int
_rte_pktmbuf_trim(struct rte_mbuf *m, uint16_t len) {
    return rte_pktmbuf_trim(m, len);
}

int
_rte_pktmbuf_is_contiguous(const struct rte_mbuf *m) {
    return rte_pktmbuf_is_contiguous(m);
}

const void *
_rte_pktmbuf_read(const struct rte_mbuf *m, uint32_t off, uint32_t len, void *buf) {
    return rte_pktmbuf_read(m, off, len, buf);
}

int
_rte_pktmbuf_chain(struct rte_mbuf *head, struct rte_mbuf *tail) {
    return rte_pktmbuf_chain(head, tail);
}

int
_rte_validate_tx_offload(const struct rte_mbuf *m) {
    return rte_validate_tx_offload(m);
}

int
_rte_pktmbuf_linearize(struct rte_mbuf *mbuf) {
    return rte_pktmbuf_linearize(mbuf);
}

uint16_t
_rte_eth_rx_burst(uint16_t port_id, uint16_t queue_id,
		 struct rte_mbuf **rx_pkts, const uint16_t nb_pkts) {
    return rte_eth_rx_burst(port_id, queue_id, rx_pkts, nb_pkts);
}

int
_rte_eth_rx_queue_count(uint16_t port_id, uint16_t queue_id) {
    return rte_eth_rx_queue_count(port_id, queue_id);
}

int
_rte_eth_rx_descriptor_done(uint16_t port_id, uint16_t queue_id, uint16_t offset) {
    return rte_eth_rx_descriptor_done(port_id, queue_id, offset);
}

int
_rte_eth_rx_descriptor_status(uint16_t port_id, uint16_t queue_id, uint16_t offset) {
    return rte_eth_rx_descriptor_status(port_id, queue_id, offset);
}

int
_rte_eth_tx_descriptor_status(uint16_t port_id,	uint16_t queue_id, uint16_t offset) {
    return rte_eth_tx_descriptor_status(port_id, queue_id, offset);
}

uint16_t
_rte_eth_tx_burst(uint16_t port_id, uint16_t queue_id,
		 struct rte_mbuf **tx_pkts, uint16_t nb_pkts) {
    return rte_eth_tx_burst(port_id, queue_id, tx_pkts, nb_pkts);
}

uint16_t
_rte_eth_tx_prepare(uint16_t port_id, uint16_t queue_id,
		struct rte_mbuf **tx_pkts, uint16_t nb_pkts) {
    return rte_eth_tx_prepare(port_id, queue_id, tx_pkts, nb_pkts);
}

uint16_t
_rte_eth_tx_buffer_flush(uint16_t port_id, uint16_t queue_id, struct rte_eth_dev_tx_buffer *buffer) {
    return rte_eth_tx_buffer_flush(port_id, queue_id, buffer);
}

uint16_t
_rte_eth_tx_buffer(uint16_t port_id, uint16_t queue_id,
		struct rte_eth_dev_tx_buffer *buffer, struct rte_mbuf *tx_pkt) {
    return rte_eth_tx_buffer(port_id, queue_id, buffer, tx_pkt);
}

int
_rte_vlan_strip(struct rte_mbuf *m) {
    return rte_vlan_strip(m);
}

int
_rte_vlan_insert(struct rte_mbuf **m) {
    return rte_vlan_insert(m);
}
