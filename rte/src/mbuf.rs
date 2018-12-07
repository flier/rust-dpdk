use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::os::unix::io::AsRawFd;
use std::ptr::NonNull;
use std::slice;

use cfile;

use ffi;

use errors::{AsResult, Result};
use mempool;
use utils::{AsRaw, FromRaw};

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
    pub struct OffloadFlags: u64 {
        /// RX packet is a 802.1q VLAN packet.
        const PKT_RX_VLAN_PKT      = ffi::PKT_RX_VLAN as u64;
        /// RX packet with RSS hash result.
        const PKT_RX_RSS_HASH      = ffi::PKT_RX_RSS_HASH as u64;
        /// RX packet with FDIR match indicate.
        const PKT_RX_FDIR          = ffi::PKT_RX_FDIR as u64;
        /// External IP header checksum error.
        const PKT_RX_EIP_CKSUM_BAD = ffi::PKT_RX_EIP_CKSUM_BAD as u64;
        /// A vlan has been stripped by the hardware and its tci is saved in mbuf->vlan_tci.
        /// This can only happen if vlan stripping is enabled in the RX configuration of the PMD.
        const PKT_RX_VLAN_STRIPPED = ffi::PKT_RX_VLAN_STRIPPED as u64;

        /// Mask of bits used to determine the status of RX IP checksum.
        const PKT_RX_IP_CKSUM_MASK    = ffi::PKT_RX_IP_CKSUM_MASK as u64;
        /// no information about the RX IP checksum
        const PKT_RX_IP_CKSUM_UNKNOWN = ffi::PKT_RX_IP_CKSUM_UNKNOWN as u64;
        /// the IP checksum in the packet is wrong
        const PKT_RX_IP_CKSUM_BAD     = ffi::PKT_RX_IP_CKSUM_BAD as u64;
        /// the IP checksum in the packet is valid
        const PKT_RX_IP_CKSUM_GOOD    = ffi::PKT_RX_IP_CKSUM_GOOD as u64;
        /// the IP checksum is not correct in the packet data,
        /// but the integrity of the IP header is verified.
        const PKT_RX_IP_CKSUM_NONE    = ffi::PKT_RX_IP_CKSUM_NONE as u64;

        /// Mask of bits used to determine the status of RX IP checksum.
        const PKT_RX_L4_CKSUM_MASK      = ffi::PKT_RX_L4_CKSUM_MASK as u64;
        /// no information about the RX L4 checksum
        const PKT_RX_L4_CKSUM_UNKNOWN   = ffi::PKT_RX_L4_CKSUM_UNKNOWN as u64;
        /// the L4 checksum in the packet is wrong
        const PKT_RX_L4_CKSUM_BAD       = ffi::PKT_RX_L4_CKSUM_BAD as u64;
        /// the L4 checksum in the packet is valid
        const PKT_RX_L4_CKSUM_GOOD      = ffi::PKT_RX_L4_CKSUM_GOOD as u64;
        /// the L4 checksum is not correct in the packet data,
        /// but the integrity of the IP header is verified.
        const PKT_RX_L4_CKSUM_NONE      = ffi::PKT_RX_L4_CKSUM_NONE as u64;

        /// RX IEEE1588 L2 Ethernet PT Packet.
        const PKT_RX_IEEE1588_PTP  = ffi::PKT_RX_IEEE1588_PTP as u64;
        /// RX IEEE1588 L2/L4 timestamped packet.
        const PKT_RX_IEEE1588_TMST = ffi::PKT_RX_IEEE1588_TMST as u64;
        /// FD id reported if FDIR match.
        const PKT_RX_FDIR_ID       = ffi::PKT_RX_FDIR_ID as u64;
        /// Flexible bytes reported if FDIR match.
        const PKT_RX_FDIR_FLX      = ffi::PKT_RX_FDIR_FLX as u64;
        /// RX packet with double VLAN stripped.
        const PKT_RX_QINQ_STRIPPED = ffi::PKT_RX_QINQ_STRIPPED as u64;

        /// When packets are coalesced by a hardware or virtual driver, this flag
        /// can be set in the RX mbuf, meaning that the m->tso_segsz field is
        /// valid and is set to the segment size of original packets.
        const PKT_RX_LRO = ffi::PKT_RX_LRO as u64;

        /// Indicate that the timestamp field in the mbuf is valid.
        const PKT_RX_TIMESTAMP = ffi::PKT_RX_TIMESTAMP as u64;

        /// Indicate that security offload processing was applied on the RX packet.
        const PKT_RX_SEC_OFFLOAD = ffi::PKT_RX_SEC_OFFLOAD as u64;

        /// Indicate that security offload processing failed on the RX packet.
        const PKT_RX_SEC_OFFLOAD_FAILED = ffi::PKT_RX_SEC_OFFLOAD_FAILED as u64;

        /// The RX packet is a double VLAN, and the outer tci has been
        /// saved in in mbuf->vlan_tci_outer. If PKT_RX_QINQ set, PKT_RX_VLAN
        /// also should be set and inner tci should be saved to mbuf->vlan_tci.
        /// If the flag PKT_RX_QINQ_STRIPPED is also present, both VLANs
        /// headers have been stripped from mbuf data, else they are still present.
        const PKT_RX_QINQ = ffi::PKT_RX_QINQ as u64;

        /// Mask of bits used to determine the status of outer RX L4 checksum.
        const PKT_RX_OUTER_L4_CKSUM_MASK	= ffi::PKT_RX_OUTER_L4_CKSUM_MASK as u64;
        /// no info about the outer RX L4 checksum
        const PKT_RX_OUTER_L4_CKSUM_UNKNOWN = ffi::PKT_RX_OUTER_L4_CKSUM_UNKNOWN as u64;
        /// the outer L4 checksum in the packet is wrong
        const PKT_RX_OUTER_L4_CKSUM_BAD	    = ffi::PKT_RX_OUTER_L4_CKSUM_BAD as u64;
        /// the outer L4 checksum in the packet is valid
        const PKT_RX_OUTER_L4_CKSUM_GOOD    = ffi::PKT_RX_OUTER_L4_CKSUM_GOOD as u64;
        /// invalid outer L4 checksum state.
        const PKT_RX_OUTER_L4_CKSUM_INVALID	= ffi::PKT_RX_OUTER_L4_CKSUM_INVALID as u64;

        /// Indicate that the metadata field in the mbuf is in use.
        const PKT_TX_METADATA = ffi::PKT_TX_METADATA as u64;

        /**
         * Outer UDP checksum offload flag. This flag is used for enabling
         * outer UDP checksum in PMD. To use outer UDP checksum, the user needs to
         * 1) Enable the following in mbuff,
         * a) Fill outer_l2_len and outer_l3_len in mbuf.
         * b) Set the PKT_TX_OUTER_UDP_CKSUM flag.
         * c) Set the PKT_TX_OUTER_IPV4 or PKT_TX_OUTER_IPV6 flag.
         * 2) Configure DEV_TX_OFFLOAD_OUTER_UDP_CKSUM offload flag.
         */
        const PKT_TX_OUTER_UDP_CKSUM = ffi::PKT_TX_OUTER_UDP_CKSUM as u64;

        /**
         * UDP Fragmentation Offload flag. This flag is used for enabling UDP
         * fragmentation in SW or in HW. When use UFO, mbuf->tso_segsz is used
         * to store the MSS of UDP fragments.
         */
        const PKT_TX_UDP_SEG = ffi::PKT_TX_UDP_SEG as u64;

        /// Request security offload processing on the TX packet.
        const PKT_TX_SEC_OFFLOAD = ffi::PKT_TX_SEC_OFFLOAD as u64;

        /// Offload the MACsec. This flag must be set by the application to enable
        /// this offload feature for a packet to be transmitted.
        const PKT_TX_MACSEC = ffi::PKT_TX_MACSEC as u64;

        /**
         * Bits 45:48 used for the tunnel type.
         * The tunnel type must be specified for TSO or checksum on the inner part
         * of tunnel packets.
         * These flags can be used with PKT_TX_TCP_SEG for TSO, or PKT_TX_xxx_CKSUM.
         * The mbuf fields for inner and outer header lengths are required:
         * outer_l2_len, outer_l3_len, l2_len, l3_len, l4_len and tso_segsz for TSO.
         */
        const PKT_TX_TUNNEL_VXLAN  = ffi::PKT_TX_TUNNEL_VXLAN as u64;
        const PKT_TX_TUNNEL_GRE    = ffi::PKT_TX_TUNNEL_GRE as u64;
        const PKT_TX_TUNNEL_IPIP   = ffi::PKT_TX_TUNNEL_IPIP as u64;
        const PKT_TX_TUNNEL_GENEVE = ffi::PKT_TX_TUNNEL_GENEVE as u64;
        /** TX packet with MPLS-in-UDP RFC 7510 header. */
        const PKT_TX_TUNNEL_MPLSINUDP = ffi::PKT_TX_TUNNEL_MPLSINUDP as u64;
        const PKT_TX_TUNNEL_VXLAN_GPE = ffi::PKT_TX_TUNNEL_VXLAN_GPE as u64;
        /**
         * Generic IP encapsulated tunnel type, used for TSO and checksum offload.
         * It can be used for tunnels which are not standards or listed above.
         * It is preferred to use specific tunnel flags like PKT_TX_TUNNEL_GRE
         * or PKT_TX_TUNNEL_IPIP if possible.
         * The ethdev must be configured with DEV_TX_OFFLOAD_IP_TNL_TSO.
         * Outer and inner checksums are done according to the existing flags like
         * PKT_TX_xxx_CKSUM.
         * Specific tunnel headers that contain payload length, sequence id
         * or checksum are not expected to be updated.
         */
        const PKT_TX_TUNNEL_IP = ffi::PKT_TX_TUNNEL_IP as u64;
        /**
         * Generic UDP encapsulated tunnel type, used for TSO and checksum offload.
         * UDP tunnel type implies outer IP layer.
         * It can be used for tunnels which are not standards or listed above.
         * It is preferred to use specific tunnel flags like PKT_TX_TUNNEL_VXLAN
         * if possible.
         * The ethdev must be configured with DEV_TX_OFFLOAD_UDP_TNL_TSO.
         * Outer and inner checksums are done according to the existing flags like
         * PKT_TX_xxx_CKSUM.
         * Specific tunnel headers that contain payload length, sequence id
         * or checksum are not expected to be updated.
         */
        const PKT_TX_TUNNEL_UDP = ffi::PKT_TX_TUNNEL_UDP as u64;

        const PKT_TX_TUNNEL_MASK = ffi::PKT_TX_TUNNEL_MASK as u64;

        /// TX packet with double VLAN inserted.
        const PKT_TX_QINQ = ffi::PKT_TX_QINQ as u64;

        /// TX packet with double VLAN inserted.
        const PKT_TX_QINQ_PKT = ffi::PKT_TX_QINQ_PKT as u64;

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
        const PKT_TX_TCP_SEG = ffi::PKT_TX_TCP_SEG as u64;

        /// TX IEEE1588 packet to timestamp.
        const PKT_TX_IEEE1588_TMST = ffi::PKT_TX_IEEE1588_TMST as u64;

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

        /// Disable L4 cksum of TX pkt.
        const PKT_TX_L4_NO_CKSUM   = ffi::PKT_TX_L4_NO_CKSUM as u64;
        /// TCP cksum of TX pkt. computed by NIC.
        const PKT_TX_TCP_CKSUM     = ffi::PKT_TX_TCP_CKSUM;
        /// SCTP cksum of TX pkt. computed by NIC.
        const PKT_TX_SCTP_CKSUM    = ffi::PKT_TX_SCTP_CKSUM;
        /// UDP cksum of TX pkt. computed by NIC.
        const PKT_TX_UDP_CKSUM     = ffi::PKT_TX_UDP_CKSUM;
        /// Mask for L4 cksum offload request.
        const PKT_TX_L4_MASK       = ffi::PKT_TX_L4_MASK;

        /**
         * Offload the IP checksum in the hardware. The flag PKT_TX_IPV4 should
         * also be set by the application, although a PMD will only check
         * PKT_TX_IP_CKSUM.
         *  - set the IP checksum field in the packet to 0
         *  - fill the mbuf offload information: l2_len, l3_len
         */
        const PKT_TX_IP_CKSUM = ffi::PKT_TX_IP_CKSUM;

        /**
         * Packet is IPv4. This flag must be set when using any offload feature
         * (TSO, L3 or L4 checksum) to tell the NIC that the packet is an IPv4
         * packet. If the packet is a tunneled packet, this flag is related to
         * the inner headers.
         */
        const PKT_TX_IPV4 = ffi::PKT_TX_IPV4;

        /**
         * Packet is IPv6. This flag must be set when using an offload feature
         * (TSO or L4 checksum) to tell the NIC that the packet is an IPv6
         * packet. If the packet is a tunneled packet, this flag is related to
         * the inner headers.
         */
        const PKT_TX_IPV6          = ffi::PKT_TX_IPV6;

        /// TX packet is a 802.1q VLAN packet.
        const PKT_TX_VLAN_PKT      = ffi::PKT_TX_VLAN_PKT;

        /**
         * Offload the IP checksum of an external header in the hardware. The
         * flag PKT_TX_OUTER_IPV4 should also be set by the application, alto ugh
         * a PMD will only check PKT_TX_IP_CKSUM.  The IP checksum field in the
         * packet must be set to 0.
         *  - set the outer IP checksum field in the packet to 0
         *  - fill the mbuf offload information: outer_l2_len, outer_l3_len
         */
        const PKT_TX_OUTER_IP_CKSUM   = ffi::PKT_TX_OUTER_IP_CKSUM;

        /**
         * Packet outer header is IPv4. This flag must be set when using any
         * outer offload feature (L3 or L4 checksum) to tell the NIC that the
         * outer header of the tunneled packet is an IPv4 packet.
         */
        const PKT_TX_OUTER_IPV4   = ffi::PKT_TX_OUTER_IPV4;

        /**
         * Packet outer header is IPv6. This flag must be set when using any
         * outer offload feature (L4 checksum) to tell the NIC that the outer
         * header of the tunneled packet is an IPv6 packet.
         */
        const PKT_TX_OUTER_IPV6    = ffi::PKT_TX_OUTER_IPV6;
        /// reserved for future mbuf use
        const EXT_ATTACHED_MBUF    = ffi::EXT_ATTACHED_MBUF;
        /// Indirect attached mbuf
        const IND_ATTACHED_MBUF    = ffi::IND_ATTACHED_MBUF;

        /// Use final bit of flags to indicate a control mbuf
        ///
        /// Mbuf contains control data
        const CTRL_MBUF_FLAG       = 1 << 63;
    }
}

