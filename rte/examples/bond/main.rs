#[macro_use]
extern crate log;
extern crate env_logger;
extern crate libc;
extern crate nix;
extern crate cfile;

#[macro_use]
extern crate rte;

use std::env;

use nix::sys::signal;

use rte::*;

const EXIT_FAILURE: i32 = -1;

const MAX_PORTS: u8 = 4;

// Number of mbufs in mempool that is created
const NB_MBUF: u32 = 8192;

// How many packets to attempt to read from NIC in one go
const PKT_BURST_SZ: u32 = 32;

// How many objects (mbufs) to keep in per-lcore mempool cache
const MEMPOOL_CACHE_SZ: u32 = PKT_BURST_SZ;


// Configurable number of RX/TX ring descriptors
//
const RTE_RX_DESC_DEFAULT: u16 = 128;
const RTE_TX_DESC_DEFAULT: u16 = 512;

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

    info!("Port {} MAC: {}", port_id, dev.mac_addr());
}

fn bond_port_init(slave_count: u8,
                  port_conf: &ethdev::EthConf,
                  pktmbuf_pool: &mempool::RawMemoryPool) {
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

    bond_port_init(nb_ports, &port_conf, &pktmbuf_pool);
}
