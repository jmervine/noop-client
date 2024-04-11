use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use clap::Parser;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::*;

#[derive(Debug)]
pub enum ConfigError {
    InvalidConfigurationError(String),
    ToHeaderSplitError(String)
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Parser,Debug,Clone)]
pub struct Config {
    #[arg(long="endpoint", short='e')]
    pub o_endpoint: Option<String>, // REQUIRED
    // TODO: This shouldn't be required when 'input' is passed.

    #[arg(long,short, default_value = "GET")]
    pub method: String,

    #[arg(long, short = 'x', default_value = "")]
    pub headers: Vec<String>,

    #[arg(long="script", short='s')]
    pub o_script: Option<String>,

    // TODO: Implement
    // #[arg(long, short, required = false, default_value = "")]
    // pub output: String,

    // TODO: Make '--verbose' without a value work.
    #[arg(long="verbose", short='v')]
    pub o_verbose: Option<bool>,

    #[arg(long, short = 'n', default_value_t = 1)]
    pub iterations: usize,
}

impl Config {
    pub fn validate(&self) -> Option<ConfigError> {
        if self.endpoint().is_empty() && self.script().is_empty() {
            return Some(ConfigError::InvalidConfigurationError(
                "Either endpoint or script must be set, with script \
                a default endpoint is required for empty endpoint fields".to_string()
            ))
        }

        None
    }
    pub fn endpoint(&self) -> String {
        match &self.o_endpoint {
            Some(endpoint) => endpoint.clone(), _ => String::new()
        }
    }

    pub fn script(&self) -> String {
        match &self.o_script {
            Some(script) => script.clone(), _ => String::new()
        }
    }

    pub fn verbose(&self) -> bool {
        match self.o_verbose {
            Some(verbose) => verbose, _ => false
        }
    }

    fn has_file(&self) -> bool {
        if self.script().is_empty() {
            return false
        }

        if fs::metadata(self.script().clone()).is_err() {
            return false
        }

        return true
    }

    pub fn to_vec(&self) -> Result<Vec<Config>, Box<dyn Error>> {
        let mut configs: Vec<Config> = vec![];

        if !self.has_file() {
            configs.push(self.clone());
            return Ok(configs)
        }

        let content = File::open(&self.script())?;
        let reader = BufReader::new(content);

        // Count line iterations
        let mut i: usize = 0;

        configs = reader.lines().filter(|line| {
          match &line {
            Ok(l) => {
              // Empty lines and '#' based comments are ignored
              !(l.is_empty() || l.to_string().starts_with("#"))
            },
            Err(_) => false
          }
        }).map( |r_line| {
            // TODO: Make this block a function, or a few functions
            let line = r_line.unwrap_or(String::new());

            i += 1;

            let mut new = self.clone();

            // Find the number of '|' characters (+1) to ensure all fields are present.
            let n = line.chars().filter(|&c| c == '|').count() + 1;
            if n != 4 {
                bad_error!(format!(
                    "Found {} of 4 expected fields in '{}' for file:'{}', entry:'{}'",
                    n, line, &self.script(), i
                ));
            }

            let mut parts = line.split('|').map(|p| p.to_string());

            // Fetch for iterations, or use default from 'new'
            match parts.next() {
                Some(v) => match v.parse::<usize>() {
                  Ok(v) => { new.iterations = v },
                  Err(_) => ()
                },
                _ => ()
            };

            // Fetch for method, or use default from 'new'
            match parts.next() {
                Some(v) => if v != "" {
                  new.method = v.to_string();
                },
                _ => ()
            };

            // Fetch for endpoint, or use default from 'new'
            // TODO: Handle errors once when endpoint is no longer required.
            match parts.next() {
                Some(v) => if v != "" {
                  new.o_endpoint = Some(v.to_string());
                },
                _ => ()
            };

            if new.endpoint().is_empty(){
                bad_error!(format!(
                    "Empty endpoint without a default in '{}' for file:'{}', entry:'{}'",
                    line, &self.script(), i
                ));
            }

            // Fetch for headers, or use default from 'new'
            match parts.next() {
                Some(v) => if !v.is_empty() {
                  new.headers = v.split(',').map(|s|s.to_string()).collect()
                },
                _ => ()
            };

            // panic if not valid
            some_bad_error!(new.validate());

            new
        }).collect();

        Ok(configs)
    }
}

