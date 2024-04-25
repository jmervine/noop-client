#[macro_use]
mod utils;
mod client;
mod config;

use config::Config;

use clap::Parser;
use scoped_pool::Pool;
use signal_hook::flag;
use std::error;
use std::sync;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time;

struct Counter {
    requested: usize,
    processed: usize,
    success: usize,
    fail: usize,
    error: usize,
}

static mut VERBOSE: bool = false;

fn main() -> Result<(), Box<dyn error::Error>> {
    let sigint = sync::Arc::new(AtomicBool::new(false));
    flag::register(signal_hook::consts::SIGINT, sync::Arc::clone(&sigint))?;

    let config: Config = Config::parse();
    if !config.is_valid() {
        eprintln!(
            "Error: Configuration is invalid, see '--help', in:\n{:#?}",
            config
        );
        std::process::exit(1);
    }
    set_verbose!(config.verbose);

    let configs = config.to_vector()?;
    let requested: usize = configs.clone().into_iter().map(|c| c.iterations).sum();
    let threadpool = Pool::new(config.pool_size);

    let counter = sync::Arc::new(sync::Mutex::new(Counter {
        requested: requested,
        processed: 0,
        success: 0,
        fail: 0,
        error: 0,
    }));

    let (count_tx, count_rx) = sync::mpsc::channel();
    let (kill_tx, kill_rx) = sync::mpsc::channel();

    let start_time = time::Instant::now();
    // TODO: Pretty print.
    println!("Starting noop-client with {:?}", config);
    for config in configs {
        for _ in 0..config.iterations {
            let shared_counter = sync::Arc::clone(&counter);
            let config = config.clone();

            let count_tx = count_tx.clone();
            threadpool.spawn(move || {
                let _ = count_tx.send(1);

                if config.sleep() > time::Duration::ZERO {
                    thread::sleep(config.sleep());
                }

                let (method, endpoint, headers) = (config.method, config.endpoint, config.headers);
                let client = client::Client::new(method.clone(), endpoint.clone(), headers.clone());
                if client.is_err() {
                    let mut locked_counter = shared_counter.lock().unwrap();
                    locked_counter.error += 1;
                    eprintln!(
                        "Failed to create client method='{}', endpoint='{}', headers='{:?}'",
                        method, endpoint, headers,
                    );
                    return;
                }

                let code = client.unwrap().execute();
                let mut locked_counter = shared_counter.lock().unwrap();
                match code {
                    Ok(code) => {
                        if code >= 200 || code < 300 {
                            locked_counter.success += 1;
                        } else {
                            locked_counter.fail += 1;
                        }
                    }
                    Err(_) => {
                        locked_counter.error += 1;
                    }
                }
                locked_counter.processed += 1;
            });
        }
    }

    let _ = hup_waiter(counter.clone(), start_time);

    let kill = kill_tx.clone();
    thread::spawn(move || {
        while !sigint.load(atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        println!("SIGINT called, shutting down...");
        let _ = kill.send(true);
    });

    let kill = kill_tx.clone();
    thread::spawn(move || {
        let mut count: usize = 0;
        while count < requested {
            if let Ok(_) = count_rx.recv() {
                count += 1;
            }
        }
        let _ = kill.send(true);
    });

    if let Ok(_) = kill_rx.recv() {
        final_results(counter, start_time);
    }

    return Ok(());
}

fn final_results(counts: sync::Arc<sync::Mutex<Counter>>, start_time: time::Instant) {
    let counts = counts.lock().unwrap();
    println!("-------------------------");
    println!("      Requested: {:?}", counts.requested);
    println!("-------------------------");
    println!("        success: {:?}", counts.success);
    println!("        failure: {:?}", counts.fail);
    println!("         errors: {:?}", counts.error);
    println!("----------------------");
    println!("      Processed: {:?}", counts.processed);
    println!("----------------------");
    let duration = time::Instant::now() - start_time;
    println!("Run took: {:?}", duration);
}

fn print_results(counts: sync::Arc<sync::Mutex<Counter>>, start_time: time::Instant) {
    let counts = counts.lock().unwrap();
    let duration = time::Instant::now() - start_time;
    println!(
        "SIGHUP Report: requeted={} success={} failed={} errors={} processed={} for={:?} ...continuing.",
        counts.requested, counts.success, counts.fail, counts.error, counts.processed, duration,
    );
}

fn hup_waiter(
    counter: sync::Arc<sync::Mutex<Counter>>,
    start_time: time::Instant,
) -> Result<(), Box<dyn error::Error>> {
    let sighup = sync::Arc::new(AtomicBool::new(false));
    flag::register(signal_hook::consts::SIGHUP, sync::Arc::clone(&sighup))?;

    let counts = counter.clone();
    thread::spawn(move || {
        while !sighup.load(atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        print_results(counts, start_time);
        // Reset to continue waiting

        let _ = hup_waiter(counter, start_time);
    });

    return Ok(());
}