/**
 * Some NICs need at least 2KB buffer to RX standard Ethernet frame without
 * splitting it into multiple segments.
 * So, for mbufs that planned to be involved into RX/TX, the recommended
 * minimal buffer length is 2KB + RTE_PKTMBUF_HEADROOM.
 */
pub const RTE_MBUF_DEFAULT_BUF_SIZE: u16 = (ffi::RTE_MBUF_DEFAULT_DATAROOM + ffi::RTE_PKTMBUF_HEADROOM) as u16;

pub type RawMBuf = ffi::rte_mbuf;
pub type RawMBufPtr = *mut ffi::rte_mbuf;

#[repr(transparent)]
#[derive(Debug)]
pub struct MBuf(NonNull<RawMBuf>);

impl Deref for MBuf {
    type Target = RawMBuf;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for MBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl AsRaw for MBuf {
    type Raw = RawMBuf;

    fn as_raw(&self) -> *mut Self::Raw {
        self.0.as_ptr() as *mut _
    }
}

impl FromRaw for MBuf {
    fn from_raw(raw: *mut Self::Raw) -> Option<Self> {
        NonNull::new(raw).map(MBuf)
    }
}

impl From<RawMBufPtr> for MBuf {
    fn from(p: RawMBufPtr) -> Self {
        Self::from_raw(p).unwrap()
    }
}

impl MBuf {
    /// Prefetch the first part of the mbuf
    #[inline]
    pub fn prefetch_part1(&self) {
        unsafe { ffi::rte_mbuf_prefetch_part1(self.as_raw()) }
    }

