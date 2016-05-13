use std::ffi::CString;

use ffi;

use errors::{Error, Result};
use mempool;

// Packet Offload Features Flags. It also carry packet type information.
// Critical resources. Both rx/tx shared these bits. Be cautious on any change
//
// - RX flags start at bit position zero, and get added to the left of previous
//   flags.
// - The most-significant 3 bits are reserved for generic mbuf flags
// - TX flags therefore start at bit position 60 (i.e. 63-3), and new flags get
//   added to the right of the previously defined flags i.e. they should count
//   downwards, not upwards.
//
// Keep these flags synchronized with rte_get_rx_ol_flag_name() and
// rte_get_tx_ol_flag_name().
//
bitflags! {
    pub flags OffloadFlags: u64 {
        /// RX packet is a 802.1q VLAN packet.
        const PKT_RX_VLAN_PKT      = 1 << 0,
        /// RX packet with RSS hash result.
        const PKT_RX_RSS_HASH      = 1 << 1,
        /// RX packet with FDIR match indicate.
        const PKT_RX_FDIR          = 1 << 2,
        /// L4 cksum of RX pkt. is not OK.
        const PKT_RX_L4_CKSUM_BAD  = 1 << 3,
        /// IP cksum of RX pkt. is not OK.
        const PKT_RX_IP_CKSUM_BAD  = 1 << 4,
        /// External IP header checksum error.
        const PKT_RX_EIP_CKSUM_BAD = 1 << 5,
        /// Num of desc of an RX pkt oversize.
        const PKT_RX_OVERSIZE      = 0 << 0,
        /// Header buffer overflow.
        const PKT_RX_HBUF_OVERFLOW = 0 << 0,
        /// Hardware processing error.
        const PKT_RX_RECIP_ERR     = 0 << 0,
        /// MAC error.
        const PKT_RX_MAC_ERR       = 0 << 0,
        /// RX IEEE1588 L2 Ethernet PT Packet.
        const PKT_RX_IEEE1588_PTP  = 1 << 9,
        /// RX IEEE1588 L2/L4 timestamped packet.
        const PKT_RX_IEEE1588_TMST = 1 << 10,
        /// FD id reported if FDIR match.
        const PKT_RX_FDIR_ID       = 1 << 13,
        /// Flexible bytes reported if FDIR match.
        const PKT_RX_FDIR_FLX      = 1 << 14,
        /// RX packet with double VLAN stripped.
        const PKT_RX_QINQ_PKT      = 1 << 15,

        /// TX packet with double VLAN inserted.
        const PKT_TX_QINQ_PKT      = 1 << 49,

        /**
         * TCP segmentation offload. To enable this offload feature for a
         * packet to be transmitted on hardware supporting TSO:
         *  - set the PKT_TX_TCP_SEG flag in mbuf->ol_flags (this flag implies
         *    PKT_TX_TCP_CKSUM)
         *  - set the flag PKT_TX_IPV4 or PKT_TX_IPV6
         *  - if it's IPv4, set the PKT_TX_IP_CKSUM flag and write the IP checksum
         *    to 0 in the packet
         *  - fill the mbuf offload information: l2_len, l3_len, l4_len, tso_segsz
         *  - calculate the pseudo header checksum without taking ip_len in account,
         *    and set it in the TCP header. Refer to rte_ipv4_phdr_cksum() and
         *    rte_ipv6_phdr_cksum() that can be used as helpers.
         */
        const PKT_TX_TCP_SEG       = 1 << 50,

        const PKT_TX_IEEE1588_TMST = 1 << 51, /**< TX IEEE1588 packet to timestamp. */

        /**
         * Bits 52+53 used for L4 packet type with checksum enabled: 00: Reserved,
         * 01: TCP checksum, 10: SCTP checksum, 11: UDP checksum. To use hardware
         * L4 checksum offload, the user needs to:
         *  - fill l2_len and l3_len in mbuf
         *  - set the flags PKT_TX_TCP_CKSUM, PKT_TX_SCTP_CKSUM or PKT_TX_UDP_CKSUM
         *  - set the flag PKT_TX_IPV4 or PKT_TX_IPV6
         *  - calculate the pseudo header checksum and set it in the L4 header (only
         *    for TCP or UDP). See rte_ipv4_phdr_cksum() and rte_ipv6_phdr_cksum().
         *    For SCTP, set the crc field to 0.
         */
        const PKT_TX_L4_NO_CKSUM   = 0 << 52, /**< Disable L4 cksum of TX pkt. */
        const PKT_TX_TCP_CKSUM     = 1 << 52, /**< TCP cksum of TX pkt. computed by NIC. */
        const PKT_TX_SCTP_CKSUM    = 2 << 52, /**< SCTP cksum of TX pkt. computed by NIC. */
        const PKT_TX_UDP_CKSUM     = 3 << 52, /**< UDP cksum of TX pkt. computed by NIC. */
        const PKT_TX_L4_MASK       = 3 << 52, /**< Mask for L4 cksum offload request. */

        /**
         * Offload the IP checksum in the hardware. The flag PKT_TX_IPV4 should
         * also be set by the application, although a PMD will only check
         * PKT_TX_IP_CKSUM.
         *  - set the IP checksum field in the packet to 0
         *  - fill the mbuf offload information: l2_len, l3_len
         */
        const PKT_TX_IP_CKSUM      = 1 << 54,

        /**
         * Packet is IPv4. This flag must be set when using any offload feature
         * (TSO, L3 or L4 checksum) to tell the NIC that the packet is an IPv4
         * packet. If the packet is a tunneled packet, this flag is related to
         * the inner headers.
         */
        const PKT_TX_IPV4          = 1 << 55,

        /**
         * Packet is IPv6. This flag must be set when using an offload feature
         * (TSO or L4 checksum) to tell the NIC that the packet is an IPv6
         * packet. If the packet is a tunneled packet, this flag is related to
         * the inner headers.
         */
        const PKT_TX_IPV6          = 1 << 56,

        const PKT_TX_VLAN_PKT      = 1 << 57, /**< TX packet is a 802.1q VLAN packet. */

        /**
         * Offload the IP checksum of an external header in the hardware. The
         * flag PKT_TX_OUTER_IPV4 should also be set by the application, alto ugh
         * a PMD will only check PKT_TX_IP_CKSUM.  The IP checksum field in the
         * packet must be set to 0.
         *  - set the outer IP checksum field in the packet to 0
         *  - fill the mbuf offload information: outer_l2_len, outer_l3_len
         */
        const PKT_TX_OUTER_IP_CKSUM   = 1 << 58,

        /**
         * Packet outer header is IPv4. This flag must be set when using any
         * outer offload feature (L3 or L4 checksum) to tell the NIC that the
         * outer header of the tunneled packet is an IPv4 packet.
         */
        const PKT_TX_OUTER_IPV4   = 1 << 59,

        /**
         * Packet outer header is IPv6. This flag must be set when using any
         * outer offload feature (L4 checksum) to tell the NIC that the outer
         * header of the tunneled packet is an IPv6 packet.
         */
        const PKT_TX_OUTER_IPV6    = 1 << 60,
        /// reserved for future mbuf use
        const __RESERVED           = 1 << 61,
        /// Indirect attached mbuf
        const IND_ATTACHED_MBUF    = 1 << 62,

        /// Use final bit of flags to indicate a control mbuf
        ///
        /// Mbuf contains control data
        const CTRL_MBUF_FLAG       = 1 << 63,
    }
}

