#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

extern crate rte;

use std::process::exit;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use std::path::Path;

const MAX_RX_QUEUE_PER_LCORE: u32 = 16;

const MAX_TIMER_PERIOD: u32 = 86400; /* 1 day max */

const NB_MBUF: u32 = 8192;

// display usage
fn l2fwd_usage(program: &String, opts: getopts::Options) -> ! {
    let brief = format!("Usage: {} [EAL options] -- [options]", program);

    print!("{}", opts.usage(&brief));

    exit(-1);
}

// Parse the argument given in the command line of the application
fn l2fwd_parse_args(program: &String, args: &Vec<String>) -> (u32, u32, Duration) {
    let mut opts = getopts::Options::new();

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

    let matches = match opts.parse(args) {
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

    let args: Vec<String> = env::args().collect();
    let program = String::from(Path::new(&args[0]).file_name().unwrap().to_str().unwrap());

    let (eal_args, opt_args) = if let Some(pos) = args.iter().position(|arg| arg == "--") {
        args.split_at(pos)
    } else {
        (&args[..1], &args[1..])
    };

    let (l2fwd_enabled_port_mask, l2fwd_rx_queue_per_lcore, timer_period) =
        l2fwd_parse_args(&program, &Vec::from(opt_args));

    // init EAL
    rte::eal_init(&Vec::from(eal_args));

    // create the mbuf pool
    let l2fwd_pktmbuf_pool = rte::pktmbuf_pool_create("mbuf_pool",
                                                      NB_MBUF,
                                                      32,
                                                      0,
                                                      RTE_MBUF_DEFAULT_BUF_SIZE,
                                                      rte::socket_id());
}