    /// Prefetch the second part of the mbuf
    pub fn prefetch_part2(&self) {
        unsafe { ffi::rte_mbuf_prefetch_part2(self.as_raw()) }
    }

    /// Return the mbuf owning the data buffer address of an indirect mbuf.
    pub fn from_indirect(other: &MBuf) -> Self {
        unsafe { ffi::rte_mbuf_from_indirect(other.as_raw()) }.into()
    }

    /// Return the buffer address embedded in the given mbuf.
    pub fn buf_addr(&self) -> NonNull<u8> {
        NonNull::new(unsafe { ffi::rte_mbuf_to_baddr(self.as_raw()) })
            .unwrap()
            .cast()
    }

    /// Return the starting address of the private data area embedded in the given mbuf.
    pub fn priv_addr(&self) -> NonNull<u8> {
        NonNull::new(unsafe { ffi::rte_mbuf_to_priv(self.as_raw()) })
            .unwrap()
            .cast()
    }

    /// Offload features.
    #[inline]
    pub fn offload(&self) -> OffloadFlags {
        OffloadFlags::from_bits_truncate(self.ol_flags)
    }

    /// The mbuf is cloned by mbuf indirection.
    #[inline]
    pub fn has_cloned(&self) -> bool {
        self.offload().contains(OffloadFlags::IND_ATTACHED_MBUF)
    }

