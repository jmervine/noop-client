extern crate clap;
extern crate reqwest;

mod config;
mod client;

use std::time::Instant;
use clap::Parser;
use std::sync::mpsc;
use reqwest::Response;

use config::*;
use client::*;

#[macro_use]
mod macros;

// ----- 
static mut VERBOSE: bool = false;

#[tokio::main]
async fn main() {
    let start_time = Instant::now();

    // Create a channel for communication
    // let (sender, recv) = mpsc::channel();
    let (sender, recv) = mpsc::channel();

    let config: Config = Config::parse();
    set_verbose!(config.verbose);

    // coroutine::scope( |scope| {
    //     let send = sender.clone();

        let r_configs = config.to_vec();
        echeck!(r_configs);

        let configs = r_configs.unwrap();

        let expect: usize = configs.clone().into_iter().map( |c| c.iterations).sum();

        let mut results: Vec<(Option<Response>, Option<ClientError>)> = vec![];
        for c in configs {
            let send = sender.clone();
            tokio::spawn( async move {
                let r_client = Client::new(&c.method, &c.url, c.headers, c.iterations);
                echeck!(r_client);

                let client = r_client.unwrap();
                let result = client.run().await;

                let sent = send.send(result);
                echeck!(sent);
            });
        }

        while results.len() < expect {
            let r_result = recv.recv();
            echeck!(r_result);
            let mut result = r_result.unwrap();

            results.append(&mut result);
        }

        debug!(format!("{:?}", results));
        println!("Received result: {:?}", results.len());

        let duration = Instant::now() - start_time;
        println!("Run took: {:?}", duration);
    // });
}