pub trait HeaderStringVec {
    fn to_headers(self) -> Result<HeaderMap, ConfigError>;
}

impl HeaderStringVec for Config {
    fn to_headers(self) -> Result<HeaderMap, ConfigError> {
        return self.headers.to_headers()
    }
}

pub fn header_map_from_vec(headers: Vec<String>) -> Result<HeaderMap, ConfigError> {
    let mut map = HeaderMap::new();

    if headers.is_empty() {
        return Ok(map)
    }

    for header in &headers {
        if !header.is_empty() {
            let (name, value) = header.clone().to_header()?;
            map.insert(name, value);
        }
    }

    Ok(map)
}

impl HeaderStringVec for Vec<String> {
    fn to_headers(self) -> Result<HeaderMap, ConfigError> {
        header_map_from_vec(self)
    }
}

fn splitter_from_list(s: String, list: Vec<char>) -> Result<char, ConfigError> {
    let mut matches: Vec<char> = vec![];
    for sp in list {
        if s.contains(sp) {
            matches.push(sp);
        }
    }

    if matches.is_empty() {
        return split_err!(format!("string '{}' missing required splitter", s))
    }

    if matches.len() != 1 {
        return split_err!(format!("string '{}' has many required splitters", s))
    }

    Ok(matches[0])
}

impl HeaderStringVec for String {
    fn to_headers(self) -> Result<HeaderMap, ConfigError> {
        let splitters = vec![','];
        let splitter = splitter_from_list(self.clone(), splitters)?;
        let sheaders: Vec<String> = self.split(splitter).map(|s| s.to_string()).collect();
        sheaders.to_headers()
    }
}

pub trait HeaderStringSplit {
    fn to_header(self) -> Result<(HeaderName, HeaderValue), ConfigError>;
}

impl HeaderStringSplit for String {
    fn to_header(self) -> Result<(HeaderName, HeaderValue), ConfigError> {
        let splitters = vec!['=', ':'];

        let splitter = splitter_from_list(self.clone(), splitters)?;
        let (sname, svalue) = match self.split_once(splitter) {
            Some((sname, svalue)) => (sname, svalue),
            _ => {
                return split_err!(format!("failed to split '{}' on '{}'", self, splitter))
            }
        };

        let hname = match HeaderName::try_from(sname) {
            Ok(hname) => hname,
            Err(e) => {
                return split_err!(e.to_string())
            }
        };

        let hvalue = match HeaderValue::try_from(svalue) {
            Ok(hvalue) => hvalue,
            Err(e) => {
                return split_err!(e.to_string())
            }
        };

        Ok((hname, hvalue))
    }
}

// ------
// TODO: Figure out how to move tests in to their own file.
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn vec_to_header_test() {
        // reused
        let hval1 = HeaderValue::from_str("application/json").unwrap();
        let hval2 = HeaderValue::from_str("testing").unwrap();

        // ok
        let hvec = vec![
            "Content-Type=application/json".to_string(), // standard
            "X-Foobar=testing".to_string()               // custom
        ];
        let mut expected = HeaderMap::new();
        expected.insert(reqwest::header::CONTENT_TYPE, hval1);
        expected.insert("X-Foobar", hval2);

        //let result  = headers(hvec).unwrap();
        let result  = hvec.to_headers().unwrap();
        assert_eq!(expected, result, "should create header map");

        let badvec1 = vec![
            "Content-Type;application/json".to_string() // bad split
        ];
        assert!(badvec1.to_headers().is_err(), "should error when not able to split on '='");

        let badvec2 = vec![
            "=application/json".to_string() // empty name
        ];
        assert!(badvec2.to_headers().is_err(), "shouldn't allow empty name");

        let badvec3 = vec![
            "content-type=".to_string() // empty value
        ];
        assert!(!badvec3.to_headers().is_err(), "should allow empty value");
    }
}