use crate::errors::ClientError;
use std::fs;
use std::{thread, time};

#[cfg(any(feature = "json", feature = "yaml"))]
use std::ffi;

#[cfg(any(feature = "json", feature = "yaml"))]
use std::path;

use clap::Parser;
use rand::Rng;
use serde_derive::Deserialize;

/// This is a (hopefully) simple method of sending http requests (kind of like curl). Either directly; or via a pipe delimited text file
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Number of requests to make for each endpoint
    #[arg(long, short = 'n', default_value_t = 1)]
    pub iterations: usize,

    /// Target endpoint to make an http requests against
    #[arg(long = "endpoint", short = 'e', default_value = "")]
    pub endpoint: String,

    /// Method to be used when making an http requests
    #[arg(long, short, default_value = "GET")]
    pub method: String,

    /// Headers to be used when making an http requests
    #[arg(long, short = 'x', default_value = "")]
    pub headers: Vec<String>,

    /// Built in sleep duration (in milliseconds) to be used when making multiple requests
    #[arg(long = "sleep", short = 's', default_value = "0")]
    pub sleep: u64,

    /// File path containing a list of options to be used, in place of other arguments
    #[arg(long = "script", short = 'f', default_value = "")]
    pub script: String,

    /// Number of parallel requests
    #[arg(long = "pool-size", short = 'p', default_value = "100")]
    pub pool_size: usize,

    /// Output format; options: default, json, csv, (with features) yaml, json
    #[arg(long = "output", short = 'o', default_value = "default")]
    pub output: String,

    /// Randomize 'endpoint' or 'headers';
    /// TIMESTAMP is replaced with a timestamp,
    /// RANDOM is replaced with a random number
    #[arg(
        long = "random",
        short = 'r',
        default_value = "false",
        default_missing_value = "true"
    )]
    pub randomize: bool,

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

fn default_string() -> String {
    return String::new();
}

fn default_usize() -> usize {
    return 0;
}

fn default_u64() -> u64 {
    return 0;
}

#[derive(Debug, Deserialize, Default)]
struct ConfigDeserializer {
    #[serde(default = "default_usize")]
    pub iterations: usize,

    #[serde(default = "default_string")]
    pub method: String,

    #[serde(default = "default_string")]
    pub endpoint: String,

    #[serde(default = "default_string")]
    pub headers: String,

    #[serde(default = "default_u64")]
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

    fn valid_outputs(&self) -> Vec<&str> {
        #[allow(unused)]
        let mut outputs = vec!["default", "csv"];

        #[cfg(feature = "json")]
        outputs.push("json");

        #[cfg(feature = "yaml")]
        outputs.push("yaml");

        return outputs;
    }

    pub fn is_valid(&self) -> bool {
        let o = self.valid_outputs();

        return !(self.endpoint.is_empty() && self.script.is_empty())
            && o.contains(&self.output.as_str());
    }

    pub fn sleep(&self) {
        let sleep = std::time::Duration::from_millis(self.sleep);
        if sleep > time::Duration::ZERO {
            thread::sleep(sleep);
        }
    }

    pub fn headers(&self) -> Vec<String> {
        if !self.randomize {
            return self.headers.clone();
        }

        let mut headers: Vec<String> = Vec::with_capacity(self.headers.len());
        for header in self.headers.clone() {
            headers.push(Config::randomize_string(header));
        }

        return headers;
    }

    pub fn endpoint(&self) -> String {
        let endpoint = self.endpoint.clone();

        if !self.randomize {
            return endpoint;
        }

        return Config::randomize_string(endpoint);
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

    #[cfg(any(feature = "json", feature = "yaml"))]
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

    fn now() -> String {
        return time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string();
    }

    fn randomize_string(mut s: String) -> String {
        if s.contains("RANDOM") {
            let rnd: u32 = rand::thread_rng().gen();

            s = s.replace("RANDOM", &rnd.to_string());
        }

        if s.contains("TIMESTAMP") {
            let now = Config::now();
            s = s.replace("TIMESTAMP", &now);
        }

        return s;
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

    #[cfg(feature = "yaml")]
    fn from_yaml(&self) -> Result<Vec<Config>, ClientError> {
        let script_body = self.script_body()?;

        let mut configs: Vec<Config> = vec![];

        let expect = format!("Bad YAML from {}", self.script);
        let records: Vec<ConfigDeserializer> = serde_yaml::from_str(&script_body).expect(&expect);

        for record in records {
            configs.push(self.deserialize(record));
        }

        return Ok(configs);
    }

    #[cfg(feature = "json")]
    fn from_json(&self) -> Result<Vec<Config>, ClientError> {
        let script_body = self.script_body()?;

        let mut configs: Vec<Config> = vec![];

        let expect = format!("Bad JSON from {}", self.script);
        let records: Vec<ConfigDeserializer> = serde_json::from_str(&script_body).expect(&expect);

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

        #[cfg(feature = "yaml")]
        if self.script_ext()? == "yaml".to_string() {
            return self.from_yaml();
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
        randomize: false,
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
#[cfg(any(feature = "json", feature = "yaml"))]
fn script_ext_test() {
    let mut c = test_config();
    assert!(c.script_ext().is_err());

    c.script = "/foo.json".to_string();
    assert_eq!(c.script_ext().unwrap(), "json");
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