/**
 * Some NICs need at least 2KB buffer to RX standard Ethernet frame without
 * splitting it into multiple segments.
 * So, for mbufs that planned to be involved into RX/TX, the recommended
 * minimal buffer length is 2KB + RTE_PKTMBUF_HEADROOM.
 */
pub const RTE_MBUF_DEFAULT_BUF_SIZE: u16 =
    (ffi::RTE_MBUF_DEFAULT_DATAROOM + ffi::RTE_PKTMBUF_HEADROOM) as u16;


/// Create a mbuf pool.
///
/// This function creates and initializes a packet mbuf pool.
/// It is a wrapper to rte_mempool_create() with the proper packet constructor
/// and mempool constructor.
pub fn pktmbuf_pool_create(name: &str,
                           n: u32,
                           cache_size: u32,
                           priv_size: u16,
                           data_room_size: u16,
                           socket_id: i32)
                           -> Result<mempool::RawMemoryPool> {
    let name = try!(CString::new(name))
                   .as_bytes_with_nul()
                   .as_ptr() as *const i8;

    let p = unsafe {
        ffi::rte_pktmbuf_pool_create(name, n, cache_size, priv_size, data_room_size, socket_id)
    };

    if p.is_null() {
        Err(Error::rte_error())
    } else {
        Ok(mempool::from_raw(p))
    }
}
