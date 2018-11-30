#include <stdio.h>
#include <stdlib.h>

#include <rte_config.h>
#include <rte_common.h>
#include <rte_log.h>
#include <rte_cycles.h>
#include <rte_prefetch.h>
#include <rte_lcore.h>
#include <rte_ether.h>
#include <rte_ethdev.h>
#include <rte_mbuf.h>
#include <rte_kni.h>

/* Macros for printing using RTE_LOG */
#define RTE_LOGTYPE_APP RTE_LOGTYPE_USER1

/* How many packets to attempt to read from NIC in one go */
#define PKT_BURST_SZ 32

/* How many objects (mbufs) to keep in per-lcore mempool cache */
#define MEMPOOL_CACHE_SZ PKT_BURST_SZ

#define KNI_MAX_KTHREAD 32

/*
 * Structure of port parameters
 */
struct kni_port_params
{
    uint8_t port_id;                      /* Port ID */
    unsigned lcore_rx;                    /* lcore ID for RX */
    unsigned lcore_tx;                    /* lcore ID for TX */
    uint32_t nb_lcore_k;                  /* Number of lcores for KNI multi kernel threads */
    uint32_t nb_kni;                      /* Number of KNI devices to be created */
    unsigned lcore_k[KNI_MAX_KTHREAD];    /* lcore ID list for kthreads */
    struct rte_kni *kni[KNI_MAX_KTHREAD]; /* KNI context pointers */
} __rte_cache_aligned;

struct kni_port_params **kni_port_params_array;

/* Structure type for recording kni interface specific stats */
struct kni_interface_stats
{
    /* number of pkts received from NIC, and sent to KNI */
    uint64_t rx_packets;

    /* number of pkts received from NIC, but failed to send to KNI */
    uint64_t rx_dropped;

    /* number of pkts received from KNI, and sent to NIC */
    uint64_t tx_packets;

    /* number of pkts received from KNI, but failed to send to NIC */
    uint64_t tx_dropped;
};

/* kni device statistics array */
struct kni_interface_stats kni_stats[RTE_MAX_ETHPORTS];

int kni_stop = 0;

/* Print out statistics on packets handled */
void kni_print_stats(void)
{
    uint8_t i;

    printf("\n**KNI example application statistics**\n"
           "======  ==============  ============  ============  ============  ============\n"
           " Port    Lcore(RX/TX)    rx_packets    rx_dropped    tx_packets    tx_dropped\n"
           "------  --------------  ------------  ------------  ------------  ------------\n");
    for (i = 0; i < RTE_MAX_ETHPORTS; i++)
    {
        if (!kni_port_params_array[i])
            continue;

        printf("%7d %10u/%2u %13" PRIu64 " %13" PRIu64 " %13" PRIu64 " "
               "%13" PRIu64 "\n",
               i,
               kni_port_params_array[i]->lcore_rx,
               kni_port_params_array[i]->lcore_tx,
               kni_stats[i].rx_packets,
               kni_stats[i].rx_dropped,
               kni_stats[i].tx_packets,
               kni_stats[i].tx_dropped);
    }
    printf("======  ==============  ============  ============  ============  ============\n");
}

static void
kni_burst_free_mbufs(struct rte_mbuf **pkts, unsigned num)
{
    unsigned i;

    if (pkts == NULL)
        return;

    for (i = 0; i < num; i++)
    {
        rte_pktmbuf_free(pkts[i]);
        pkts[i] = NULL;
    }
}

/**
 * Interface to burst rx and enqueue mbufs into rx_q
 */
int kni_ingress(struct kni_port_params *p)
{
    uint8_t i, port_id;
    unsigned nb_rx, num;
    uint32_t nb_kni;
    struct rte_mbuf *pkts_burst[PKT_BURST_SZ];

    if (p == NULL)
        return 0;

    nb_kni = p->nb_kni;
    port_id = p->port_id;

    while (!kni_stop)
    {
        for (i = 0; i < nb_kni; i++)
        {
            /* Burst rx from eth */
            nb_rx = rte_eth_rx_burst(port_id, 0, pkts_burst, PKT_BURST_SZ);
            if (unlikely(nb_rx > PKT_BURST_SZ))
            {
                RTE_LOG(ERR, APP, "Error receiving from eth\n");
                return -1;
            }
            /* Burst tx to kni */
            num = rte_kni_tx_burst(p->kni[i], pkts_burst, nb_rx);
            kni_stats[port_id].rx_packets += num;

            rte_kni_handle_request(p->kni[i]);
            if (unlikely(num < nb_rx))
            {
                /* Free mbufs not tx to kni interface */
                kni_burst_free_mbufs(&pkts_burst[num], nb_rx - num);
                kni_stats[port_id].rx_dropped += nb_rx - num;
            }
        }
    }

    return 0;
}

/**
 * Interface to dequeue mbufs from tx_q and burst tx
 */
int kni_egress(struct kni_port_params *p)
{
    uint8_t i, port_id;
    unsigned nb_tx, num;
    uint32_t nb_kni;
    struct rte_mbuf *pkts_burst[PKT_BURST_SZ];

    if (p == NULL)
        return -1;

    nb_kni = p->nb_kni;
    port_id = p->port_id;

    while (!kni_stop)
    {
        for (i = 0; i < nb_kni; i++)
        {
            /* Burst rx from kni */
            num = rte_kni_rx_burst(p->kni[i], pkts_burst, PKT_BURST_SZ);
            if (unlikely(num > PKT_BURST_SZ))
            {
                RTE_LOG(ERR, APP, "Error receiving from KNI\n");
                return -1;
            }
            /* Burst tx to eth */
            nb_tx = rte_eth_tx_burst(port_id, 0, pkts_burst, (uint16_t)num);
            kni_stats[port_id].tx_packets += nb_tx;
            if (unlikely(nb_tx < num))
            {
                /* Free mbufs not tx to NIC */
                kni_burst_free_mbufs(&pkts_burst[nb_tx], num - nb_tx);
                kni_stats[port_id].tx_dropped += num - nb_tx;
            }
        }
    }

    return 0;
}