    /// The mbuf has an external buffer.
    #[inline]
    pub fn has_ext_buf(&self) -> bool {
        self.offload().contains(OffloadFlags::EXT_ATTACHED_MBUF)
    }

    /// The mbuf is indirect
    #[inline]
    pub fn is_indirect(&self) -> bool {
        self.has_cloned()
    }

    /// The mbuf is direct
    #[inline]
    pub fn is_direct(&self) -> bool {
        self.offload()
            .intersects(OffloadFlags::IND_ATTACHED_MBUF | OffloadFlags::EXT_ATTACHED_MBUF)
    }

    /// Put mbuf back into its original mempool.
    pub fn free(&mut self) {
        unsafe { ffi::rte_mbuf_raw_free(self.as_raw()) }
    }

    /// Reads the value of an mbuf's refcnt.
    pub fn refcnt_read(&self) -> u16 {
        unsafe { ffi::rte_mbuf_refcnt_read(self.as_raw()) }
    }

    /// Sets an mbuf's refcnt to a defined value.
    pub fn refcnt_set(&mut self, new: u16) {
        unsafe { ffi::rte_mbuf_refcnt_set(self.as_raw(), new) }
    }

    /// Adds given value to an mbuf's refcnt and returns its new value.
    pub fn refcnt_update(&mut self, new: i16) -> u16 {
        unsafe { ffi::rte_mbuf_refcnt_update(self.as_raw(), new) }
    }

