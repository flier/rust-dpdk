#pragma clang diagnostic ignored "-Wdeprecated-register"

#include <rte_config.h>

// common
#include <rte_alarm.h>

// eal
#include <rte_atomic.h>
#include <rte_byteorder.h>
#include <rte_common.h>
#include <rte_cpuflags.h>
#include <rte_cycles.h>
#include <rte_debug.h>
#include <rte_dev.h>
#include <rte_devargs.h>
#include <rte_eal_memconfig.h>
#include <rte_eal.h>
#include <rte_errno.h>
#include <rte_hexdump.h>
#include <rte_interrupts.h>
#include <rte_keepalive.h>

#include <rte_launch.h>
#include <rte_lcore.h>
#include <rte_per_lcore.h>

#include <rte_log.h>
#include <rte_malloc.h>
#include <rte_malloc_heap.h>
#include <rte_memory.h>
#include <rte_memcpy.h>
#include <rte_memzone.h>
#include <rte_pci.h>
#include <rte_prefetch.h>
#include <rte_random.h>
#include <rte_spinlock.h>
#include <rte_string_fns.h>
#include <rte_timer.h>
#include <rte_version.h>

// acl
#include <rte_acl.h>

// bond
#include <rte_eth_bond.h>

// config
#include <rte_cfgfile.h>

// cmdline
#include <cmdline_parse_etheraddr.h>
#include <cmdline_parse_ipaddr.h>
#include <cmdline_parse_num.h>
#include <cmdline_parse_portlist.h>
#include <cmdline_parse_string.h>
#include <cmdline_socket.h>
#include <cmdline.h>

// distributor
#include <rte_distributor.h>

// ether
#include <rte_ethdev.h>

// hash
#include <rte_hash.h>

// IP fragment
#include <rte_ip_frag.h>

// KNI
#include <rte_kni.h>

// LPM
#include <rte_lpm.h>
#include <rte_lpm6.h>
#include <rte_lpm_sse.h>

// mbuf
#include <rte_mbuf.h>

// mempool
#include <rte_mempool.h>

// meter
#include <rte_meter.h>

// net
#include <rte_ether.h>
#include <rte_arp.h>
#include <rte_ip.h>
#include <rte_icmp.h>
#include <rte_gre.h>
#include <rte_tcp.h>
#include <rte_udp.h>
#include <rte_sctp.h>
#include <rte_net.h>

// packet dump
#include <rte_pdump.h>

// pipeline
#include <rte_pipeline.h>

// power
#include <rte_power.h>

// reorder
#include <rte_reorder.h>

// ring
#include <rte_ring.h>

// scheduler
#include <rte_sched.h>
#include <rte_approx.h>
#include <rte_bitmap.h>
#include <rte_reciprocal.h>
#include <rte_red.h>

// table
#include <rte_table.h>
#include <rte_table_acl.h>
#include <rte_table_array.h>
#include <rte_table_hash.h>
#include <rte_table_lpm.h>

// timer
#include <rte_timer.h>

