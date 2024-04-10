extern crate clap;
extern crate reqwest;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{error::Error, str::FromStr};
use clap::Parser;
use reqwest::blocking::*;
use reqwest::header::*;
use reqwest::{Url, Method};

#[path = "config.rs"]
mod config;
use config::*;

static mut VERBOSE: bool = false;

// -- macros
macro_rules! is_verbose {
    () => { ( unsafe { VERBOSE } ) }
}

macro_rules! set_verbose {
    ($v:expr) => { 
        unsafe {
            match $v {
                Some(true) =>  { VERBOSE = true },
                _ => { VERBOSE = false },
                // Some(false) => { VERBOSE = false },
                // None => { VERBOSE = false }
            }
        }
    }
}

// macro_rules! todo {
//     ($s:expr) => {
//         println!("[TODO]: {} on line {}", $s, line!())
//     }
// }

macro_rules! debug {
    ($s:expr) => { 
        if is_verbose!() { 
            println!("[DEBUG]: {}", $s)
        }
    };
}

macro_rules! unless_debug {
    ($s:expr) => { 
        if !is_verbose!() { 
            print!("{}", $s)
        }
    };
}


fn request(method: &str, endpoint: &str, headers: HeaderMap) -> Result<String, Box<dyn Error>> {
    let client = Client::builder()
        .default_headers(headers)
        .build()?;

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

fn cli_request(cfg: Config) -> Result<String, Box<dyn Error>> {
    //let headers = headers(cfg.headers)?;
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

    if cfg.input.len() == 0 {
        for _ in 0..iters {
            let res = cli_request(cfg.clone())?;
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

                let vheaders: Vec<String> = if sheaders.len() == 0 || sheaders == "" {
                    cfg.headers.clone()
                } else {
                    sheaders.split(',')
                        .map(|s| s.to_string())
                        .collect()
                };

                debug!(format!("Headers: '{}'", vheaders.join(",")));

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
    let args: Config = Config::parse();
    set_verbose!(args.verbose);

    let res = do_requests(args)?;
    unless_debug!(format!("\nRequests made: {}", res.len()));
    unless_debug!("\n");

    Ok(())
}

// ------
#[cfg(test)]
mod tests {
    use reqwest::header::CONTENT_TYPE;

    use super::*;

    #[test]
    fn headers_ok_test() {
        // reused
        let hval1 = HeaderValue::from_str("application/json").unwrap();
        let hval2 = HeaderValue::from_str("testing").unwrap();

        // ok
        let hvec = vec![ 
            "Content-Type=application/json".to_string(), // standard
            "X-Foobar=testing".to_string()               // custom
        ];
        let mut expected = HeaderMap::new();
        expected.insert(CONTENT_TYPE, hval1);
        expected.insert("X-Foobar", hval2);

        //let result  = headers(hvec).unwrap();
        let result  = hvec.to_headers().unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn headers_bad_test() {
        let badvec = vec![ 
            "Content-Type;application/json".to_string() // bad split
        ];
        assert!(badvec.to_headers().is_err(), "should error when not able to split on '='"); 
    }

    #[test]
    fn headers_empty_name_test() {
        let badvec = vec![ 
            "=application/json".to_string() // empty name 
        ];
        assert!(badvec.to_headers().is_err(), "shouldn't allow empty name"); 
    }

    #[test]
    fn headers_empty_value_test() {
        let badvec = vec![ 
            "content-type=".to_string() // empty value 
        ];
        assert!(!badvec.to_headers().is_err(), "should allow empty value"); 
    }
}