    /// Sanity checks on an mbuf.
    ///
    /// Check the consistency of the given mbuf.
    /// The function will cause a panic if corruption is detected.
    pub fn sanity_check(&self, is_header: bool) {
        unsafe { ffi::rte_mbuf_sanity_check(self.as_raw(), if is_header { 1 } else { 0 }) }
    }

    /// Reset the data_off field of a packet mbuf to its default value.
    pub fn reset_headroom(&mut self) {
        unsafe { ffi::rte_pktmbuf_reset_headroom(self.as_raw()) }
    }

    /// Reset the fields of a packet mbuf to their default values.
    pub fn reset(&mut self) {
        unsafe { ffi::rte_pktmbuf_reset(self.as_raw()) }
    }

    /// Detach a packet mbuf from external buffer or direct buffer.
    pub fn detach(&self) {
        unsafe { ffi::rte_pktmbuf_detach(self.as_raw()) }
    }

    /// Get the headroom in a packet mbuf.
    pub fn headroom(&self) -> u16 {
        unsafe { ffi::rte_pktmbuf_headroom(self.as_raw()) }
    }

    /// Get the tailroom of a packet mbuf.
    pub fn tailroom(&self) -> u16 {
        unsafe { ffi::rte_pktmbuf_tailroom(self.as_raw()) }
    }

