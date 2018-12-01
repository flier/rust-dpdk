#[macro_use]
extern crate log;
extern crate getopts;
extern crate libc;
extern crate nix;
extern crate pretty_env_logger;
extern crate rte;

use std::clone::Clone;
use std::env;
use std::io;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::process;
use std::str::FromStr;

use nix::sys::signal;

use rte::ethdev::{EthDevice, EthDeviceInfo, TxBuffer};
use rte::memory::AsMutRef;
use rte::*;

const EXIT_FAILURE: i32 = -1;

const MAX_PKT_BURST: usize = 32;

const MAX_RX_QUEUE_PER_LCORE: u32 = 16;

// A tsc-based timer responsible for triggering statistics printout
const TIMER_MILLISECOND: i64 = 2000000; /* around 1ms at 2 Ghz */
const MAX_TIMER_PERIOD: u32 = 86400; /* 1 day max */

const NB_MBUF: u32 = 2048;

// Configurable number of RX/TX ring descriptors

const RTE_TEST_RX_DESC_DEFAULT: u16 = 128;
const RTE_TEST_TX_DESC_DEFAULT: u16 = 512;

struct LcoreQueueConf {
    n_rx_port: u32,
    rx_port_list: [u32; MAX_RX_QUEUE_PER_LCORE as usize],
}

struct Conf {
    nb_rxd: u16,
    nb_txd: u16,

    queue_conf: [LcoreQueueConf; RTE_MAX_LCORE as usize],
}

impl Default for Conf {
    fn default() -> Self {
        let mut conf: Self = unsafe { mem::zeroed() };

        conf.nb_rxd = RTE_TEST_RX_DESC_DEFAULT;
        conf.nb_txd = RTE_TEST_TX_DESC_DEFAULT;

        return conf;
    }
}

// display usage
fn print_usage(program: &String, opts: getopts::Options) -> ! {
    let brief = format!("Usage: {} [EAL options] -- [options]", program);

    print!("{}", opts.usage(&brief));

    process::exit(-1);
}

// Parse the argument given in the command line of the application
fn parse_args(args: &Vec<String>) -> (u32, u32, u32) {
    let mut opts = getopts::Options::new();
    let program = args[0].clone();

    opts.optopt(
        "p",
        "",
        "hexadecimal bitmask of ports to configure",
        "PORTMASK",
    );
    opts.optopt(
        "q",
        "",
        "number of queue (=ports) per lcore (default is 1)",
        "NQ",
    );
    opts.optopt(
        "T",
        "",
        "statistics will be refreshed each PERIOD seconds (0 to disable, 10 default, \
         86400 maximum)",
        "PERIOD",
    );
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("Invalid L2FWD arguments, {}", err);

            print_usage(&program, opts);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
    }

    let mut enabled_port_mask: u32 = 0; // mask of enabled ports
    let mut rx_queue_per_lcore: u32 = 1;
    let mut timer_period_seconds: u32 = 10; // default period is 10 seconds

    if let Some(arg) = matches.opt_str("p") {
        match u32::from_str_radix(arg.as_str(), 16) {
            Ok(mask) if mask != 0 => enabled_port_mask = mask,
            _ => {
                println!("invalid portmask, {}", arg);

                print_usage(&program, opts);
            }
        }
    }

    if let Some(arg) = matches.opt_str("q") {
        match u32::from_str(arg.as_str()) {
            Ok(n) if 0 < n && n < MAX_RX_QUEUE_PER_LCORE => rx_queue_per_lcore = n,
            _ => {
                println!("invalid queue number, {}", arg);

                print_usage(&program, opts);
            }
        }
    }

    if let Some(arg) = matches.opt_str("T") {
        match u32::from_str(arg.as_str()) {
            Ok(t) if 0 < t && t < MAX_TIMER_PERIOD => timer_period_seconds = t,
            _ => {
                println!("invalid timer period, {}", arg);

                print_usage(&program, opts);
            }
        }
    }

    (enabled_port_mask, rx_queue_per_lcore, timer_period_seconds)
}

