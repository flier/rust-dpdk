#[macro_use]
extern crate log;
extern crate cfile;
extern crate libc;
extern crate nix;
extern crate pretty_env_logger;
extern crate rte;

use std::cell::RefCell;
use std::env;
use std::mem;
use std::net;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rte::arp::{ARP_HRD_ETHER, ARP_OP_REPLY, ARP_OP_REQUEST};
use rte::bond::BondedDevice;
use rte::ethdev::EthDevice;
use rte::ether::{ETHER_TYPE_IPv4, ETHER_ADDR_LEN, ETHER_TYPE_ARP};
use rte::lcore::RTE_MAX_LCORE;
use rte::mbuf::MBufPool;
use rte::memory::AsMutRef;
use rte::*;

const EXIT_FAILURE: i32 = -1;

const MAX_PORTS: u16 = 4;

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
    lcore_main_core_id: lcore::Id,
    bond_ip: net::Ipv4Addr,
    bond_mac_addr: ether::EtherAddr,
    bonded_port_id: PortId,
    pktmbuf_pool: mempool::MemoryPool,
    port_packets: [AtomicUsize; 4],
}

impl Default for AppConfig {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

impl AppConfig {
    fn is_running(&self) -> bool {
        self.lcore_main_is_running.load(Ordering::Relaxed)
    }

    fn start(&self) {
        launch::remote_launch(lcore_main, Some(self), self.lcore_main_core_id).expect("Cannot launch task");

        self.lcore_main_is_running.store(true, Ordering::Relaxed);

        info!(
            "Starting lcore_main on core {} Our IP {}",
            self.lcore_main_core_id, self.bond_ip
        );
    }

    fn stop(&self) {
        self.lcore_main_is_running.store(false, Ordering::Relaxed);

        self.lcore_main_core_id.wait();
    }
}

fn slave_port_init(port_id: ethdev::PortId, port_conf: &ethdev::EthConf, pktmbuf_pool: &mut mempool::MemoryPool) {
    info!("Setup port {}", port_id);

    let dev = port_id;

    dev.configure(1, 1, &port_conf)
        .expect(&format!("fail to configure device: port={}", port_id));

    // init one RX queue
    dev.rx_queue_setup(0, RTE_RX_DESC_DEFAULT, None, pktmbuf_pool)
        .expect(&format!("fail to setup device rx queue: port={}", port_id));

    // init one TX queue on each port
    dev.tx_queue_setup(0, RTE_TX_DESC_DEFAULT, None)
        .expect(&format!("fail to setup device tx queue: port={}", port_id));

    // Start device
    dev.start().expect(&format!("fail to start device: port={}", port_id));

    dev.promiscuous_enable();

    info!("Port {} MAC: {}", port_id, dev.mac_addr());
}

fn bond_port_init(
    slave_count: u16,
    port_conf: &ethdev::EthConf,
    pktmbuf_pool: &mut mempool::MemoryPool,
) -> ethdev::PortId {
    let dev = bond::create("bond0", bond::BondMode::AdaptiveLB, 0).expect("Faled to create bond port");

    let bonded_port_id = dev;

    dev.configure(1, 1, &port_conf)
        .expect(&format!("fail to configure device: port={}", bonded_port_id));

    // init one RX queue
    dev.rx_queue_setup(0, RTE_RX_DESC_DEFAULT, None, pktmbuf_pool)
        .expect(&format!("fail to setup device rx queue: port={}", bonded_port_id));

    // init one TX queue on each port
    dev.tx_queue_setup(0, RTE_TX_DESC_DEFAULT, None)
        .expect(&format!("fail to setup device tx queue: port={}", bonded_port_id));

    for slave_port_id in 0..slave_count {
        dev.add_slave(slave_port_id).expect(&format!(
            "Oooops! adding slave {} to bond {} failed!",
            slave_port_id, bonded_port_id
        ));
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
            let mut vlan_hdr = ether_hdr.offset(1) as *const ether::VlanHdr;

            while (*vlan_hdr).eth_proto == ether::ETHER_TYPE_VLAN_BE {
                vlan_hdr = vlan_hdr.offset(1);
            }

            debug!("VLAN taged frame, offset: {}", vlan_hdr as usize - ether_hdr as usize);

            (vlan_hdr.offset(1) as *const libc::c_void, (*vlan_hdr).eth_proto)
        }
    }
}

