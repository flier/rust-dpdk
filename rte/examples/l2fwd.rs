#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

extern crate rte;

use std::mem;
use std::env;
use std::clone::Clone;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;
use std::path::Path;

use rte::*;

const EXIT_FAILURE: i32 = 1;
const EXIT_SUCCESS: i32 = 0;

const MAX_RX_QUEUE_PER_LCORE: u32 = 16;

const MAX_TIMER_PERIOD: u32 = 86400; /* 1 day max */

const NB_MBUF: u32 = 2048;

// Configurable number of RX/TX ring descriptors

const RTE_TEST_RX_DESC_DEFAULT: u16 = 128;
const RTE_TEST_TX_DESC_DEFAULT: u16 = 512;

static nb_rxd: u16 = RTE_TEST_RX_DESC_DEFAULT;
static nb_txd: u16 = RTE_TEST_TX_DESC_DEFAULT;

#[derive(Copy)]
struct lcore_queue_conf {
    n_rx_port: u32,
    rx_port_list: [u8; MAX_RX_QUEUE_PER_LCORE as usize],
}
impl Clone for lcore_queue_conf {
    fn clone(&self) -> Self {
        *self
    }
}
impl Default for lcore_queue_conf {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}

// display usage
fn l2fwd_usage(program: &String, opts: getopts::Options) -> ! {
    let brief = format!("Usage: {} [EAL options] -- [options]", program);

    print!("{}", opts.usage(&brief));

    exit(-1);
}

// Parse the argument given in the command line of the application
fn l2fwd_parse_args(args: &Vec<String>) -> (u32, u32, Duration) {
    let mut opts = getopts::Options::new();
    let program = args[0].clone();

    opts.optopt("p",
                "",
                "hexadecimal bitmask of ports to configure",
                "PORTMASK");
    opts.optopt("q",
                "",
                "number of queue (=ports) per lcore (default is 1)",
                "NQ");
    opts.optopt("T",
                "",
                "statistics will be refreshed each PERIOD seconds (0 to disable, 10 default, \
                 86400 maximum)",
                "PERIOD");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("Invalid L2FWD arguments, {}", err);

            l2fwd_usage(&program, opts);
        }
    };

    if matches.opt_present("h") {
        l2fwd_usage(&program, opts);
    }

    let mut l2fwd_enabled_port_mask: u32 = 0; /* mask of enabled ports */
    let mut l2fwd_rx_queue_per_lcore: u32 = 1;
    let mut timer_period: u32 = 10;

    if let Some(arg) = matches.opt_str("p") {
        match u32::from_str_radix(arg.as_str(), 16) {
            Ok(mask) if mask != 0 => l2fwd_enabled_port_mask = mask,
            _ => {
                println!("invalid portmask, {}", arg);

                l2fwd_usage(&program, opts);
            }
        }
    }

    if let Some(arg) = matches.opt_str("q") {
        match u32::from_str(arg.as_str()) {
            Ok(n) if 0 < n && n < MAX_RX_QUEUE_PER_LCORE => l2fwd_rx_queue_per_lcore = n,
            _ => {
                println!("invalid queue number, {}", arg);

                l2fwd_usage(&program, opts);
            }
        }
    }

    if let Some(arg) = matches.opt_str("T") {
        match u32::from_str(arg.as_str()) {
            Ok(t) if 0 < t && t < MAX_TIMER_PERIOD => timer_period = t,
            _ => {
                println!("invalid timer period, {}", arg);

                l2fwd_usage(&program, opts);
            }
        }
    }

    (l2fwd_enabled_port_mask, l2fwd_rx_queue_per_lcore, Duration::from_secs(timer_period as u64))
}

