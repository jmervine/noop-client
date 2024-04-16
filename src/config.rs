use std::fs::{self, File};
use std::io::{BufRead, BufReader};

use clap::Parser;

use crate::*;

static SPLIT_SCRIPT_CHAR: char = '|';
static SPLIT_HEADER_CHAR: char = ';';

// TODO: Split header kv string on '=' in addition to ':'
static SPLIT_HEADER_VALUE_CHAR: char = ':';

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long = "endpoint", short = 'e')]
    pub o_endpoint: Option<String>, // REQUIRED
    // TODO: This shouldn't be required when 'input' is passed.
    #[arg(long, short, default_value = "GET")]
    pub method: String,

    #[arg(long, short = 'x', default_value = "")]
    pub headers: Vec<String>,

    #[arg(long = "script", short = 'f')]
    pub o_script: Option<String>,

    #[arg(long = "sleep", short = 's')]
    pub o_sleep: Option<u64>,

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

    pub fn sleep(&self) -> std::time::Duration {
        let s = match &self.o_sleep {
            Some(s) => s.clone(),
            _ => return std::time::Duration::ZERO,
        };

        let t = std::time::Duration::from_millis(s);
        return t;
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

            // Find the number of '|' characters (+1) to to match the number of fields (to be clear)
            let n = line.chars().filter(|&c| c == '|').count() + 1;
            if n != 5 {
                // TODO: Consider skipping and warning, over erroring.
                return error_str!(format!(
                    "Found {} of 5 expected fields in '{}' for file:'{}', entry:'{}'",
                    n,
                    line,
                    &self.script(),
                    idx
                ));
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
                    new.headers = h.split(SPLIT_HEADER_CHAR).map(|s| s.to_string()).collect()
                }
            }

            if let Some(s) = parts.next() {
                if !s.is_empty() {
                    let sm = s.parse::<u64>();
                    if sm.is_err() {
                        return error_str!(format!(
                            "Couldn't convert '{:}' to duration for sleep in '{}' for file:'{}', entry:'{}'",
                            s,
                            line,
                            &self.script(),
                            idx
                        ));
                    }

                    new.o_sleep = Some(sm.unwrap());
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

// ---
// I'm not totally sure this belongs in this package, but there is other splitting here
// TODO: Consider storing headers as `Vec<(String, String>)>`
pub trait HeaderStringSplit {
    fn to_header(self) -> Result<(String, String), utils::Errors>;
}

impl HeaderStringSplit for String {
    fn to_header(self) -> Result<(String, String), utils::Errors> {
        match self.split_once(SPLIT_HEADER_VALUE_CHAR) {
            Some((name, value)) => {
                if name == "" {
                    return error_str!(format!("Name cannot be empty in '{}'", self));
                }
                return Ok((name.to_string(), value.to_string()));
            }
            None => return Err(utils::Errors::Ignorable),
        }
    }
}

mod test {
    // For some reason this doesn't show as being used, even though it is.
    #[allow(unused_imports)]
    use super::*;

    // For some reason this doesn't show as being used, even though it is.
    #[allow(unused)]
    fn config() -> Config {
        Config {
            o_endpoint: Some("http://www.example.com".to_string()),
            method: "GET".to_string(),
            headers: vec!["foo=bar".to_string()],
            o_script: None,
            o_sleep: None,
            o_verbose: None,
            iterations: 1,
        }
    }

    #[test]
    fn is_valid_test() {
        let mut c = config();
        assert!(c.is_valid());

        c.o_endpoint = None;
        assert!(!c.is_valid());

        c.o_script = Some("file.txt".to_string());
        assert!(c.is_valid());
    }

    #[test]
    fn endpoint_test() {
        let mut c = config();
        assert_eq!(c.endpoint(), "http://www.example.com".to_string());

        c.o_endpoint = None;
        assert_eq!(c.endpoint(), "".to_string());
    }

    #[test]
    fn script_test() {
        let mut c = config();
        assert_eq!(c.script(), "".to_string());

        c.o_script = Some("file.txt".to_string());
        assert_eq!(c.script(), "file.txt".to_string());
    }

    #[test]
    fn has_file_test() {
        let mut c = config();

        assert!(!c.has_file()); // with none

        c.o_script = Some("this_should_never_exist.ack".to_string());
        assert!(!c.has_file()); // with invalid file

        // Fragile - assume project root
        c.o_script = Some("test/test_script.txt".to_string());
        assert!(c.has_file()); // with valid file
    }

    #[test]
    fn to_vector_test() {
        let c = config();

        // with no file
        let v = c.to_vector();
        assert!(!v.is_err());
        assert_eq!(v.clone().unwrap().len(), 1);
        assert_eq!(v.clone().unwrap()[0].method, "GET".to_string());

        // TODO: Test Config#to_vector() with 'test/test_script.txt'
    }

    #[test]
    fn verbose_test() {
        let mut c = config();
        assert!(!c.verbose());

        c.o_verbose = Some(false);
        assert!(!c.verbose());

        c.o_verbose = Some(true);
        assert!(c.verbose());
    }

    #[test]
    fn to_header_from_string_test() {
        let good = "foo:bar".to_string().to_header();
        let fine = "foo:".to_string().to_header();
        let ugly = ":bar".to_string().to_header();
        let none = "".to_string().to_header();

        assert!(good.is_ok());
        assert!(fine.is_ok());
        assert!(ugly.is_err());
        assert!(none.is_err());
    }
}