// Check the link status of all ports in up to 9s, and print them finally
fn check_all_ports_link_status(enabled_devices: &Vec<ethdev::PortId>) {
    print!("Checking link status");

    const CHECK_INTERVAL: u32 = 100;
    const MAX_CHECK_TIME: usize = 90;

    for _ in 0..MAX_CHECK_TIME {
        if unsafe { l2fwd_force_quit != 0 } {
            break;
        }

        if enabled_devices.iter().all(|dev| dev.link_nowait().up) {
            break;
        }

        delay_ms(CHECK_INTERVAL);

        print!(".");

        io::stdout().flush().unwrap();
    }

    println!("Done:");

    for dev in enabled_devices {
        let link = dev.link();

        if link.up {
            println!(
                "  Port {} Link Up - speed {} Mbps - {}",
                dev.portid(),
                link.speed,
                if link.duplex {
                    "full-duplex"
                } else {
                    "half-duplex"
                }
            )
        } else {
            println!("  Port {} Link Down", dev.portid());
        }
    }
}

#[link(name = "l2fwd_core")]
extern "C" {
    static mut l2fwd_force_quit: libc::c_int;

    static mut l2fwd_enabled_port_mask: libc::uint32_t;

    static mut l2fwd_ports_eth_addr: [[libc::uint8_t; 6usize]; RTE_MAX_ETHPORTS as usize];

    static mut l2fwd_dst_ports: [libc::uint32_t; RTE_MAX_ETHPORTS as usize];

    static mut l2fwd_tx_buffers: [*mut rte::raw::rte_eth_dev_tx_buffer; RTE_MAX_ETHPORTS as usize];

    static mut l2fwd_timer_period: libc::int64_t;

    fn l2fwd_main_loop(rx_port_list: *const libc::uint32_t, n_rx_port: libc::c_uint)
        -> libc::c_int;
}

fn l2fwd_launch_one_lcore(conf: Option<&Conf>) -> i32 {
    let lcore_id = lcore::current().unwrap();
    let qconf = &conf.unwrap().queue_conf[*lcore_id as usize];

    if qconf.n_rx_port == 0 {
        info!("lcore {} has nothing to do", lcore_id);

        return -1;
    }

    info!("entering main loop on lcore {}", lcore_id);

    for portid in &qconf.rx_port_list[..qconf.n_rx_port as usize] {
        info!(" -- lcoreid={} portid={}", lcore_id, portid);
    }

    unsafe { l2fwd_main_loop(qconf.rx_port_list.as_ptr(), qconf.n_rx_port) }
}

extern "C" fn handle_sigint(sig: libc::c_int) {
    match signal::Signal::from_c_int(sig).unwrap() {
        signal::SIGINT | signal::SIGTERM => unsafe {
            println!("Signal {} received, preparing to exit...", sig);

            l2fwd_force_quit = 1;
        },
        _ => info!("unexpect signo: {}", sig),
    }
}

fn handle_signals() -> nix::Result<()> {
    let sig_action = signal::SigAction::new(
        signal::SigHandler::Handler(handle_sigint),
        signal::SaFlags::empty(),
        signal::SigSet::empty(),
    );
    unsafe {
        try!(signal::sigaction(signal::SIGINT, &sig_action));
        try!(signal::sigaction(signal::SIGTERM, &sig_action));
    }

    Ok(())
}

fn prepare_args(args: &mut Vec<String>) -> (Vec<String>, Vec<String>) {
    let program = String::from(Path::new(&args[0]).file_name().unwrap().to_str().unwrap());

    if let Some(pos) = args.iter().position(|arg| arg == "--") {
        let (eal_args, opt_args) = args.split_at_mut(pos);

        opt_args[0] = program;

        (eal_args.to_vec(), opt_args.to_vec())
    } else {
        (args[..1].to_vec(), args.clone())
    }
}

