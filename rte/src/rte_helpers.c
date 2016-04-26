#include <rte_config.h>
#include <rte_lcore.h>

unsigned _rte_lcore_id() {
    return rte_lcore_id();
}
