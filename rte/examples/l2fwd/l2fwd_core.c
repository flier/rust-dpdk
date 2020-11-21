#include <stdio.h>
#include <stdlib.h>

#include <net/ethernet.h>

#include <rte_config.h>
#include <rte_common.h>
#include <rte_cycles.h>
#include <rte_prefetch.h>
#include <rte_lcore.h>
#include <rte_ether.h>
#include <rte_ethdev.h>
#include <rte_mbuf.h>

#define MAX_PKT_BURST 32
#define BURST_TX_DRAIN_US 100 /* TX drain every ~100us */

int l2fwd_force_quit = 0;

/* mask of enabled ports */
uint32_t l2fwd_enabled_port_mask = 0;

/* ethernet addresses of ports */
struct ether_addr l2fwd_ports_eth_addr[RTE_MAX_ETHPORTS];

/* list of enabled ports */
uint32_t l2fwd_dst_ports[RTE_MAX_ETHPORTS];

struct rte_eth_dev_tx_buffer *l2fwd_tx_buffers[RTE_MAX_ETHPORTS];

/* Per-port statistics struct */
struct l2fwd_port_statistics
{
    uint64_t tx;
    uint64_t rx;
    uint64_t dropped;
} __rte_cache_aligned;

struct l2fwd_port_statistics port_statistics[RTE_MAX_ETHPORTS];

int64_t l2fwd_timer_period; /* default period is 10 seconds */

/* Print out statistics on packets dropped */
static void
print_stats(void)
{
    uint64_t total_packets_dropped, total_packets_tx, total_packets_rx;
    unsigned portid;

    total_packets_dropped = 0;
    total_packets_tx = 0;
    total_packets_rx = 0;

    const char clr[] = {27, '[', '2', 'J', '\0'};
    const char topLeft[] = {27, '[', '1', ';', '1', 'H', '\0'};

    /* Clear screen and move to top left */
    printf("%s%s", clr, topLeft);

    printf("\nPort statistics ====================================");

    for (portid = 0; portid < RTE_MAX_ETHPORTS; portid++)
    {
        /* skip disabled ports */
        if ((l2fwd_enabled_port_mask & (1 << portid)) == 0)
            continue;

        printf("\nStatistics for port %u ------------------------------"
               "\nPackets sent: %24" PRIu64
               "\nPackets received: %20" PRIu64
               "\nPackets dropped: %21" PRIu64,
               portid,
               port_statistics[portid].tx,
               port_statistics[portid].rx,
               port_statistics[portid].dropped);

        total_packets_dropped += port_statistics[portid].dropped;
        total_packets_tx += port_statistics[portid].tx;
        total_packets_rx += port_statistics[portid].rx;
    }
    printf("\nAggregate statistics ==============================="
           "\nTotal packets sent: %18" PRIu64
           "\nTotal packets received: %14" PRIu64
           "\nTotal packets dropped: %15" PRIu64,
           total_packets_tx,
           total_packets_rx,
           total_packets_dropped);
    printf("\n====================================================\n");
}

static void
l2fwd_simple_forward(struct rte_mbuf *m, unsigned portid)
{
    struct rte_ether_hdr *eth;
    void *tmp;
    unsigned dst_port;
    int sent;
    struct rte_eth_dev_tx_buffer *buffer;

    dst_port = l2fwd_dst_ports[portid];
    eth = rte_pktmbuf_mtod(m, struct ether_hdr *);

    /* 02:00:00:00:00:xx */
    tmp = &eth->d_addr.addr_bytes[0];
    *((uint64_t *)tmp) = 0x000000000002 + ((uint64_t)dst_port << 40);

    /* src addr */
    rte_ether_addr_copy(&l2fwd_ports_eth_addr[dst_port], &eth->s_addr);

    buffer = l2fwd_tx_buffers[dst_port];
    sent = rte_eth_tx_buffer(dst_port, 0, buffer, m);
    if (sent)
        port_statistics[dst_port].tx += sent;
}

int l2fwd_main_loop(uint32_t *rx_port_list, unsigned n_rx_port)
{
    unsigned lcore_id = rte_lcore_id();
    uint64_t prev_tsc = 0, diff_tsc, cur_tsc, timer_tsc = 0;
    const uint64_t drain_tsc = (rte_get_tsc_hz() + US_PER_S - 1) / US_PER_S * BURST_TX_DRAIN_US;
    unsigned portid, nb_rx;
    struct rte_eth_dev_tx_buffer *buffer;
    struct rte_mbuf *pkts_burst[MAX_PKT_BURST], *m;
    int sent, i, j;

    while (!l2fwd_force_quit)
    {
        cur_tsc = rte_rdtsc();

        /*
         * TX burst queue drain
         */
        diff_tsc = cur_tsc - prev_tsc;

        if (unlikely(diff_tsc > drain_tsc))
        {
            for (i = 0; i < (int)n_rx_port; i++)
            {

                portid = l2fwd_dst_ports[rx_port_list[i]];
                buffer = l2fwd_tx_buffers[portid];

                sent = rte_eth_tx_buffer_flush(portid, 0, buffer);
                if (sent)
                    port_statistics[portid].tx += sent;
            }

            /* if timer is enabled */
            if (l2fwd_timer_period > 0)
            {

                /* advance the timer */
                timer_tsc += diff_tsc;

                /* if timer has reached its timeout */
                if (unlikely(timer_tsc >= (uint64_t)l2fwd_timer_period))
                {

                    /* do this only on master core */
                    if (lcore_id == rte_get_master_lcore())
                    {
                        print_stats();
                        /* reset the timer */
                        timer_tsc = 0;
                    }
                }
            }

            prev_tsc = cur_tsc;
        }

        /*
         * Read packet from RX queues
         */
        for (i = 0; i < (int)n_rx_port; i++)
        {

            portid = rx_port_list[i];
            nb_rx = rte_eth_rx_burst((uint8_t)portid, 0, pkts_burst, MAX_PKT_BURST);

            port_statistics[portid].rx += nb_rx;

            for (j = 0; j < (int)nb_rx; j++)
            {
                m = pkts_burst[j];
                rte_prefetch0(rte_pktmbuf_mtod(m, void *));
                l2fwd_simple_forward(m, portid);
            }
        }
    }

    return 0;
}
