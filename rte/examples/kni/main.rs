#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate libc;
extern crate nix;

#[macro_use]
extern crate rte;

use std::io;
use std::io::prelude::*;
use std::env;
use std::fmt;
use std::mem;
use std::ptr;
use std::cmp;
use std::result;
use std::process;
use std::path::Path;
use std::str::FromStr;

use nix::sys::signal;

use rte::*;
use rte::ethdev::{EthDevice, EthDeviceInfo};

const EXIT_FAILURE: i32 = -1;

// Max size of a single packet
const MAX_PACKET_SZ: u32 = 2048;

// Size of the data buffer in each mbuf
const MBUF_DATA_SZ: u32 = MAX_PACKET_SZ + RTE_PKTMBUF_HEADROOM;

// Number of mbufs in mempool that is created
const NB_MBUF: u32 = 8192;

// How many packets to attempt to read from NIC in one go
const PKT_BURST_SZ: u32 = 32;

// How many objects (mbufs) to keep in per-lcore mempool cache
const MEMPOOL_CACHE_SZ: u32 = PKT_BURST_SZ;

// Number of RX ring descriptors
const NB_RXD: u16 = 128;

// Number of TX ring descriptors
const NB_TXD: u16 = 512;

// Total octets in ethernet header
const KNI_ENET_HEADER_SIZE: u32 = 14;

// Total octets in the FCS
const KNI_ENET_FCS_SIZE: u32 = 4;

const KNI_MAX_KTHREAD: usize = 32;

#[repr(C)]
struct Struct_kni_port_params {
    // Port ID
    port_id: libc::uint8_t,
    // lcore ID for RX
    lcore_rx: libc::c_uint,
    // lcore ID for TX
    lcore_tx: libc::c_uint,
    // Number of lcores for KNI multi kernel threads
    nb_lcore_k: libc::uint32_t,
    // Number of KNI devices to be created
    nb_kni: libc::uint32_t,
    // lcore ID list for kthreads
    lcore_k: [libc::c_uint; KNI_MAX_KTHREAD],
    // KNI context pointers
    kni: [kni::RawKniDevicePtr; KNI_MAX_KTHREAD],
}

struct Conf {
    // mask of enabled ports
    enabled_port_mask: u32,

    promiscuous_on: bool,

    port_params: [*mut Struct_kni_port_params; RTE_MAX_ETHPORTS as usize],
}

impl fmt::Debug for Conf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            for p in self.port_params.iter().filter(|p| !p.is_null()) {
                let conf = &**p;

                try!(write!(f, "Port ID: {}\n", conf.port_id));
                try!(write!(f,
                            "  Rx lcore ID: {}, Tx lcore ID: {}\n",
                            conf.lcore_rx,
                            conf.lcore_tx));

                for lcore_id in &conf.lcore_k[..conf.nb_lcore_k as usize] {
                    try!(write!(f, "    Kernel thread lcore ID: {}\n", lcore_id));
                }
            }
        }

        Ok(())
    }
}

impl Conf {
    fn new() -> Conf {
        unsafe { mem::zeroed() }
    }

    fn parse_config(&mut self, arg: &str) -> result::Result<(), String> {
        let mut fields = arg.split(',')
            .map(|s| u32::from_str(s).expect("Invalid config parameters"));

        let port_id = try!(fields.next().ok_or("Invalid config parameter, missed port_id field"));

        if port_id > RTE_MAX_ETHPORTS {
            return Err(format!("Port ID {} could not exceed the maximum {}",
                               port_id,
                               RTE_MAX_ETHPORTS));
        }

        if !self.port_params[port_id as usize].is_null() {
            return Err(format!("Port {} has been configured", port_id));
        }

        let param: &mut Struct_kni_port_params = rte_new!(Struct_kni_port_params);

        param.port_id = port_id as u8;
        param.lcore_rx = try!(fields.next()
            .ok_or("Invalid config parameter, missed lcore_rx field"));
        param.lcore_tx = try!(fields.next()
            .ok_or("Invalid config parameter, missed lcore_tx field"));

        if param.lcore_rx >= RTE_MAX_LCORE || param.lcore_tx >= RTE_MAX_LCORE {
            return Err(format!("lcore_rx {} or lcore_tx {} ID could not exceed the maximum {}",
                               param.lcore_rx,
                               param.lcore_tx,
                               RTE_MAX_LCORE));
        }

        let lcores: Vec<u32> = fields.collect();

        unsafe {
            ptr::copy_nonoverlapping(lcores.as_ptr(), param.lcore_k.as_mut_ptr(), lcores.len());
        }

        param.nb_lcore_k = lcores.len() as u32;

        self.port_params[port_id as usize] = param;

        Ok(())
    }
}

