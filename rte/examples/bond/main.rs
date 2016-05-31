#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate nix;
extern crate cfile;

#[macro_use]
extern crate rte;

use std::env;
use std::mem;
use std::net;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rte::*;

const EXIT_FAILURE: i32 = -1;

const MAX_PORTS: u8 = 4;

// Number of mbufs in mempool that is created
const NB_MBUF: u32 = 8192;

const MAX_PKT_BURST: usize = 32;

// How many packets to attempt to read from NIC in one go
const PKT_BURST_SZ: u32 = 32;

// How many objects (mbufs) to keep in per-lcore mempool cache
const MEMPOOL_CACHE_SZ: u32 = PKT_BURST_SZ;


// Configurable number of RX/TX ring descriptors
//
const RTE_RX_DESC_DEFAULT: u16 = 128;
const RTE_TX_DESC_DEFAULT: u16 = 512;

struct AppConfig {
    lcore_main_is_running: AtomicBool,
    lcore_main_core_id: LcoreId,
    bond_ip: u32,
    bond_mac_addr: ether::EtherAddr,
    bonded_port_id: PortId,
    port_packets: [AtomicUsize; 4],
    lock: spinlock::SpinLock,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut cfg: AppConfig = unsafe { mem::zeroed() };

        cfg.lock.init();

        cfg
    }
}

fn slave_port_init(port_id: u8,
                   port_conf: &ethdev::EthConf,
                   pktmbuf_pool: &mempool::RawMemoryPool) {
    info!("Setup port {}", port_id);

    let dev = ethdev::dev(port_id);

    dev.configure(1, 1, &port_conf)
        .expect(&format!("fail to configure device: port={}", port_id));

    // init one RX queue
    dev.rx_queue_setup(0, RTE_RX_DESC_DEFAULT, None, &pktmbuf_pool)
        .expect(&format!("fail to setup device rx queue: port={}", port_id));

    // init one TX queue on each port
    dev.tx_queue_setup(0, RTE_TX_DESC_DEFAULT, None)
        .expect(&format!("fail to setup device tx queue: port={}", port_id));

    // Start device
    dev.start().expect(&format!("fail to start device: port={}", port_id));

    dev.promiscuous_enable();

    info!("Port {} MAC: {}", port_id, dev.mac_addr());
}

fn bond_port_init(slave_count: u8,
                  port_conf: &ethdev::EthConf,
                  pktmbuf_pool: &mempool::RawMemoryPool)
                  -> bond::BondedDevice {
    let dev = bond::create("bond0", bond::BondMode::AdaptiveLB, 0)
        .expect("Faled to create bond port");

    let bonded_port_id = dev.portid();

    dev.configure(1, 1, &port_conf)
        .expect(&format!("fail to configure device: port={}", bonded_port_id));

    // init one RX queue
    dev.rx_queue_setup(0, RTE_RX_DESC_DEFAULT, None, &pktmbuf_pool)
        .expect(&format!("fail to setup device rx queue: port={}", bonded_port_id));

    // init one TX queue on each port
    dev.tx_queue_setup(0, RTE_TX_DESC_DEFAULT, None)
        .expect(&format!("fail to setup device tx queue: port={}", bonded_port_id));

    for slave_port_id in 0..slave_count {
        dev.add_slave(&ethdev::dev(slave_port_id))
            .expect(&format!("Oooops! adding slave {} to bond {} failed!",
                             slave_port_id,
                             bonded_port_id));
    }

    // Start device
    dev.start()
        .expect(&format!("fail to start device: port={}", bonded_port_id));

    dev.promiscuous_enable();

    info!("Bonded port {} MAC: {}", bonded_port_id, dev.mac_addr());

    dev
}

fn strip_vlan_hdr(ether_hdr: *const ether::EtherHdr) -> (*const libc::c_void, u16) {
    unsafe {
        if (*ether_hdr).ether_type != ether::ETHER_TYPE_VLAN_BE {
            (ether_hdr.offset(1) as *const libc::c_void, (*ether_hdr).ether_type)
        } else {
            let mut vlan_hdr: *const ether::VlanHdr = mem::transmute(ether_hdr.offset(1));

            while (*vlan_hdr).eth_proto == ether::ETHER_TYPE_VLAN_BE {
                vlan_hdr = vlan_hdr.offset(1);
            }

            debug!("VLAN taged frame, offset: {}",
                   vlan_hdr as usize - ether_hdr as usize);

            (vlan_hdr.offset(1) as *const libc::c_void, (*vlan_hdr).eth_proto)
        }
    }
}