// Main thread that does the work, reading from INPUT_PORT and writing to OUTPUT_PORT
fn lcore_main(app_conf: Option<&AppConfig>) -> i32 {
    debug!("lcore_main is starting @ lcore {}", lcore::current().unwrap());

    let app_conf = app_conf.unwrap();
    let dev = app_conf.bonded_port_id;
    let mut pkts: [Option<mbuf::MBuf>; MAX_PKT_BURST] = unsafe { mem::zeroed() };
    let bond_ip = u32::from(app_conf.bond_ip).to_be();

    while app_conf.lcore_main_is_running.load(Ordering::Relaxed) {
        let rx_cnt = dev.rx_burst(0, &mut pkts[..]);

        // If didn't receive any packets, wait and go to next iteration
        if rx_cnt == 0 {
            delay_us(50);

            continue;
        }

        debug!("received {} packets from bonded port {}", rx_cnt, dev.portid());

        app_conf.port_packets[0].fetch_add(rx_cnt, Ordering::Relaxed);

        // Search incoming data for ARP packets and prepare response
        for pkt in pkts.into_iter().take(rx_cnt) {
            if let Some(m) = pkt {
                let mut p = m.mtod::<ether::EtherHdr>();
                let ether_hdr = unsafe { p.as_mut() };
                let (next_hdr, next_proto) = strip_vlan_hdr(ether_hdr);

                match next_proto {
                    ether::ETHER_TYPE_ARP_BE => {
                        app_conf.port_packets[1].fetch_add(1, Ordering::Relaxed);

                        if let Some(mut arp_hdr) = (next_hdr as *mut arp::ArpHdr).as_mut_ref() {
                            if arp_hdr.arp_data.arp_tip == bond_ip {
                                debug!(
                                    "received ARP {:x} packet from {}",
                                    arp_hdr.arp_op.to_le(),
                                    ether::EtherAddr::from(arp_hdr.arp_data.arp_sha)
                                );

                                if arp_hdr.arp_op == (ARP_OP_REQUEST as u16).to_be() {
                                    arp_hdr.arp_op = (ARP_OP_REPLY as u16).to_be();

                                    ether::EtherAddr::copy(
                                        &ether_hdr.s_addr.addr_bytes,
                                        &mut ether_hdr.d_addr.addr_bytes,
                                    );
                                    ether::EtherAddr::copy(&app_conf.bond_mac_addr, &mut ether_hdr.s_addr.addr_bytes);

                                    ether::EtherAddr::copy(
                                        &arp_hdr.arp_data.arp_sha.addr_bytes,
                                        &mut arp_hdr.arp_data.arp_tha.addr_bytes,
                                    );
                                    ether::EtherAddr::copy(
                                        &app_conf.bond_mac_addr,
                                        &mut arp_hdr.arp_data.arp_sha.addr_bytes,
                                    );

                                    arp_hdr.arp_data.arp_tip = arp_hdr.arp_data.arp_sip;
                                    arp_hdr.arp_data.arp_sip = bond_ip;

                                    let _ = dev.tx_burst(0, &mut [m]);
                                }
                            }
                        }
                    }
                    ether::ETHER_TYPE_IPV4_BE => {
                        app_conf.port_packets[2].fetch_add(1, Ordering::Relaxed);

                        if let Some(mut ipv4_hdr) = (next_hdr as *mut ip::Ipv4Hdr).as_mut_ref() {
                            if ipv4_hdr.dst_addr == bond_ip {
                                debug!("received IP packet from {}", net::Ipv4Addr::from(ipv4_hdr.src_addr));

                                ether::EtherAddr::copy(&ether_hdr.s_addr.addr_bytes, &mut ether_hdr.d_addr.addr_bytes);
                                ether::EtherAddr::copy(&app_conf.bond_mac_addr, &mut ether_hdr.s_addr.addr_bytes);

                                ipv4_hdr.dst_addr = ipv4_hdr.src_addr;
                                ipv4_hdr.src_addr = bond_ip;

                                let _ = dev.tx_burst(0, &mut [m]);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    debug!("BYE lcore_main");

    0
}

struct CmdActionResult {
    action: cmdline::FixedStr,
    ip: cmdline::IpNetAddr,
}

impl CmdActionResult {
    fn send(&mut self, cl: &cmdline::CmdLine, data: Option<Rc<RefCell<AppConfig>>>) {
        let app_conf = &*data.unwrap();
        let mut app_conf = app_conf.borrow_mut();

        match self.ip.to_ipaddr() {
            net::IpAddr::V4(ip) => {
                let mut m = app_conf.pktmbuf_pool.alloc().unwrap();

                let pkt_size = mem::size_of::<ether::EtherHdr>() + mem::size_of::<arp::ArpHdr>();

                m.data_len = pkt_size as u16;
                m.pkt_len = pkt_size as u32;

                let mut p = m.mtod::<ether::EtherHdr>();
                {
                    let ether_hdr = unsafe { p.as_mut() };
                    ether_hdr.ether_type = (ETHER_TYPE_ARP as u16).to_be();

                    ether::EtherAddr::copy(&app_conf.bond_mac_addr, &mut ether_hdr.s_addr.addr_bytes);
                    ether::EtherAddr::copy(&ether::EtherAddr::broadcast(), &mut ether_hdr.d_addr.addr_bytes);
                }

                let mut p = unsafe { NonNull::new_unchecked(p.as_ptr().add(1) as *mut arp::ArpHdr) };
                let arp_hdr = unsafe { p.as_mut() };

                arp_hdr.arp_hrd = (ARP_HRD_ETHER as u16).to_be();
                arp_hdr.arp_pro = (ETHER_TYPE_IPv4 as u16).to_be();
                arp_hdr.arp_hln = ETHER_ADDR_LEN as u8;
                arp_hdr.arp_pln = mem::size_of::<u32>() as u8;
                arp_hdr.arp_op = (ARP_OP_REQUEST as u16).to_be();

                ether::EtherAddr::copy(&app_conf.bond_mac_addr, &mut arp_hdr.arp_data.arp_sha.addr_bytes);
                ether::EtherAddr::copy(&ether::EtherAddr::zeroed(), &mut arp_hdr.arp_data.arp_tha.addr_bytes);

                arp_hdr.arp_data.arp_sip = u32::from(app_conf.bond_ip).to_be();
                arp_hdr.arp_data.arp_tip = u32::from(ip).to_be();

                if app_conf.bonded_port_id.tx_burst(0, &mut [m]) == 1 {
                    debug!("send ARP request to {}", ip);
                }
            }
            _ => {
                cl.println("Wrong IP format. Only IPv4 is supported").unwrap();
            }
        }
    }

    fn start(&mut self, cl: &cmdline::CmdLine, data: Option<Rc<RefCell<AppConfig>>>) {
        let app_conf = &*data.unwrap();
        let app_conf = app_conf.borrow();

        if app_conf.is_running() {
            cl.println(&format!(
                "lcore_main already running on core: {}",
                app_conf.lcore_main_core_id
            )).unwrap();
        } else {
            app_conf.start();
        }
    }

    fn stop(&mut self, cl: &cmdline::CmdLine, data: Option<Rc<RefCell<AppConfig>>>) {
        let app_conf = &*data.unwrap();
        let app_conf = app_conf.borrow();

        if !app_conf.is_running() {
            cl.println(&format!(
                "lcore_main not running on core: {}",
                app_conf.lcore_main_core_id
            )).unwrap();
        } else {
            app_conf.stop();

            cl.println(&format!("lcore_main stopped on core: {}", app_conf.lcore_main_core_id))
                .unwrap();
        }
    }

    fn show(&mut self, cl: &cmdline::CmdLine, data: Option<Rc<RefCell<AppConfig>>>) {
        let app_conf = &*data.unwrap();
        let app_conf = app_conf.borrow();

        let dev = app_conf.bonded_port_id;

        let active_slaves = dev.active_slaves().unwrap();
        let primary = dev.primary().unwrap();

        for slave in dev.slaves().unwrap() {
            let role = if slave == primary {
                "primary"
            } else if active_slaves.contains(&slave) {
                "active"
            } else {
                "unused"
            };

            cl.println(&format!("Slave {}, MAC={}, {}", slave.portid(), slave.mac_addr(), role))
                .unwrap();
        }

        cl.println(&format!(
            "Active_slaves: {}, packets received:Tot: {}, Arp: {}, IPv4: {}",
            active_slaves.len(),
            app_conf.port_packets[0].load(Ordering::Relaxed),
            app_conf.port_packets[1].load(Ordering::Relaxed),
            app_conf.port_packets[2].load(Ordering::Relaxed)
        )).unwrap();
    }

    fn help(&mut self, cl: &cmdline::CmdLine, _: Option<Rc<RefCell<AppConfig>>>) {
        cl.println(
            r#"ALB - link bonding mode 6 example
    send IP    - sends one ARPrequest thru bonding for IP.
    start      - starts listening ARPs.
    stop       - stops lcore_main.
    show       - shows some bond info: ex. active slaves etc.
    help       - prints help.
    quit       - terminate all threads and quit."#,
        ).unwrap();
    }

    fn quit(&mut self, cl: &cmdline::CmdLine, data: Option<Rc<RefCell<AppConfig>>>) {
        self.stop(cl, data);

        cl.quit();
    }
}

fn prompt(app_conf: AppConfig) {
    let app_conf = Rc::new(RefCell::new(app_conf));

    let cmd_obj_action_send = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "send");
    let cmd_obj_ip = TOKEN_IPV4_INITIALIZER!(CmdActionResult, ip);
    let cmd_obj_action_start = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "start");
    let cmd_obj_action_stop = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "stop");
    let cmd_obj_action_show = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "show");
    let cmd_obj_action_help = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "help");
    let cmd_obj_action_quit = TOKEN_STRING_INITIALIZER!(CmdActionResult, action, "quit");

    let cmd_send = cmdline::inst(
        CmdActionResult::send,
        Some(app_conf.clone()),
        "send client_ip",
        &[&cmd_obj_action_send, &cmd_obj_ip],
    );
    let cmd_start = cmdline::inst(
        CmdActionResult::start,
        Some(app_conf.clone()),
        "starts listening if not started at startup",
        &[&cmd_obj_action_start],
    );
    let cmd_stop = cmdline::inst(
        CmdActionResult::stop,
        Some(app_conf.clone()),
        "stops listening if started at startup",
        &[&cmd_obj_action_stop],
    );
    let cmd_show = cmdline::inst(
        CmdActionResult::show,
        Some(app_conf.clone()),
        "show listening status",
        &[&cmd_obj_action_show],
    );
    let cmd_help = cmdline::inst(
        CmdActionResult::help,
        Some(app_conf.clone()),
        "show help",
        &[&cmd_obj_action_help],
    );
    let cmd_quit = cmdline::inst(
        CmdActionResult::quit,
        Some(app_conf.clone()),
        "quit",
        &[&cmd_obj_action_quit],
    );

    let cmds = &[&cmd_send, &cmd_start, &cmd_stop, &cmd_show, &cmd_help, &cmd_quit];

    cmdline::new(cmds)
        .open_stdin("bond6> ")
        .expect("fail to open stdin")
        .interact();
}

// Main function, does initialisation and calls the per-lcore functions
fn main() {
    pretty_env_logger::init();

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
    let mut pktmbuf_pool = mbuf::pool_create(
        "mbuf_pool",
        NB_MBUF,
        MEMPOOL_CACHE_SZ,
        0,
        mbuf::RTE_MBUF_DEFAULT_BUF_SIZE as u16,
        rte::socket_id() as i32,
    ).expect("fail to initial mbuf pool");

    let port_conf = ethdev::EthConf {
        rx_adv_conf: Some(ethdev::RxAdvConf {
            rss_conf: Some(ethdev::EthRssConf {
                key: None,
                hash: ethdev::RssHashFunc::ETH_RSS_IP,
            }),
            ..ethdev::RxAdvConf::default()
        }),
        ..ethdev::EthConf::default()
    };

    // initialize all ports
    for portid in 0..nb_ports {
        slave_port_init(portid, &port_conf, &mut pktmbuf_pool);
    }

    let bonded_dev = bond_port_init(nb_ports, &port_conf, &mut pktmbuf_pool);

    // check state of lcores
    lcore::foreach_slave(|lcore_id| {
        if lcore_id.state() != launch::State::Wait {
            eal::exit(-libc::EBUSY, "lcores not ready");
        }
    });

    // start lcore main on core != master_core - ARP response thread
    let slave_core_id = lcore::current().unwrap().next().unwrap();

    if slave_core_id == 0 || slave_core_id >= RTE_MAX_LCORE {
        eal::exit(-libc::EPERM, "missing slave core");
    }

    let app_conf = AppConfig {
        bond_ip: net::Ipv4Addr::new(10, 0, 0, 7),
        bond_mac_addr: bonded_dev.mac_addr(),
        bonded_port_id: bonded_dev.portid(),
        lcore_main_is_running: AtomicBool::new(true),
        lcore_main_core_id: slave_core_id,
        pktmbuf_pool,
        ..AppConfig::default()
    };

    app_conf.start();

    prompt(app_conf);

    launch::mp_wait_lcore();
}
