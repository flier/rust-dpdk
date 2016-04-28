#include <rte_config.h>
#include <rte_lcore.h>
#include <rte_errno.h>

unsigned _rte_lcore_id() {
    return rte_lcore_id();
}

int _rte_errno() {
    return rte_errno;
}

size_t _rte_cache_line_size() {
    return RTE_CACHE_LINE_SIZE;
}
