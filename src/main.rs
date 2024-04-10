extern crate clap;
extern crate reqwest;

use std::time::Instant;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{error::Error, str::FromStr};
use clap::Parser;
use reqwest::blocking::*;
use reqwest::header::*;
use reqwest::{Url, Method};

#[path = "config.rs"]
#[macro_use]
mod config;
use config::*;

fn request(method: &str, endpoint: &str, headers: HeaderMap) -> Result<String, Box<dyn Error>> {
    let mut builder = Client::builder();

    if !headers.is_empty() {
        builder = builder.default_headers(headers)
    }

    let client = builder.build()?;

    let url = Url::parse(endpoint)?;
    let meth = Method::from_str(method)?;
    let req = Request::new(meth, url);

    let resp = client.execute(req)?; 
    let body = resp.text()?;

    debug!("---");
    debug!(body);
    unless_debug!("+");
    Ok(body)
}

// TODO: The names of these functions are terrible...
fn make_request(cfg: Config) -> Result<String, Box<dyn Error>> {
    let headers = match cfg.clone().to_headers() {
        Ok(headers) => headers,
        Err(e) => return Err(e.to_string().into()),
    };

    let res = request(
        &cfg.method, &cfg.url, headers
    );
 
    let body = match res {
        Ok(s) => s,
        Err(e) => format!("ERROR: {}", e.to_string()), 
    };

    Ok(body)
}

fn do_requests(cfg: Config) -> Result<Vec<String>, Box<dyn Error>> {
    debug!(format!("{:#?}", cfg));

    let iters: i32 = if cfg.iterations == 0 { 1 } else { cfg.iterations };

    let mut output: Vec<String> = Vec::new();

    if cfg.input.is_empty() {
        if iters == 1 {
            let res = make_request(cfg.clone())?;
            output.push(res);
            return Ok(output)
        }

        for _ in 0..iters {
            let res = make_request(cfg.clone())?;
            output.push(res);
        }

        return Ok(output)
    }

    let input = File::open(&cfg.input)?;
    let reader = BufReader::new(input);

    for line in reader.lines() {
        match line {
            Ok(l) => {
                debug!(format!("Extracting: '{}'", l));

                let mut parts = l.split('|');
                let (mut method, mut url, sheaders) = (
                    parts.next().unwrap_or(""),
                    parts.next().unwrap_or(""),
                    parts.next().unwrap_or("")
                );

                if method.len() == 0 || method == "" {
                    method = &cfg.method;
                }

                if url.len() == 0 || url == "" {
                    url = &cfg.url;
                }

                //let vheaders: Vec<String> = if sheaders.len() == 0 || sheaders == "" {
                let vheaders: Vec<String> = if sheaders.is_empty() {
                    cfg.headers.clone()
                } else {
                    sheaders.split(',')
                        .map(|s| s.to_string())
                        .collect()
                };

                debug!(format!("Headers: '{:#?}'", vheaders));

                let headers = vheaders.to_headers()?;
                for _ in 0..iters {
                    let body = request(method, url, headers.clone())?;
                    output.push(body);
                }
            },
            Err(err) => {
                eprintln!("Error reading line: {}", err);
            }
        }
    }

    Ok(output)
}

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    let args: Config = Config::parse();
    set_verbose!(args.verbose);

    let res = do_requests(args)?;
    unless_debug!(format!("\nRequests made: {}", res.len()));
    unless_debug!("\n");

    let duration = Instant::now() - start_time;
    println!("Run took: {:?}", duration);

    Ok(())
}
