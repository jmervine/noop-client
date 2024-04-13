use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use clap::Parser;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::*;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long = "endpoint", short = 'e')]
    pub o_endpoint: Option<String>, // REQUIRED
    // TODO: This shouldn't be required when 'input' is passed.
    #[arg(long, short, default_value = "GET")]
    pub method: String,

    #[arg(long, short = 'x', default_value = "")]
    pub headers: Vec<String>,

    #[arg(long = "script", short = 's')]
    pub o_script: Option<String>,

    // TODO: Implement
    // #[arg(long, short, required = false, default_value = "")]
    // pub output: String,

    // TODO: Make '--verbose' without a value work.
    #[arg(long = "verbose", short = 'v')]
    pub o_verbose: Option<bool>,

    #[arg(long, short = 'n', default_value_t = 1)]
    pub iterations: usize,
}

impl Config {
    pub fn is_valid(&self) -> bool {
        return !(self.endpoint().is_empty() && self.script().is_empty());
    }

    pub fn endpoint(&self) -> String {
        match &self.o_endpoint {
            Some(endpoint) => endpoint.clone(),
            _ => String::new(),
        }
    }

    pub fn script(&self) -> String {
        match &self.o_script {
            Some(script) => script.clone(),
            _ => String::new(),
        }
    }

    pub fn verbose(&self) -> bool {
        match self.o_verbose {
            Some(verbose) => verbose,
            _ => false,
        }
    }

    fn has_file(&self) -> bool {
        if self.script().is_empty() {
            return false;
        }

        if fs::metadata(self.script().clone()).is_err() {
            return false;
        }

        return true;
    }

    pub fn to_vector(&self) -> Result<Vec<Config>, utils::Errors> {
        let mut configs: Vec<Config> = vec![];

        if !self.has_file() {
            configs.push(self.clone());
            return Ok(configs);
        }

        let content = File::open(&self.script());
        if content.is_err() {
            return error!(content);
        }

        let lines = BufReader::new(content.unwrap()).lines();

        for (idx, l) in lines.enumerate() {
            if l.is_err() {
                return error!(l);
            }

            let line = l.unwrap();

            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let mut new = self.clone();

            // Find the number of '|' characters (+1) to ensure all fields are present.
            let n = line.chars().filter(|&c| c == '|').count() + 1;
            if n != 4 {
                // TODO: Consider skipping and warning, over erroring.
                return error_str!(format!(
                    "Found {} of 4 expected fields in '{}' for file:'{}', entry:'{}'",
                    n,
                    line,
                    &self.script(),
                    idx
                ));
            }

            let mut parts = line.split('|').map(|p| p.to_string());

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
                    new.o_endpoint = Some(e);
                }
            }

            if new.endpoint().is_empty() {
                return error_str!(format!(
                    "Empty endpoint without a default in '{}' for file:'{}', entry:'{}'",
                    line,
                    &self.script(),
                    idx
                ));
            }

            // Fetch for headers, or use default from 'new'
            if let Some(h) = parts.next() {
                if !h.is_empty() {
                    new.headers = h.split(',').map(|s| s.to_string()).collect()
                }
            }

            // panic if not valid
            if !new.is_valid() {
                return error_str!("Invalid configurations, see help for details.");
            }

            configs.push(new);
        }

        Ok(configs)
    }
}

pub trait HeaderStringVec {
    fn to_headers(self) -> HeaderMap;
}

impl HeaderStringVec for Config {
    fn to_headers(self) -> HeaderMap {
        return self.headers.to_headers();
    }
}

pub fn header_map_from_vec(headers: Vec<String>) -> HeaderMap {
    let mut map = HeaderMap::new();

    if headers.is_empty() {
        return map;
    }

    for header in &headers {
        if !header.is_empty() {
            if let Some((name, value)) = header.clone().to_header() {
                // TODO: Maybe warn or error if it's a bad header, instead of just skipping it.
                map.insert(name, value);
            }
        }
    }

    return map;
}

impl HeaderStringVec for Vec<String> {
    fn to_headers(self) -> HeaderMap {
        header_map_from_vec(self)
    }
}

impl HeaderStringVec for String {
    fn to_headers(self) -> HeaderMap {
        let sheaders: Vec<String> = self
            .split(',')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        sheaders.to_headers()
    }
}

pub trait HeaderStringSplit {
    fn to_header(self) -> Option<(HeaderName, HeaderValue)>;
}

impl HeaderStringSplit for String {
    fn to_header(self) -> Option<(HeaderName, HeaderValue)> {
        if let Some((sname, svalue)) = self.split_once(':') {
            if let Ok(name) = HeaderName::try_from(sname) {
                if let Ok(value) = HeaderValue::try_from(svalue) {
                    return Some((name, value));
                }
            }
        }
        return None;
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
            "Content-Type:application/json".to_string(), // standard
            "X-Foobar:testing".to_string(),              // custom
        ];
        let mut expected = HeaderMap::new();
        expected.insert(reqwest::header::CONTENT_TYPE, hval1);
        expected.insert("X-Foobar", hval2);

        let result = hvec.to_headers();
        assert_eq!(expected, result, "should create header map");

        let badvec1 = vec![
            "Content-Type;application/json".to_string(), // bad split
        ];
        assert!(badvec1.to_headers().is_empty());

        let badvec2 = vec![
            ":application/json".to_string(), // empty name
        ];
        assert!(
            badvec2.to_headers().is_empty(),
            "shouldn't allow empty name"
        );

        let badvec3 = vec![
            "content-type:".to_string(), // empty value
        ];
        assert!(!badvec3.to_headers().is_empty(), "should allow empty value");
    }
}
