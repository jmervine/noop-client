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
                    Err(err) => {
                        state.2 = 1;
                        if config.errors {
                            eprintln!(
                                "method={} endpoint='{}' error='{}'",
                                &client.method, &client.endpoint, err,
                            )
                        }
                    }
                }

                let _ = state_tx.send(state);
            });
        }
    }
    Ok(())
}
