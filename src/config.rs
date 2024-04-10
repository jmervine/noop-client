use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use clap::Parser;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::*;

#[derive(Debug)]
pub enum ConfigError {
    ToHeaderSplitError(String)
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Parser,Debug,Clone)]
pub struct Config {
    #[arg(long, short, default_value = "http://localhost:3000/")]
    pub url: String,

    #[arg(long,short, default_value = "GET")]
    pub method: String,

    #[arg(long, short = 'x', default_value = "")]
    pub headers: Vec<String>,

    #[arg(long, short, required = false, default_value = "")]
    pub input: String,

    #[arg(long, short, required = false, default_value = "")]
    pub ouptut: String,

    // TODO: Make '--verbose' without a value work.
    #[arg(long, short, default_value = "false")]
    pub verbose: Option<bool>,

    #[arg(long, short = 'n', default_value_t = 1)]
    pub iterations: u32,
}

impl Config {
    fn has_file(&self) -> bool {
        if self.input.is_empty() {
            return false
        }

        if fs::metadata(self.input.clone()).is_err() {
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

        let content = File::open(&self.input)?;
        let reader = BufReader::new(content);

        configs = reader.lines().map( |line| {
            let mut new = self.clone();

            let split = line.unwrap_or(String::new());
            let mut parts = split.split('|').map(|p| p.to_string());

            match parts.next() {
                Some(v) => if v != "" { new.method = v.to_string(); },
                _ => ()
            };

            match parts.next() {
                Some(v) => if v != "" { new.url = v.to_string(); },
                _ => ()
            };

            // headers
            new.headers = match parts.next() {
                Some(v) => v.split(',').map(|s|s.to_string()).collect(),
                None => new.headers
            };

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

impl HeaderStringVec for Vec<String> {
    fn to_headers(self) -> Result<HeaderMap, ConfigError> {
        let mut map = HeaderMap::new(); 

        if self.is_empty() {
            return Ok(map)
        }

        for header in &self {
            if !header.is_empty() {
                let (name, value) = header.clone().to_header()?;
                map.insert(name, value);
            }
        }

        Ok(map)
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