extern "C" fn handle_sigint(sig: signal::SigNum) {
    match sig {
        // When we receive a USR1 signal, print stats
        signal::SIGUSR1 => unsafe {
            kni_print_stats();
        },
        // When we receive a USR2 signal, reset stats
        signal::SIGUSR2 => {
            unsafe {
                kni_stats = mem::zeroed();
            }

            println!("**Statistics have been reset**");
        }
        // When we receive a TERM or SIGINT signal, stop kni processing
        signal::SIGINT | signal::SIGTERM => {
            unsafe {
                kni_stop = 1;
            }

            println!("SIGINT or SIGTERM is received, and the KNI processing is going to stop\n");
        }
        _ => info!("unexpect signo: {}", sig),
    }
}

/// Associate signal_hanlder function with USR signals
fn handle_signals() -> nix::Result<()> {
    let sig_action = signal::SigAction::new(signal::SigHandler::Handler(handle_sigint),
                                            signal::SaFlag::empty(),
                                            signal::SigSet::empty());
    unsafe {
        try!(signal::sigaction(signal::SIGUSR1, &sig_action));
        try!(signal::sigaction(signal::SIGUSR2, &sig_action));
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

// display usage
fn print_usage(program: &String, opts: getopts::Options) -> ! {
    let brief = format!("Usage: {} [EAL options] -- [options]", program);

    print!("{}", opts.usage(&brief));

    process::exit(-1);
}

// Parse the argument given in the command line of the application
fn parse_args(args: &Vec<String>) -> result::Result<Conf, String> {
    let mut opts = getopts::Options::new();
    let program = args[0].clone();

    opts.optflag("h", "help", "print this help menu");
    opts.optopt("p",
                "",
                "hexadecimal bitmask of ports to configure",
                "PORTMASK");
    opts.optflag("P", "", "enable promiscuous mode");
    opts.optmulti("c",
                  "config",
                  "port and lcore configurations",
                  "port,lcore_rx,lcore_tx,lcore_kthread...");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            println!("Invalid option specified, {}", err);

            print_usage(&program, opts);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
    }

    let mut conf = Conf::new();

    if let Some(arg) = matches.opt_str("p") {
        match u32::from_str_radix(arg.as_str(), 16) {
            Ok(mask) if mask != 0 => conf.enabled_port_mask = mask,
            _ => {
                println!("invalid portmask, {}", arg);

                print_usage(&program, opts);
            }
        }
    }

    conf.promiscuous_on = matches.opt_present("P");

    for arg in matches.opt_strs("c") {
        try!(conf.parse_config(&arg));
    }

    debug!("{:?}", conf);

    Ok(conf)
}

// Initialize KNI subsystem
fn init_kni(conf: &Conf) {
    let num_of_kni_ports = unsafe {
        conf.port_params
            .iter()
            .filter(|param| !param.is_null())
            .fold(0, |acc, param| acc + cmp::max((**param).nb_lcore_k, 1))
    };

    // Invoke rte KNI init to preallocate the ports
    kni::init(num_of_kni_ports as usize);
}

// Initialise a single port on an Ethernet device
fn init_port(conf: &Conf,
             dev: ethdev::PortId,
             port_conf: &ethdev::EthConf,
             pktmbuf_pool: &mut mempool::RawMemoryPool) {
    let portid = dev.portid();

    // Initialise device and RX/TX queues
    info!("Initialising port {} ...", portid);

    dev.configure(1, 1, &port_conf)
        .expect(&format!("fail to configure device: port={}", portid));

    // init one RX queue
    dev.rx_queue_setup(0, NB_RXD, None, pktmbuf_pool)
        .expect(&format!("fail to setup device rx queue: port={}", portid));

    // init one TX queue on each port
    dev.tx_queue_setup(0, NB_TXD, None)
        .expect(&format!("fail to setup device tx queue: port={}", portid));

    // Start device
    dev.start().expect(&format!("fail to start device: port={}", portid));

    info!("Done: ");

    if conf.promiscuous_on {
        dev.promiscuous_enable();
    }
}

extern "C" fn kni_change_mtu(port_id: u8, new_mtu: libc::c_uint) -> libc::c_int {
    debug!("port {} change MTU to {}", port_id, new_mtu);

    let nb_sys_ports = ethdev::count();

    if port_id > nb_sys_ports || port_id as u32 > RTE_MAX_ETHPORTS {
        error!("Invalid port id {}", port_id);

        return -libc::EINVAL;
    }

    if new_mtu > ETHER_MAX_LEN {
        let dev = port_id as ethdev::PortId;

        dev.stop();

        // Set new MTU
        let mut port_conf = ethdev::EthConf::default();

        let mut rxmode: ethdev::EthRxMode = Default::default();

        rxmode.max_rx_pkt_len = new_mtu + KNI_ENET_HEADER_SIZE + KNI_ENET_FCS_SIZE;

        port_conf.rxmode = Some(rxmode);

        if let Err(err) = dev.configure(1, 1, &port_conf) {
            error!("Fail to reconfigure port {}, {}", port_id, err);

            if let Error::RteError(errno) = err {
                return errno;
            }
        }

        if let Err(err) = dev.start() {
            error!("Failed to start port {}, {}", port_id, err);

            if let Error::RteError(errno) = err {
                return errno;
            }
        }
    }

    0
}

extern "C" fn kni_config_network_interface(port_id: u8, if_up: u8) -> libc::c_int {
    debug!("port {} change status to {}",
           port_id,
           if if_up != 0 {
               "up"
           } else {
               "down"
           });

    let nb_sys_ports = ethdev::count();

    if port_id > nb_sys_ports || port_id as u32 > RTE_MAX_ETHPORTS {
        error!("Invalid port id {}", port_id);

        return -libc::EINVAL;
    }

    let dev = port_id as ethdev::PortId;

    dev.stop();

    if if_up != 0 {
        if let Err(err) = dev.start() {
            error!("Failed to start port {}, {}", port_id, err);

            if let Error::RteError(errno) = err {
                return errno;
            }
        }
    }

    0
}

fn kni_alloc(conf: &Conf, dev: ethdev::PortId, pktmbuf_pool: &mut mempool::RawMemoryPool) {
    let portid = dev.portid();

    let param = unsafe {
        if conf.port_params[portid as usize].is_null() {
            return;
        }

        &mut *conf.port_params[portid as usize]
    };

    param.nb_kni = cmp::max(param.nb_lcore_k, 1);

    for i in 0..param.nb_kni {
        let name = if param.nb_lcore_k > 0 {
            format!("vEth{}_{}", portid, i)
        } else {
            format!("vEth{}", portid)
        };

        let mut conf = kni::KniDeviceConf::default();

        conf.name = name.as_str();
        conf.group_id = portid as u16;
        conf.mbuf_size = MAX_PACKET_SZ;

        let mut kni = (if i == 0 {
                // The first KNI device associated to a port is the master,
                // for multiple kernel thread environment.
                let dev_info = dev.info();
                let pci_dev = dev_info.pci_dev()
                    .expect(&format!("port {} haven't PCI dev info", dev.portid()));

                conf.pci_addr = pci_dev.addr;
                conf.pci_id = pci_dev.id;

                let ops = kni::KniDeviceOps {
                    port_id: portid,
                    change_mtu: Some(kni_change_mtu),
                    config_network_if: Some(kni_config_network_interface),
                };

                kni::alloc(pktmbuf_pool, &conf, Some(&ops))
            } else {
                kni::alloc(pktmbuf_pool, &conf, None)
            })
            .expect(&format!("Fail to create kni for port: {}", portid));

        param.kni[i as usize] = kni.into_raw();

        debug!("allocated kni device `{}` @{:p} for port #{}",
               conf.name,
               param.kni[i as usize],
               portid);
    }
}

fn kni_free_kni(conf: &Conf, dev: ethdev::PortId) {
    let portid = dev;

    let param = unsafe {
        if conf.port_params[portid as usize].is_null() {
            return;
        }

        &mut *conf.port_params[portid as usize]
    };

    for kni in &param.kni[..param.nb_kni as usize] {
        let _ = kni::KniDevice::from_raw(*kni);
    }

    dev.stop();
}

// Check the link status of all ports in up to 9s, and print them finally
fn check_all_ports_link_status(enabled_devices: &Vec<ethdev::PortId>) {
    print!("Checking link status");

    const CHECK_INTERVAL: u32 = 100;
    const MAX_CHECK_TIME: usize = 90;

    for _ in 0..MAX_CHECK_TIME {
        if unsafe { kni_stop != 0 } {
            break;
        }

        if enabled_devices.iter().all(|dev| dev.link_nowait().up) {
            break;
        }

        eal::delay_ms(CHECK_INTERVAL);

        print!(".");

        io::stdout().flush().unwrap();
    }

    println!("Done:");

    for dev in enabled_devices {
        let link = dev.link();

        if link.up {
            println!("  Port {} Link Up - speed {} Mbps - {}",
                     dev.portid(),
                     link.speed,
                     if link.duplex {
                         "full-duplex"
                     } else {
                         "half-duplex"
                     })
        } else {
            println!("  Port {} Link Down", dev.portid());
        }
    }
}

#[repr(C)]
struct Struct_kni_interface_stats {
    // number of pkts received from NIC, and sent to KNI
    rx_packets: libc::uint64_t,

    // number of pkts received from NIC, but failed to send to KNI
    rx_dropped: libc::uint64_t,

    // number of pkts received from KNI, and sent to NIC
    tx_packets: libc::uint64_t,

    // number of pkts received from KNI, but failed to send to NIC
    tx_dropped: libc::uint64_t,
}

#[link(name = "kni_core")]
extern "C" {
    static mut kni_stop: libc::c_int;

    static mut kni_port_params_array: *const *mut Struct_kni_port_params;

    static mut kni_stats: [Struct_kni_interface_stats; RTE_MAX_ETHPORTS as usize];

    fn kni_print_stats();

    fn kni_ingress(param: *const Struct_kni_port_params) -> libc::c_int;

    fn kni_egress(param: *const Struct_kni_port_params) -> libc::c_int;
}

extern "C" fn main_loop(conf: *const Conf) -> i32 {
    enum LcoreType<'a> {
        Rx(&'a Struct_kni_port_params),
        Tx(&'a Struct_kni_port_params),
    };

    let lcore_id = lcore::id().unwrap();
    let mut lcore_type: Option<LcoreType> = None;

    for portid in ethdev::devices() {
        if unsafe { (*conf).port_params[portid as usize].is_null() } {
            continue;
        }

        let param = unsafe { &*(*conf).port_params[portid as usize] };

        if param.lcore_rx == lcore_id {
            lcore_type = Some(LcoreType::Rx(param));
            break;
        }

        if (*param).lcore_tx == lcore_id {
            lcore_type = Some(LcoreType::Tx(param));
            break;
        }
    }

    match lcore_type {
        Some(LcoreType::Rx(param)) => {
            info!("Lcore {} is reading from port {}",
                  param.lcore_rx,
                  param.port_id);

            unsafe { kni_ingress(param) }
        }
        Some(LcoreType::Tx(param)) => {
            info!("Lcore {} is writing from port {}",
                  param.lcore_tx,
                  param.port_id);

            unsafe { kni_egress(param) }
        }
        _ => {
            info!("Lcore {} has nothing to do", lcore_id);

            0
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    handle_signals().expect("fail to handle signals");

    let mut args: Vec<String> = env::args().collect();

    let (eal_args, opt_args) = prepare_args(&mut args);

    debug!("eal args: {:?}, l2fwd args: {:?}", eal_args, opt_args);

    // Initialise EAL
    eal::init(&eal_args).expect("Cannot init EAL");

    // Parse application arguments (after the EAL ones)
    let conf = parse_args(&opt_args).expect("Could not parse input parameters");

    unsafe {
        kni_port_params_array = conf.port_params.as_ptr();
    }

    // create the mbuf pool
    let p = mbuf::pktmbuf_pool_create("mbuf_pool",
                                      NB_MBUF,
                                      MEMPOOL_CACHE_SZ,
                                      0,
                                      MBUF_DATA_SZ as u16,
                                      eal::socket_id())
        .expect("fail to initial mbuf pool");

    let pktmbuf_pool = as_mut_ref!(p).unwrap();

    let enabled_devices: Vec<ethdev::PortId> = ethdev::devices()
        .filter(|dev| ((1 << dev.portid()) & conf.enabled_port_mask) != 0)
        .collect();

    if enabled_devices.is_empty() {
        eal::exit(EXIT_FAILURE,
                  "All available ports are disabled. Please set portmask.\n");
    }

    // Initialize KNI subsystem
    init_kni(&conf);

    // Initialise each port
    let port_conf = ethdev::EthConf::default();

    for dev in &enabled_devices {
        init_port(&conf, dev.portid(), &port_conf, pktmbuf_pool);

        kni_alloc(&conf, dev.portid(), pktmbuf_pool);
    }

    check_all_ports_link_status(&enabled_devices);

    // launch per-lcore init on every lcore
    launch::mp_remote_launch(main_loop, Some(&conf), false).unwrap();

    lcore::foreach_slave(|lcore_id| launch::wait_lcore(lcore_id));

    // Release resources
    for dev in &enabled_devices {
        kni_free_kni(&conf, dev.portid());
    }

    kni::close();

    for param in &conf.port_params[..] {
        if !param.is_null() {
            rte_free!(*param);
        }
    }
}
