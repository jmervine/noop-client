use crate::errors::ClientError;
use std::fs;
use std::{thread, time};

#[cfg(feature = "json")]
use std::ffi;

#[cfg(feature = "json")]
use std::path;

use clap::Parser;
use serde_derive::Deserialize;

#[cfg(feature = "json")]
static VALID_OUTPUTS: [&str; 2] = ["default", "csv"];

#[cfg(not(feature = "json"))]
static VALID_OUTPUTS: [&str; 3] = ["default", "json", "csv"];

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

    // --- < output options > ---
    #[cfg(feature = "json")]
    /// Output format; options: default, json, csv
    #[arg(long = "output", short = 'o', default_value = "default")]
    pub output: String,

    #[cfg(not(feature = "json"))]
    /// Output format; options: default, csv
    #[arg(long = "output", short = 'o', default_value = "default")]
    pub output: String,
    // --- < / output options > ---
    //
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

#[derive(Debug, Deserialize, Default)]
struct ConfigDeserializer {
    pub iterations: usize,
    pub method: String,
    pub endpoint: String,
    pub headers: String,
    pub sleep: u64,
}

impl Config {
    pub fn new() -> Result<Self, ClientError> {
        let config = Config::parse();

        if config.is_valid() {
            return Ok(config);
        }

        return Err(ClientError::ConfigError(format!(
            "Configuration is invalid, see '--help' for details."
        )));
    }

    pub fn is_valid(&self) -> bool {
        return !(self.endpoint.is_empty() && self.script.is_empty())
            && VALID_OUTPUTS.contains(&self.output.as_str());
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

    #[cfg(feature = "json")]
    fn script_ext(&self) -> Result<String, ClientError> {
        let extension = path::Path::new(&self.script)
            .extension()
            .and_then(ffi::OsStr::to_str);
        match extension {
            Some(extension) => return Ok(extension.to_string()),
            None => {
                return Err(ClientError::ConfigError(
                    "invalid script extension".to_string(),
                ))
            }
        }
    }

    fn script_body(&self) -> Result<String, ClientError> {
        let script = self.script.clone();
        let content = fs::read_to_string(script);
        match content {
            Ok(content) => return Ok(content),
            Err(_) => {
                return Err(ClientError::ConfigError(
                    "invalid script file path".to_string(),
                ))
            }
        }
    }

    fn deserialize(&self, record: ConfigDeserializer) -> Config {
        let mut config: Config = self.clone();

        if record.iterations != 0 {
            config.iterations = record.iterations;
        }

        if !record.method.is_empty() {
            config.method = record.method;
        }

        if !record.endpoint.is_empty() {
            config.endpoint = record.endpoint;
        }

        if !record.headers.is_empty() {
            config.endpoint = record.headers;
        }

        if record.sleep != 0 {
            config.sleep = record.sleep;
        }

        return config;
    }

    fn from_csv(&self) -> Result<Vec<Config>, ClientError> {
        let script_body = self.script_body()?;

        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .from_reader(script_body.as_bytes());

        let mut configs: Vec<Config> = vec![];
        for record in reader.deserialize() {
            if record.is_err() {
                return Err(ClientError::ConfigError(record.unwrap_err().to_string()));
            }

            let config: Config = self.deserialize(record.unwrap());

            configs.push(config);
        }

        return Ok(configs);
    }

    #[cfg(feature = "json")]
    fn from_json(&self) -> Result<Vec<Config>, ClientError> {
        let script_body = self.script_body()?;

        let mut configs: Vec<Config> = vec![];
        let records: Vec<ConfigDeserializer> =
            serde_json::from_str(&script_body).expect("Bad JSON");

        for record in records {
            configs.push(self.deserialize(record));
        }

        return Ok(configs);
    }

    pub fn to_vector(&self) -> Result<Vec<Config>, ClientError> {
        let mut configs: Vec<Config> = vec![];

        if !self.has_file() {
            configs.push(self.clone());
            return Ok(configs);
        }

        #[cfg(feature = "json")]
        if self.script_ext()? == "json".to_string() {
            return self.from_json();
        }

        return self.from_csv();
    }
}

// ---
// For some reason this doesn't show as being used, even though it is.
#[allow(unused)]
fn test_config() -> Config {
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
        output: "default".to_string(),
    }
}

#[test]
fn is_valid_test() {
    let mut c = test_config();
    assert!(c.is_valid());

    c.endpoint = String::new();
    assert!(!c.is_valid());

    c.script = "file.txt".to_string();
    assert!(c.is_valid());
}

#[test]
fn endpoint_test() {
    let mut c = test_config();
    assert_eq!(c.endpoint, "http://www.example.com".to_string());

    c.endpoint = String::new();
    assert_eq!(c.endpoint, "".to_string());
}

#[test]
fn script_test() {
    let mut c = test_config();
    assert_eq!(c.script, "".to_string());

    c.script = "file.txt".to_string();
    assert_eq!(c.script, "file.txt".to_string());
}

#[test]
fn has_file_test() {
    let mut c = test_config();

    assert!(!c.has_file()); // with none

    c.script = "this_should_never_exist.ack".to_string();
    assert!(!c.has_file()); // with invalid file

    // Fragile - assume project root
    c.script = "test/test_script.txt".to_string();
    assert!(c.has_file()); // with valid file
}

#[test]
fn to_vector_test() {
    let c = test_config();

    // with no file
    let v = c.to_vector();
    assert!(!v.is_err());

    let v = v.unwrap().clone();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].method, "GET".to_string());
}

#[test]
fn verbose_test() {
    let mut c = test_config();
    assert!(!c.verbose);

    c.verbose = false;
    assert!(!c.verbose);

    c.verbose = true;
    assert!(c.verbose);
}

#[test]
#[cfg(feature = "json")]
fn script_ext_test() {
    let mut c = test_config();
    assert!(c.script_ext().is_err());

    c.script = "/foo.csv".to_string();
    assert_eq!(c.script_ext().unwrap(), "csv");
}

#[test]
fn deserialize_test() {
    let cfg = test_config();
    let r = ConfigDeserializer {
        iterations: 5,
        method: "GET".to_string(),
        endpoint: "https://www.example.com/".to_string(),
        headers: String::new(),
        sleep: 0,
    };

    let c = cfg.deserialize(r);

    assert_eq!(c.iterations, 5);
    assert_eq!(c.method, "GET".to_string());
    assert_eq!(c.endpoint, "https://www.example.com/".to_string());
}
