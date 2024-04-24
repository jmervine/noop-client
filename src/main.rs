#[macro_use]
mod utils;
mod client;
mod config;

use clap::Parser;
use config::Config;

use std::error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

struct Counter {
    total: usize,
    success: usize,
    fail: usize,
    error: usize,
}

static mut VERBOSE: bool = false;

fn main() -> Result<(), Box<dyn error::Error>> {
    let start_time = time::Instant::now();
    let mut handles = vec![];

    let counter = Arc::new(Mutex::new(Counter {
        total: 0,
        success: 0,
        fail: 0,
        error: 0,
    }));

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
    //let expected: usize = configs.clone().into_iter().map(|c| c.iterations).sum();

    for config in configs {
        for _ in 0..config.iterations {
            let shared_counter = Arc::clone(&counter);
            let config = config.clone();
            let joiner = thread::spawn(move || {
                let mut locked_counter = shared_counter.lock().unwrap();

                if config.sleep() > time::Duration::ZERO {
                    thread::sleep(config.sleep());
                }

                let (method, endpoint, headers) = (config.method, config.endpoint, config.headers);
                let client = client::Client::new(method.clone(), endpoint.clone(), headers.clone());
                if client.is_err() {
                    eprintln!(
                        "Failed to create client method='{}', endpoint='{}', headers='{:?}'",
                        method, endpoint, headers,
                    );
                    return;
                }

                if let Ok(code) = client.unwrap().execute() {
                    if code >= 200 || code < 300 {
                        locked_counter.success += 1;
                    } else {
                        locked_counter.fail += 1;
                    }
                } else {
                    locked_counter.error += 1;
                }
            });
            handles.push(joiner);
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    print_results(counter, start_time);

    return Ok(());
}

fn print_results(counts: Arc<Mutex<Counter>>, start_time: time::Instant) {
    let counts = counts.lock().unwrap();
    println!("-------------------------");
    println!("  Requests sent: {:?}", counts.total);
    println!("-------------------------");
    println!("        success: {:?}", counts.success);
    println!("        failure: {:?}", counts.fail);
    println!("         errors: {:?}", counts.error);
    println!("----------------------");
    let duration = time::Instant::now() - start_time;
    println!("Run took: {:?}", duration);
}
