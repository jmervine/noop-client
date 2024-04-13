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

macro_rules! error_response_vector {
    ($str:expr) => { {
        let ret: Vec<Result<reqwest::Response, utils::Errors>> = vec![
            Err(utils::Errors::Error($str))
        ];
        ret
    } };
}

#[tokio::main]
async fn main() -> Result<(), utils::Errors> {
    let start_time = Instant::now();

    let (sender, recv) = mpsc::channel();

    let config: Config = Config::parse();
    set_verbose!(config.verbose());

    let configs = config.to_vector()?;
    let expect: usize = configs.clone().into_iter().map(|c| c.iterations).sum();

    let mut results = vec![];
    for c in configs {
        let send = sender.clone();
        tokio::spawn(async move {
            let client = Client::new(&c.method, &c.endpoint(), c.headers, c.iterations);
            if client.is_err() {
                let _ = send.send(error_response_vector!(client.unwrap_err().to_string()));
            } else {
                let resp = client.unwrap().run().await;
                let _ = send.send(resp);
            }
            let _ = send.send(error_response_vector!("Failed to get a response for an unknown reason".to_string()));
        });
    }

    while results.len() < expect {
        let mut sent = match recv.recv() {
            Ok(sent) => sent,
            Err(err) => error_response_vector!(err.to_string())
        };

        results.append(&mut sent);
    }

    debug!(format!("{:?}", results));

    // TODO: Create a display mod with an enum for this.
    let breakdown: Vec<_> = results
        .into_iter()
        .map(|r| {
            let mut mapped = "FAILURE";
            if r.is_err() {
                mapped = "ERROR";
            } else {
                let code = r.unwrap().status().as_u16();
                if code >= 200 || code < 300 {
                    mapped = "SUCCESS";
                }
            }

            mapped
        })
        .collect();

    // TODO: There has to be a better way...
    macro_rules! count {
        ($vec:expr, $on:expr) => {
            $vec.clone()
                .into_iter()
                .filter(|s| s == &$on)
                .collect::<Vec<_>>()
                .len()
        };
    }

    println!("  Requests sent: {:?}", breakdown.len());
    println!("---------------------------");
    println!("        success: {:?}", count!(breakdown, "SUCCESS"));
    println!("        failure: {:?}", count!(breakdown, "FAILURE"));
    println!("         errors: {:?}", count!(breakdown, "ERROR"));

    let duration = Instant::now() - start_time;
    println!("Run took: {:?}", duration);

    Ok(())
}
