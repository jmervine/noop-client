mod client;
mod config;

#[macro_use]
mod utils;

use clap::Parser;
use std::sync::mpsc;
use std::time::Instant;

use client::*;
use config::*;

// -----
static mut VERBOSE: bool = false;

#[tokio::main]
async fn main() -> Result<(), utils::Errors> {
    let start_time = Instant::now();

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

    let mut results = vec![];
    for c in configs {
        let send = sender.clone();
        tokio::spawn(async move {
            let client = Client::new(&c.method, &c.endpoint(), c.headers, c.iterations);
            if client.is_err() {
                let _ = send.send(response_error_vector!(client.unwrap_err().to_string()));
            } else {
                let resp = client.unwrap().run().await;
                let _ = send.send(resp);
            }
        });
    }

    while results.len() < expect {
        let mut sent = match recv.recv() {
            Ok(sent) => sent,
            Err(err) => response_error_vector!(err.to_string()),
        };

        results.append(&mut sent);
    }

    // [0]total, [1]pass, [2]fail, [3]error
    let mut count: (usize, usize, usize, usize) = (0, 0, 0, 0);
    let mut pass: Vec<reqwest::Response> = vec![];
    let mut fail: Vec<reqwest::Response> = vec![];
    let mut error: Vec<utils::Errors> = vec![];

    for result in results.into_iter() {
        count.0 += 1;
        match result {
            Ok(response) => {
                let s = response.status();
                let code = s.as_u16();

                if code >= 200 && code < 300 {
                    count.1 += 1;
                    pass.push(response);
                } else {
                    count.2 += 1;
                    fail.push(response);
                }
            }
            Err(err) => {
                count.3 += 1;

                // Report errors on debug
                debug!(format!("{:?}", err));
                error.push(err);
            }
        }
    }

    println!("-------------------------");
    println!("  Requests sent: {:?}", count.0);
    println!("-------------------------");
    println!("        success: {:?}", count.1);
    println!("        failure: {:?}", count.2);
    println!("         errors: {:?}", count.3);
    println!("----------------------");

    let duration = Instant::now() - start_time;
    println!("Run took: {:?}", duration);

    Ok(())
}