    /// Get the last segment of the packet.
    pub fn lastseg(&self) -> Self {
        unsafe { ffi::rte_pktmbuf_lastseg(self.as_raw()) }.into()
    }

    /// Get a pointer which points to an offset into the data in the mbuf.
    #[inline]
    pub fn mtod_offset<T>(&self, off: usize) -> NonNull<T> {
        NonNull::new(unsafe { (self.buf_addr as *mut u8).add(self.data_off as usize + off) })
            .unwrap()
            .cast()
    }

    /// Get a pointer which points to the start of the data in the mbuf.
    #[inline]
    pub fn mtod<T>(&self) -> NonNull<T> {
        self.mtod_offset(0)
    }

    /// Return the IO address of the beginning of the mbuf data
    #[inline]
    pub fn data_iova(&self) -> ffi::rte_iova_t {
        unsafe { self.__bindgen_anon_1.buf_iova + self.data_off as u64 }
    }

    /// Return the default IO address of the beginning of the mbuf data
    #[inline]
    pub fn data_iova_default(&self) -> ffi::rte_iova_t {
        unsafe { self.__bindgen_anon_1.buf_iova + ffi::RTE_PKTMBUF_HEADROOM as u64 }
    }

    /// Get the IO address that points to an offset of the start of the data in the mbuf
    #[inline]
    pub fn iova_offset(&self, off: usize) -> ffi::rte_iova_t {
        self.data_iova() + off as u64
    }

    /// Get the IO address that points to the start of the data in the mbuf
    #[inline]
    pub fn iova(&self) -> ffi::rte_iova_t {
        self.iova_offset(0)
    }

    /// Returns the length of the packet.
    pub fn pkt_len(&self) -> usize {
        self.pkt_len as usize
    }

    /// Returns the length of the segment.
    pub fn data_len(&self) -> usize {
        self.data_len as usize
    }

    /// Prepend len bytes to an mbuf data area.
    pub fn prepend(&mut self, len: usize) -> Result<NonNull<u8>> {
        unsafe { ffi::rte_pktmbuf_prepend(self.as_raw(), len as u16) }
            .as_result()
            .map(|p| p.cast())
    }

    /// Append len bytes to an mbuf.
    pub fn append(&mut self, len: usize) -> Result<NonNull<u8>> {
        unsafe { ffi::rte_pktmbuf_append(self.as_raw(), len as u16) }
            .as_result()
            .map(|p| p.cast())
    }

    /// Remove len bytes at the beginning of an mbuf.
    pub fn adj(&mut self, len: usize) -> Result<NonNull<u8>> {
        unsafe { ffi::rte_pktmbuf_adj(self.as_raw(), len as u16) }
            .as_result()
            .map(|p| p.cast())
    }

    /// Remove len bytes of data at the end of the mbuf.
    pub fn trim(&mut self, len: usize) -> Result<()> {
        unsafe { ffi::rte_pktmbuf_trim(self.as_raw(), len as u16) }.as_result()
    }

    /// Test if mbuf data is contiguous.
    pub fn is_contiguous(&self) -> bool {
        unsafe { ffi::rte_pktmbuf_is_contiguous(self.as_raw()) != 0 }
    }

    /// Read len data bytes in a mbuf at specified offset.
    pub fn read(&self, off: usize, buf: &mut [u8]) -> Option<&[u8]> {
        unsafe {
            NonNull::new(
                ffi::rte_pktmbuf_read(self.as_raw(), off as u32, buf.len() as u32, buf.as_mut_ptr() as *mut _)
                    as *mut u8,
            )
            .map(|p| slice::from_raw_parts(p.as_ptr(), buf.len()))
        }
    }

