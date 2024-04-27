#[macro_use]
mod utils;
mod client;
mod config;
mod state;
mod threadpool;

use std::error;
use std::sync;

fn main() -> Result<(), Box<dyn error::Error>> {
    // Set up configuration
    let config = config::Config::new()?;

    if config.debug {
        println!("DEBUG:: {:?}", config);
    }

    let requests = config.to_vector()?;
    let requested: usize = requests.clone().into_iter().map(|c| c.iterations).sum();

    // Housekeeping pool for state and signals.
    let housekeeping = threadpool::ThreadPool::new(2);

    // Set up workers pool for executing requests.
    let workers = threadpool::ThreadPool::new(config.pool_size);

    // Set up state
    let (state_tx, state_rx) = sync::mpsc::channel();

    let mut state = state::State::new(requested);
    housekeeping.execute(move || {
        while !state.done() {
            let (s, f, e, code, kill) = state_rx.recv().unwrap();
            if kill {
                state.kill();
            }

            state.increment(s, f, e);

            if config.verbose {
                println!("code={} {}", code, state.string())
            }
        }

        if !config.verbose {
            println!("{}", state.string());
        }
    });

    // Execute requests
    for request in requests {
        for _ in 0..request.iterations {
            let request = request.clone();
            let state_tx = state_tx.clone();

            workers.execute(move || {
                request.sleep();

                // Keep track
                let mut state: (usize, usize, usize, u16, bool) = (0, 0, 0, 0, false);

                // Set up client
                let client = client::Client::new(request);
                if client.is_err() {
                    state.2 = 1;
                    let _ = state_tx.send(state);
                    if config.debug {
                        println!("DEBUG:: {:?}", client.unwrap_err());
                    }
                    return;
                }

                let client = client.unwrap();
                match client.execute() {
                    Ok(status) => {
                        state.3 = status;
                        if status >= 200 || status < 300 {
                            state.0 = 1;
                        } else {
                            state.1 = 1;
                        }
                    }
                    Err(_) => state.2 = 1,
                }

                let _ = state_tx.send(state);
            });
        }
    }

    /*
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

        let counter = sync::Arc::new(sync::Mutex::new(State {
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

        let (wait_tx, wait_rx) = sync::mpsc::channel();
        let hup_waiter = || {
            // match wait_rx.recv() {
            //     Ok(done) => {
            //         if done {
            //             return;
            //         }
            //     }
            //     Err(err) => {
            //         panic!("{}", err);
            //     }
            // }

            let sighup = sync::Arc::new(AtomicBool::new(false));
            flag::register(signal_hook::consts::SIGHUP, sync::Arc::clone(&sighup)).unwrap();

            let counts = counter.clone();
            thread::spawn(move || {
                while !sighup.load(atomic::Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }

                print_results(counts, start_time);
                // Reset to continue waiting
            });
        };

        hup_waiter();

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

        let _ = wait_tx.send(true);
    */
    return Ok(());
}

// fn final_results(counts: sync::Arc<sync::Mutex<State>>, start_time: time::Instant) {
//     let counts = counts.lock().unwrap();
//     println!("-------------------------");
//     println!("      Requested: {:?}", counts.requested);
//     println!("-------------------------");
//     println!("        success: {:?}", counts.success);
//     println!("        failure: {:?}", counts.fail);
//     println!("         errors: {:?}", counts.error);
//     println!("----------------------");
//     println!("      Processed: {:?}", counts.processed);
//     println!("----------------------");
//     let duration = time::Instant::now() - start_time;
//     println!("Run took: {:?}", duration);
// }
//
// fn print_results(counts: sync::Arc<sync::Mutex<State>>, start_time: time::Instant) {
//     let counts = counts.lock().unwrap();
//     let duration = time::Instant::now() - start_time;
//     println!(
//         "SIGHUP Report: requeted={} success={} failed={} errors={} processed={} for={:?} ...continuing.",
//         counts.requested, counts.success, counts.fail, counts.error, counts.processed, duration,
//     );
// }
