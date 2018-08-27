use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::str::FromStr;
use std::process;
use std::sync::mpsc::{Sender, channel};
use std::thread;

const MAX: u16 = 65535;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(0);
        }
    });

    let num_threads = arguments.threads;
    let ip_address = arguments.ip_address;
    let (tx, rx) = channel();

    for i in 0..num_threads {
        let tx = tx.clone();

        thread::spawn(move | | {
            scan(tx, i, ip_address, num_threads);
        });
    }

    drop(tx); // So that the tx is only in the other threads

    let mut out = vec![];
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}

struct Arguments {
    flag: String,
    ip_address: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }

        let ip_address = args[1].clone();
        if let Ok(ip_address) = IpAddr::from_str(&ip_address) {
            Ok(Arguments {
                flag: String::from(""),
                ip_address,
                threads: 4
            })
        } else {
            let flag = args[1].clone();
            if (flag.contains("-h") || flag.contains("-help")) &&
                args.len() == 2 {
                println!("Usage: -j to select how many threads you want
                \r\n      -h or -help to show this help message");
                return Err("help");
            } else if flag.contains("-h") || flag.contains("-help") {
                return Err("too many arguments");
            } else if flag.contains("-j") {
                let ip_address = match IpAddr::from_str(&args[3]) {
                    Ok(ip_address) => ip_address,
                    Err(_) => return Err("not a valid IP Address; must be IPv4 or IPv6"),
                };

                let threads = match args[2].parse::<u16>() {
                    Ok(thread_count) => thread_count,
                    Err(_) => return Err("failed to parse thread number"),
                };

                Ok(Arguments {
                    flag,
                    ip_address,
                    threads,
                })
            } else {
                Err("invalid syntax")
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, ip_address: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((ip_address, port)) {
            Ok(_) => {
                print!("."); // Indicate that it finds one open port
                io::stdout().flush().unwrap(); // TODO: What does this do?
                tx.send(port).unwrap();
            },
            Err(_) => {},
        };

        if (MAX - port) <= num_threads {
            break;
        }

        port += num_threads;
    }
}
