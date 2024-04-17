mod client;
mod config;

#[macro_use]
mod utils;

use clap::Parser;

use core::time;
use signal_hook as signals;
use std::sync::mpsc;
use std::time::Instant;

use client::*;
use config::*;

// -----
static mut VERBOSE: bool = false;

#[tokio::main]
async fn main() -> Result<(), utils::Errors> {
    let start_time = Instant::now();

    let mut t: usize = 0;
    let mut s: usize = 0;
    let mut f: usize = 0;
    let mut e: usize = 0;

    let (sender, recv) = mpsc::channel();

    let config: Config = Config::parse();
    if !config.is_valid() {
        eprintln!(
            "Error: Configuration is invalid, see '--help', in:\n{:#?}",
            config
        );
        std::process::exit(1);
    }
    set_verbose!(config.verbose());

    let configs = config.to_vector()?;
    let expect: usize = configs.clone().into_iter().map(|c| c.iterations).sum();

    let mut sigs =
        signals::iterator::Signals::new(&[signals::consts::SIGINT, signals::consts::SIGHUP])
            .expect("Error setting up signal handler.");

    std::thread::spawn(move || {
        for sig in sigs.forever() {
            println!("Received signal {:?}", sig);

            print_results(t, s, f, e, start_time);
            if sig == signals::consts::SIGINT {
                std::thread::sleep(time::Duration::from_secs(1));
                std::process::abort();
            }
        }
    });

    for c in configs {
        let send = sender.clone();
        tokio::spawn(async move {
            let h = c.clone().headers;
            let client = Client::new(&c.method, &c.endpoint(), h, c.iterations, c.sleep());

            // ( t, s, f, e )
            let mut re: (usize, usize, usize, usize) = (0, 0, 0, 0);

            if client.is_err() {
                eprintln!(
                    "Warning: skipping '{:?}', due to '{:?}'",
                    c,
                    client.unwrap_err()
                );
                let _ = send.send((1, 0, 0, 1)); // continue
            } else {
                let c = client.unwrap();
                let resp = c.run().await;

                for r in resp {
                    match r {
                        Ok(r) => {
                            let code = r.status().as_u16();
                            debug!(format!(
                                "code={:} method={} path={:}",
                                code, c.method, c.endpoint,
                            ));
                            if code >= 200 && code < 300 {
                                re.1 += 1;
                            } else {
                                re.2 += 1;
                            }
                        }
                        Err(err) => {
                            debug!(format!(
                                "method={} path={:} error='{:?}'",
                                c.method, c.endpoint, err
                            ));
                            re.3 += 1;
                        }
                    }
                    re.0 += 1;
                }

                let _ = send.send(re);
            }
        });
    }

    while t < expect {
        match recv.recv() {
            Ok(re) => {
                t += re.0;
                s += re.1;
                f += re.2;
                e += re.3;
            }
            Err(_) => {
                // force exit
                t = expect;
            }
        }
    }

    print_results(t, s, f, e, start_time);

    Ok(())
}

fn print_results(t: usize, s: usize, f: usize, e: usize, start_time: Instant) {
    println!("-------------------------");
    println!("  Requests sent: {:?}", t);
    println!("-------------------------");
    println!("        success: {:?}", s);
    println!("        failure: {:?}", f);
    println!("         errors: {:?}", e);
    println!("----------------------");
    let duration = Instant::now() - start_time;
    println!("Run took: {:?}", duration);
}