fn main() {
    env_logger::init().unwrap();

    let mut args: Vec<String> = env::args().collect();
    let program = String::from(Path::new(&args[0]).file_name().unwrap().to_str().unwrap());

    let (eal_args, opt_args) = if let Some(pos) = args.iter().position(|arg| arg == "--") {
        let (eal_args, opt_args) = args.split_at_mut(pos);

        opt_args[0] = program;

        (eal_args.to_vec(), opt_args.to_vec())
    } else {
        (args[..1].to_vec(), args.clone())
    };

    debug!("eal args: {:?}, l2fwd args: {:?}", eal_args, opt_args);

    let (l2fwd_enabled_port_mask, l2fwd_rx_queue_per_lcore, timer_period) =
        l2fwd_parse_args(&opt_args);

    // init EAL
    eal::init(&eal_args);

    // create the mbuf pool
    let l2fwd_pktmbuf_pool = mbuf::pktmbuf_pool_create("mbuf_pool",
                                                       NB_MBUF,
                                                       32,
                                                       0,
                                                       mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
                                                       eal::socket_id())
                                 .expect("Cannot init mbuf pool");

    let mut nb_ports = ethdev::Device::count();

    if nb_ports == 0 {
        println!("No Ethernet ports - bye");

        exit(0);
    }

    if nb_ports > RTE_MAX_ETHPORTS {
        nb_ports = RTE_MAX_ETHPORTS;
    }

    // ethernet addresses of ports
    let mut l2fwd_ports_eth_addr: [Option<ethdev::EtherAddr>; RTE_MAX_ETHPORTS as usize] =
        Default::default();

    // list of enabled ports
    let mut l2fwd_dst_ports = [0u8; RTE_MAX_ETHPORTS as usize];

    let mut last_port = 0;
    let mut nb_ports_in_mask = 0;

    let enabled_devices : Vec<ethdev::Device> = (0..nb_ports as u8)
                            .filter(|portid| (l2fwd_enabled_port_mask & (1 << portid) as u32) != 0) // skip ports that are not enabled
                            .map(|portid| ethdev::Device::from(portid))
                            .collect();

    if enabled_devices.is_empty() {
        eal::exit(EXIT_FAILURE,
                  "All available ports are disabled. Please set portmask.\n");
    }

    // Each logical core is assigned a dedicated TX queue on each port.
    for dev in enabled_devices.as_slice() {
        let portid = dev.portid();

        if (nb_ports_in_mask % 2) != 0 {
            l2fwd_dst_ports[portid as usize] = last_port;
            l2fwd_dst_ports[last_port as usize] = portid;
        } else {
            last_port = portid;
        }

        nb_ports_in_mask += 1;

        let info = dev.info();

        debug!("found port #{} with `{}` drive", portid, info.driver_name());
    }

    if (nb_ports_in_mask % 2) != 0 {
        println!("Notice: odd number of ports in portmask.");

        l2fwd_dst_ports[last_port as usize] = last_port;
    }

    let queue_conf = &mut [lcore_queue_conf::default(); RTE_MAX_LCORE as usize][..];
    let mut rx_lcore_id = 0;

    // Initialize the port/queue configuration of each logical core
    for dev in enabled_devices.as_slice() {
        let portid = dev.portid();

        while !lcore::enabled(rx_lcore_id) ||
              queue_conf[rx_lcore_id as usize].n_rx_port == l2fwd_rx_queue_per_lcore {
            rx_lcore_id += 1;

            if rx_lcore_id >= RTE_MAX_LCORE {
                eal::exit(EXIT_FAILURE, "Not enough cores\n");
            }
        }

        // Assigned a new logical core in the loop above.
        let qconf = &mut queue_conf[rx_lcore_id as usize];

        qconf.rx_port_list[qconf.n_rx_port as usize] = portid;
        qconf.n_rx_port += 1;

        println!("Lcore {}: RX port {}", rx_lcore_id, portid);
    }

    let port_conf = ethdev::ConfigBuilder::default().build();

    // Initialise each port
    for dev in enabled_devices.as_slice() {
        let portid = dev.portid() as usize;

        // init port
        print!("Initializing port {}... ", portid);

        dev.configure(1, 1, &port_conf)
           .expect(format!("fail to configure device: port={}", portid).as_str());

        let macaddr = dev.macaddr();

        l2fwd_ports_eth_addr[portid] = Some(macaddr);

        // init one RX queue
        dev.rx_queue_setup(0, nb_rxd, None, &l2fwd_pktmbuf_pool)
           .expect(format!("fail to setup device rx queue: port={}", portid).as_str());

        // init one TX queue on each port
        dev.tx_queue_setup(0, nb_txd, None)
           .expect(format!("fail to setup device tx queue: port={}", portid).as_str());

        // Initialize TX buffers

        // Start device
        dev.start().expect(format!("fail to start device: port={}", portid).as_str());

        println!("Done: ");

        dev.promiscuous_enable();

        println!("  Port {}, MAC address: {}", portid, macaddr);
    }

    for dev in enabled_devices.as_slice() {
        print!("Closing port {}...", dev.portid());
        dev.stop();
        dev.close();
        println!(" Done");
    }

    println!("Bye...");
}
