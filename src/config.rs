use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::{error, thread, time};

use clap::Parser;

static SPLIT_SCRIPT_CHAR: char = '|';
static SPLIT_HEADER_CHAR: char = ';';

/// This is a (hopefully) simple method of sending http requests (kind of like curl). Either directly; or via a pipe delimited text file
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// File path containing a list of options to be used, in place of other arguments
    #[arg(long = "script", short = 'f', default_value = "")]
    pub script: String,

    /// Target endpoint to make an http requests against
    #[arg(long = "endpoint", short = 'e', default_value = "")]
    pub endpoint: String,

    /// Method to be used when making an http requests
    #[arg(long, short, default_value = "GET")]
    pub method: String,

    /// Headers to be used when making an http requests
    #[arg(long, short = 'x', default_value = "")]
    pub headers: Vec<String>,

    /// Number of requests to make for each endpoint
    #[arg(long, short = 'n', default_value_t = 1)]
    pub iterations: usize,

    /// Built in sleep duration (in milliseconds) to be used when making multiple requests
    #[arg(long = "sleep", short = 's', default_value = "0")]
    pub sleep: u64,

    /// Number of parallel requests
    #[arg(long = "pool-size", short = 'p', default_value = "100")]
    pub pool_size: usize,

    /// Enable verbose output
    #[arg(
        long = "verbose",
        short = 'v',
        default_value = "false",
        default_missing_value = "true"
    )]
    pub verbose: bool,

    /// Enable debug output
    #[arg(
        long = "debug",
        short = 'D',
        default_value = "false",
        default_missing_value = "true"
    )]
    pub debug: bool,

    /// Enable error output for requests
    #[arg(
        long = "errors",
        short = 'E',
        default_value = "false",
        default_missing_value = "true"
    )]
    pub errors: bool,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let config = Config::parse();

        if config.is_valid() {
            return Ok(config);
        }

        return Err(format!("Configuration is invalid, see '--help' for details.").into());
    }

    pub fn is_valid(&self) -> bool {
        return !(self.endpoint.is_empty() && self.script.is_empty());
    }

    pub fn sleep(&self) {
        let sleep = std::time::Duration::from_millis(self.sleep);
        if sleep > time::Duration::ZERO {
            thread::sleep(sleep);
        }
    }

    fn has_file(&self) -> bool {
        if self.script.is_empty() {
            return false;
        }

        if fs::metadata(self.script.clone()).is_err() {
            return false;
        }

        return true;
    }

    pub fn to_vector(&self) -> Result<Vec<Config>, Box<dyn error::Error>> {
        let mut configs: Vec<Config> = vec![];

        if !self.has_file() {
            configs.push(self.clone());
            return Ok(configs);
        }

        let content = File::open(self.script.clone())?;
        let lines = BufReader::new(content).lines();

        for (idx, line) in lines.enumerate() {
            let line = line?;
            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let mut new = self.clone();

            // Find the number of '|' characters (+1) to to match the number of fields (to be clear)
            let pipe_count = line.chars().filter(|&c| c == '|').count() + 1;
            if pipe_count != 5 {
                return Err(format!(
                    "Found {} of 5 expected fields in '{}' for file:'{}', entry:'{}'",
                    pipe_count, line, self.script, idx
                )
                .into());
            }

            let mut parts = line.split(SPLIT_SCRIPT_CHAR).map(|p| p.to_string());

            // Fetch for iterations, or use default from 'new'
            if let Some(i) = parts.next() {
                let u: Result<usize, _> = i.parse();
                let itr = u.unwrap_or(0);
                if itr > 0 {
                    new.iterations = itr
                }
            }

            if let Some(m) = parts.next() {
                if !m.is_empty() {
                    new.method = m
                }
            }

            if let Some(e) = parts.next() {
                if !e.is_empty() {
                    new.endpoint = e;
                }
            }

            if new.endpoint.is_empty() {
                return Err(format!(
                    "Empty endpoint without a default in '{}' for file:'{}', entry:'{}'",
                    line, self.script, idx
                )
                .into());
            }

            // Fetch for headers, or use default from 'new'
            if let Some(h) = parts.next() {
                if !h.is_empty() {
                    new.headers = h.split(SPLIT_HEADER_CHAR).map(|s| s.to_string()).collect()
                }
            }

            if let Some(sleep) = parts.next() {
                if !sleep.is_empty() {
                    match sleep.parse::<u64>() {
                        Ok(sleep) => {
                            new.sleep = sleep;
                        }
                        Err(_) => {
                            return Err(format!(
                                "Couldn't convert '{:}' to duration for sleep in '{}' for file:'{}', entry:'{}'",
                                sleep, line, self.script, idx
                            ).into());
                        }
                    }
                }
            }

            // panic if not valid
            if !new.is_valid() {
                return Err("Invalid configurations, see help for details.".into());
            }

            configs.push(new);
        }

        Ok(configs)
    }
}

// ---
// For some reason this doesn't show as being used, even though it is.
#[allow(unused)]
fn config() -> Config {
    Config {
        endpoint: "http://www.example.com".to_string(),
        method: "GET".to_string(),
        headers: vec!["foo=bar".to_string()],
        script: "".to_string(),
        sleep: 0,
        verbose: false,
        debug: false,
        errors: false,
        iterations: 1,
        pool_size: 1,
    }
}

#[test]
fn is_valid_test() {
    let mut c = config();
    assert!(c.is_valid());

    c.endpoint = String::new();
    assert!(!c.is_valid());

    c.script = "file.txt".to_string();
    assert!(c.is_valid());
}

#[test]
fn endpoint_test() {
    let mut c = config();
    assert_eq!(c.endpoint, "http://www.example.com".to_string());

    c.endpoint = String::new();
    assert_eq!(c.endpoint, "".to_string());
}

#[test]
fn script_test() {
    let mut c = config();
    assert_eq!(c.script, "".to_string());

    c.script = "file.txt".to_string();
    assert_eq!(c.script, "file.txt".to_string());
}

#[test]
fn has_file_test() {
    let mut c = config();

    assert!(!c.has_file()); // with none

    c.script = "this_should_never_exist.ack".to_string();
    assert!(!c.has_file()); // with invalid file

    // Fragile - assume project root
    c.script = "test/test_script.txt".to_string();
    assert!(c.has_file()); // with valid file
}

#[test]
fn to_vector_test() {
    let c = config();

    // with no file
    let v = c.to_vector();
    assert!(!v.is_err());

    let v = v.unwrap().clone();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].method, "GET".to_string());
}

#[test]
fn verbose_test() {
    let mut c = config();
    assert!(!c.verbose);

    c.verbose = false;
    assert!(!c.verbose);

    c.verbose = true;
    assert!(c.verbose);
}
