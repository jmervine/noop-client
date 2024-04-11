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
    some_bad_error!(config.validate());
    set_verbose!(config.verbose());

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
                let r_client = Client::new(&c.method, &c.endpoint(), c.headers, c.iterations);
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

        let n_results = results.len();
        println!("Received result: {:?}", n_results);

        // TODO: Create a display mod with an enum for this.
        let breakdown: Vec<_> = results.into_iter().map(|result| {
          let o_response = result.0;
          let o_error = result.1;

          if !o_error.is_none() || o_response.is_none() {
            return "ERROR"
          }

          let code = o_response.unwrap().status().as_u16();
          if code >= 200 || code < 300 {
            return "SUCCESS"
          }
          return "FAILURE"
        }).collect();

        // TODO: There has to be a better way...
        macro_rules! count {
            ($vec:expr, $on:expr) => {
               $vec.clone().into_iter().filter(|s| s == &$on).collect::<Vec<_>>().len()
            };
        }

        println!("        success: {:?}", count!(breakdown, "SUCCESS"));
        println!("        failure: {:?}", count!(breakdown, "FAILURE"));
        println!("         errors: {:?}", count!(breakdown, "ERROR"));

        let duration = Instant::now() - start_time;
        println!("Run took: {:?}", duration);
    // });
}