    /// Chain an mbuf to another, thereby creating a segmented packet.
    pub fn chain(&self, tail: &Self) -> Result<()> {
        unsafe { ffi::rte_pktmbuf_chain(self.as_raw(), tail.as_raw()) }.as_result()
    }

    /// Validate general requirements for Tx offload in mbuf.
    ///
    /// This function checks correctness and completeness of Tx offload settings.
    pub fn validate_tx_offload(&self) -> Result<()> {
        unsafe { ffi::rte_validate_tx_offload(self.as_raw()) }.as_result()
    }

    /// Linearize data in mbuf.
    ///
    /// This function moves the mbuf data in the first segment if there is enough tailroom.
    /// The subsequent segments are unchained and freed.
    pub fn linearize(&self) -> Result<()> {
        unsafe { ffi::rte_pktmbuf_linearize(self.as_raw()) }.as_result()
    }

    /// Dump an mbuf structure to the console.
    pub fn dump<S: AsRawFd>(&self, s: &S, dump_len: usize) {
        if let Ok(f) = cfile::open_stream(s, "w") {
            unsafe {
                ffi::rte_pktmbuf_dump(f.stream() as *mut ffi::FILE, self.as_raw(), dump_len as u32);
            }
        }
    }
}

pub trait MBufPool {
    /// Get the data room size of mbufs stored in a pktmbuf_pool
    fn data_room_size(&self) -> usize;

    /// Get the application private size of mbufs stored in a pktmbuf_pool
    fn priv_size(&self) -> usize;

    /// Allocate a new mbuf from a mempool.
    fn alloc(&mut self) -> MBuf;

    /// Allocate a bulk of mbufs, initialize refcnt and reset the fields to default values.
    fn alloc_bulk(&mut self, mbufs: &mut [RawMBufPtr]) -> Result<()>;
}

// impl MBufPool for MemoryPool {
//     /// Get the data room size of mbufs stored in a pktmbuf_pool
//     fn data_room_size(&self) -> usize {
//         unsafe { ffi::rte_pktmbuf_data_room_size(self.as_raw()) as usize }
//     }

//     /// Get the application private size of mbufs stored in a pktmbuf_pool
//     fn priv_size(&self) -> usize {
//         unsafe { ffi::rte_pktmbuf_priv_size(self.as_raw()) as usize }
//     }

//     /// Allocate a new mbuf from a mempool.
//     fn alloc(&mut self) -> MBuf {
//         unsafe { ffi::rte_pktmbuf_alloc(self) }
//     }

//     /// Allocate a bulk of mbufs, initialize refcnt and reset the fields to default values.
//     fn alloc_bulk(&mut self, mbufs: &mut [RawMBufPtr]) -> Result<()> {
//         unsafe { ffi::rte_pktmbuf_alloc_bulk(self, mbufs.as_mut_ptr(), mbufs.len() as u32) }.as_result()
//     }
// }

// TODO rte_mbuf_raw_alloc, rte_pktmbuf_clone

/// A macro that points to the start of the data in the mbuf.
#[macro_export]
macro_rules! pktmbuf_mtod {
    ($m:expr, $t:ty) => {
        pktmbuf_mtod_offset!($m, $t, 0)
    };
}

/// Create a mbuf pool.
///
/// This function creates and initializes a packet mbuf pool.
/// It is a wrapper to rte_mempool_create() with the proper packet constructor
/// and mempool constructor.
pub fn pktmbuf_pool_create(
    name: &str,
    n: u32,
    cache_size: u32,
    priv_size: u16,
    data_room_size: u16,
    socket_id: i32,
) -> Result<mempool::RawMemoryPoolPtr> {
    unsafe {
        ffi::rte_pktmbuf_pool_create(
            try!(to_cptr!(name)),
            n,
            cache_size,
            priv_size,
            data_room_size,
            socket_id,
        )
    }
    .as_result()
    .map(|p| p.as_ptr())
}