// Main thread that does the work, reading from INPUT_PORT and writing to OUTPUT_PORT
extern "C" fn lcore_main(app_conf: &AppConfig) -> i32 {
    debug!("lcore_main is starting @ lcore {}", lcore::id().unwrap());

    let dev = ethdev::dev(app_conf.bonded_port_id);
    let mut pkts: [mbuf::RawMbufPtr; MAX_PKT_BURST] = unsafe { mem::zeroed() };

    while app_conf.lcore_main_is_running.load(Ordering::Relaxed) {
        let rx_cnt = dev.rx_burst(0, &mut pkts[..]);

        // If didn't receive any packets, wait and go to next iteration
        if rx_cnt == 0 {
            eal::delay_us(50);
        }

        // Search incoming data for ARP packets and prepare response
        for pkt in &pkts[..rx_cnt] {
            let mut has_freed = false;

            app_conf.port_packets[0].fetch_add(1, Ordering::Relaxed);

            if let Some(mut ether_hdr) =
                   ptr_as_mut_ref!(pktmbuf_mtod!(*pkt, *mut ether::EtherHdr)) {

                let (next_hdr, next_proto) = strip_vlan_hdr(ether_hdr);

                match next_proto {
                    ether::ETHER_TYPE_ARP_BE => {
                        app_conf.port_packets[1].fetch_add(1, Ordering::Relaxed);

                        if let Some(mut arp_hdr) = ptr_as_mut_ref!(next_hdr as *mut arp::ArpHdr) {
                            if arp_hdr.arp_data.arp_tip == app_conf.bond_ip {
                                debug!("received ARP {:x} packet from {}",
                                    arp_hdr.arp_op.to_le(),
                                    ether::EtherAddr::from(arp_hdr.arp_data.arp_sha));

                                if arp_hdr.arp_op == (ARP_OP_REQUEST as u16).to_be() {
                                    arp_hdr.arp_op = (ARP_OP_REPLY as u16).to_be();

                                    ether::EtherAddr::copy(&ether_hdr.s_addr.addr_bytes,
                                                           &mut ether_hdr.d_addr.addr_bytes);
                                    ether::EtherAddr::copy(&app_conf.bond_mac_addr,
                                                           &mut ether_hdr.s_addr.addr_bytes);

                                    ether::EtherAddr::copy(&arp_hdr.arp_data.arp_sha.addr_bytes,
                                                           &mut arp_hdr.arp_data
                                                               .arp_tha
                                                               .addr_bytes);
                                    ether::EtherAddr::copy(&app_conf.bond_mac_addr,
                                                           &mut arp_hdr.arp_data
                                                               .arp_sha
                                                               .addr_bytes);

                                    arp_hdr.arp_data.arp_tip = arp_hdr.arp_data.arp_sip;
                                    arp_hdr.arp_data.arp_sip = app_conf.bond_ip;

                                    if dev.tx_burst(0, &mut [*pkt]) == 1 {
                                        has_freed = true;
                                    }
                                }
                            }
                        }
                    }
                    ether::ETHER_TYPE_IPV4_BE => {
                        app_conf.port_packets[2].fetch_add(1, Ordering::Relaxed);

                        if let Some(mut ipv4_hdr) = ptr_as_mut_ref!(next_hdr as *mut ip::Ipv4Hdr) {
                            if ipv4_hdr.dst_addr == app_conf.bond_ip {
                                debug!("received IP packet from {}",
                                    net::Ipv4Addr::from(ipv4_hdr.src_addr));

                                ether::EtherAddr::copy(&ether_hdr.s_addr.addr_bytes,
                                                       &mut ether_hdr.d_addr.addr_bytes);
                                ether::EtherAddr::copy(&app_conf.bond_mac_addr,
                                                       &mut ether_hdr.s_addr.addr_bytes);

                                ipv4_hdr.dst_addr = ipv4_hdr.src_addr;
                                ipv4_hdr.src_addr = app_conf.bond_ip;

                                if dev.tx_burst(0, &mut [*pkt]) == 1 {
                                    has_freed = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Free processed packets
            if !has_freed {
                mbuf::pktmbuf_free(*pkt)
            }
        }
    }

    debug!("BYE lcore_main");

    0
}


struct CmdActionResult {
    action: cmdline::FixedStr,
}

impl CmdActionResult {
    fn help(&mut self, cl: &cmdline::RawCmdline, _: Option<&libc::c_void>) {
        cl.println(r#"ALB - link bonding mode 6 example\n"
            "send IP    - sends one ARPrequest thru bonding for IP.\n"
            "start      - starts listening ARPs.\n"
            "stop       - stops lcore_main.\n"
            "show       - shows some bond info: ex. active slaves etc.\n"
            "help       - prints help.\n"
            "quit       - terminate all threads and quit.\n"#);
    }

    fn quit(&mut self, cl: &cmdline::RawCmdline, data: Option<&AppConfig>) {
        let app_conf = data.unwrap();

        if !app_conf.lcore_main_is_running.load(Ordering::Relaxed) {
            cl.println(&format!("lcore_main not running on core: {}", app_conf.lcore_main_core_id))
                .unwrap();
        } else {
            app_conf.lcore_main_is_running.store(false, Ordering::Relaxed);

            launch::wait_lcore(app_conf.lcore_main_core_id);

            cl.println(&format!("lcore_main stopped on core: {}", app_conf.lcore_main_core_id))
                .unwrap();
        }

        cl.quit();
    }
}

fn prompt(app_conf: &AppConfig) {
    let cmd_action_help = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "help");
    let cmd_action_quit = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "quit");

    let cmd_help = cmdline::inst(CmdActionResult::help,
                                 None,
                                 "show help",
                                 &[&cmd_action_help]);
    let cmd_quit = cmdline::inst(CmdActionResult::quit,
                                 Some(app_conf),
                                 "quit",
                                 &[&cmd_action_quit]);

    let cmds = &[&cmd_help, &cmd_quit];

    cmdline::new(cmds)
        .open_stdin("bond6> ")
        .interact();
}

// Main function, does initialisation and calls the per-lcore functions
fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    // init EAL
    eal::init(&args).expect("Cannot init EAL");

    devargs::dump(&cfile::stdout().unwrap());

    let nb_ports = ethdev::count();

    if nb_ports == 0 {
        eal::exit(EXIT_FAILURE, "Give at least one port\n");
    } else if nb_ports > MAX_PORTS {
        eal::exit(EXIT_FAILURE, "You can have max 4 ports\n");
    } else {
        info!("found {} ports", nb_ports);
    }

    // create the mbuf pool
    let pktmbuf_pool = mbuf::pktmbuf_pool_create("mbuf_pool",
                                                 NB_MBUF,
                                                 MEMPOOL_CACHE_SZ,
                                                 0,
                                                 mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
                                                 eal::socket_id())
        .expect("fail to initial mbuf pool");

    let port_conf = ethdev::EthConf {
        rx_adv_conf: Some(ethdev::RxAdvConf {
            rss_conf: Some(ethdev::EthRssConf {
                key: None,
                hash: ethdev::ETH_RSS_IP,
            }),
            ..ethdev::RxAdvConf::default()
        }),
        ..ethdev::EthConf::default()
    };

    // initialize all ports
    for portid in 0..nb_ports {
        slave_port_init(portid, &port_conf, &pktmbuf_pool);
    }

    let bonded_dev = bond_port_init(nb_ports, &port_conf, &pktmbuf_pool);

    // check state of lcores
    lcore::foreach_slave(|lcore_id| {
        if launch::lcore_state(lcore_id) != launch::LcoreState::Wait {
            eal::exit(-libc::EBUSY, "lcores not ready");
        }
    });

    // start lcore main on core != master_core - ARP response thread
    let slave_core_id = lcore::next(lcore::id().unwrap(), true);

    if slave_core_id == 0 || slave_core_id >= RTE_MAX_LCORE {
        eal::exit(-libc::EPERM, "missing slave core");
    }

    let app_conf = AppConfig {
        bond_ip: u32::from(net::Ipv4Addr::new(10, 0, 0, 7)),
        bond_mac_addr: bonded_dev.mac_addr(),
        bonded_port_id: bonded_dev.portid(),
        lcore_main_is_running: AtomicBool::new(true),
        lcore_main_core_id: slave_core_id,
        ..AppConfig::default()
    };

    launch::remote_launch(unsafe { mem::transmute(lcore_main) },
                          Some(&app_conf),
                          slave_core_id)
        .expect("Cannot launch task");

    info!("Starting lcore_main on core {} Our IP {}",
        slave_core_id, net::Ipv4Addr::from(app_conf.bond_ip));

    prompt(&app_conf);

    lcore::foreach_slave(|lcore_id| launch::wait_lcore(lcore_id));
}
