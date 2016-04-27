#include <rte_config.h>
#include <rte_lcore.h>
#include <rte_errno.h>

unsigned _rte_lcore_id() {
    return rte_lcore_id();
}

int _rte_errno() {
    return rte_errno;
}
