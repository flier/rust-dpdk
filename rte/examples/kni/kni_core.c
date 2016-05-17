#include <stdio.h>
#include <stdlib.h>

#include <rte_config.h>
#include <rte_common.h>
#include <rte_cycles.h>
#include <rte_prefetch.h>
#include <rte_lcore.h>
#include <rte_ether.h>
#include <rte_ethdev.h>
#include <rte_mbuf.h>

#define KNI_MAX_KTHREAD 32

/*
 * Structure of port parameters
 */
struct kni_port_params {
    uint8_t port_id;/* Port ID */
    unsigned lcore_rx; /* lcore ID for RX */
    unsigned lcore_tx; /* lcore ID for TX */
    uint32_t nb_lcore_k; /* Number of lcores for KNI multi kernel threads */
    uint32_t nb_kni; /* Number of KNI devices to be created */
    unsigned lcore_k[KNI_MAX_KTHREAD]; /* lcore ID list for kthreads */
    struct rte_kni *kni[KNI_MAX_KTHREAD]; /* KNI context pointers */
} __rte_cache_aligned;

struct kni_port_params *kni_port_params_array[RTE_MAX_ETHPORTS];

int kni_stop = 0;

int kni_main_loop() {
    return 0;
}