fn main() {
    pretty_env_logger::init();

    handle_signals().expect("fail to handle signals");

    let mut args: Vec<String> = env::args().collect();

    let (eal_args, opt_args) = prepare_args(&mut args);

    debug!("eal args: {:?}, l2fwd args: {:?}", eal_args, opt_args);

    let (enabled_port_mask, rx_queue_per_lcore, timer_period_seconds) = parse_args(&opt_args);

    unsafe {
        l2fwd_enabled_port_mask = enabled_port_mask;
        l2fwd_timer_period = timer_period_seconds as i64 * TIMER_MILLISECOND * 1000;
    }

    // init EAL
    eal::init(&eal_args).expect("fail to initial EAL");

    // create the mbuf pool
    let l2fwd_pktmbuf_pool = mbuf::pktmbuf_pool_create(
        "mbuf_pool",
        NB_MBUF,
        32,
        0,
        mbuf::RTE_MBUF_DEFAULT_BUF_SIZE,
        eal::socket_id(),
    ).expect("fail to initial mbuf pool")
    .as_mut_ref()
    .unwrap();

    let enabled_devices: Vec<ethdev::PortId> = ethdev::devices()
        .filter(|dev| ((1 << dev.portid()) & enabled_port_mask) != 0)
        .collect();

    if enabled_devices.is_empty() {
        eal::exit(
            EXIT_FAILURE,
            "All available ports are disabled. Please set portmask.\n",
        );
    }

    let mut last_port = 0;
    let mut nb_ports_in_mask = 0;

    // Each logical core is assigned a dedicated TX queue on each port.
    for dev in &enabled_devices {
        let portid = dev.portid();

        if (nb_ports_in_mask % 2) != 0 {
            unsafe {
                l2fwd_dst_ports[portid as usize] = last_port as u32;
                l2fwd_dst_ports[last_port as usize] = portid as u32;
            }
        } else {
            last_port = portid;
        }

        nb_ports_in_mask += 1;

        let info = dev.info();

        debug!("found port #{} with `{}` drive", portid, info.driver_name());
    }

    if (nb_ports_in_mask % 2) != 0 {
        println!("Notice: odd number of ports in portmask.");

        unsafe {
            l2fwd_dst_ports[last_port as usize] = last_port as u32;
        }
    }

    let mut conf = Conf::default();

    let mut rx_lcore_id = lcore::id(0);

    // Initialize the port/queue configuration of each logical core
    for dev in &enabled_devices {
        let portid = dev.portid();

        loop {
            if let Some(id) = rx_lcore_id.next() {
                if conf.queue_conf[*rx_lcore_id as usize].n_rx_port == rx_queue_per_lcore {
                    rx_lcore_id = id
                }
            }

            break;
        }

        // Assigned a new logical core in the loop above.
        let qconf = &mut conf.queue_conf[*rx_lcore_id as usize];

        qconf.rx_port_list[qconf.n_rx_port as usize] = portid as u32;
        qconf.n_rx_port += 1;

        println!("Lcore {}: RX port {}", rx_lcore_id, portid);
    }

    let port_conf = ethdev::EthConf::default();

    // Initialise each port
    for dev in &enabled_devices {
        let portid = dev.portid() as usize;

        // init port
        print!("Initializing port {}... ", portid);

        dev.configure(1, 1, &port_conf)
            .expect(&format!("fail to configure device: port={}", portid));

        let mac_addr = dev.mac_addr();

        unsafe {
            l2fwd_ports_eth_addr[portid] = *mac_addr.octets();
        }

        // init one RX queue
        dev.rx_queue_setup(0, conf.nb_rxd, None, l2fwd_pktmbuf_pool)
            .expect(&format!("fail to setup device rx queue: port={}", portid));

        // init one TX queue on each port
        dev.tx_queue_setup(0, conf.nb_txd, None)
            .expect(&format!("fail to setup device tx queue: port={}", portid));

        // Initialize TX buffers
        let buf = ethdev::alloc_buffer(MAX_PKT_BURST, dev.socket_id())
            .as_mut_ref()
            .expect(&format!("fail to allocate buffer for tx: port={}", portid));

        buf.count_err_packets().expect(&format!(
            "failt to set error callback for tx buffer: port={}",
            portid
        ));

        unsafe {
            l2fwd_tx_buffers[portid] = buf;
        }

        // Start device
        dev.start()
            .expect(&format!("fail to start device: port={}", portid));

        println!("Done: ");

        dev.promiscuous_enable();

        println!(
            "  Port {}, MAC address: {} (promiscuous {})",
            portid,
            mac_addr,
            dev.is_promiscuous_enabled()
                .map(|enabled| if enabled { "enabled" } else { "disabled" })
                .expect(&format!(
                    "fail to enable promiscuous mode for device: port={}",
                    portid
                ))
        );
    }

    check_all_ports_link_status(&enabled_devices);

    // launch per-lcore init on every lcore
    launch::mp_remote_launch(l2fwd_launch_one_lcore, Some(&conf), false).unwrap();

    launch::mp_wait_lcore();

    for dev in &enabled_devices {
        print!("Closing port {}...", dev.portid());
        dev.stop();
        dev.close();
        println!(" Done");

        if let Some(buf) = (unsafe { l2fwd_tx_buffers[dev.portid() as usize] }).as_mut_ref() {
            buf.free();
        }
    }

    println!("Bye...");
}
