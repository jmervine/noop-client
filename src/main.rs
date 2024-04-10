extern crate clap;
extern crate reqwest;

use std::time::Instant;
use clap::Parser;
use reqwest::blocking::Response;

#[macro_use]
mod macros;

mod config;
use config::*;

mod client;
use client::*;

static mut VERBOSE: bool = false;

fn main() {
    let start_time = Instant::now();

    let config: Config = Config::parse();
    set_verbose!(config.verbose);

    let configs = config.to_vec();
    echeck!(configs);

    let mut results: Vec<(Option<Response>, Option<ClientError>)> = vec![];
    for cfg in configs.unwrap() {
        let client = Client::new(
            &cfg.method,
            &cfg.url,
            cfg.headers,
            cfg.iterations,
        );
        echeck!(client);

        let mut res = client.unwrap().run();
        debugln!(format!("+ batch count: {}", res.len()));
        results.append(&mut res);
    }

    let duration = Instant::now() - start_time;

    unless_debugln!("------");
    unless_debugln!(format!("Requests made: {}", results.len()));
    println!("Run took: {:?}", duration);

}
