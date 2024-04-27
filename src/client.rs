use reqwest;
use reqwest::header;
use std::error;

use crate::config;

static SPLIT_HEADER_VALUE_CHAR: [char; 2] = [':', '='];

#[derive(Debug, Clone)]
pub struct Client {
    method: reqwest::Method,
    endpoint: reqwest::Url,
    headers: header::HeaderMap,
    debug: bool,
}

impl Client {
    pub fn new(config: config::Config) -> Result<Client, Box<dyn error::Error>> {
        let method = reqwest::Method::from_bytes(config.method.as_bytes())?;
        let endpoint = reqwest::Url::parse(&config.endpoint)?;
        let headers = header_map(config.headers)?;

        return Ok(Client {
            method: method,
            endpoint: endpoint,
            headers: headers,
            debug: config.debug,
        });
    }

    // Only returning status code or error right now.
    pub fn execute(&self) -> Result<u16, Box<dyn error::Error>> {
        let client = reqwest::blocking::Client::new();
        let mut request = client.request(self.method.clone(), self.endpoint.clone());

        for (key, value) in self.headers.clone() {
            if let Some(key) = key {
                request = request.header(key, value);
            }
        }

        let request = request.build()?;

        if self.debug {
            println!("DEBUG:: {:?}", request);
        }

        let response = client.execute(request)?;

        if self.debug {
            println!("DEBUG:: {:?}", response);
        }

        return Ok(response.status().as_u16());
    }
}

fn header_map(headers: Vec<String>) -> Result<header::HeaderMap, Box<dyn error::Error>> {
    let mut map = header::HeaderMap::new();
    if headers.is_empty() {
        return Ok(map);
    }

    for header in headers {
        if header != "" {
            let (key, val) = header.to_header()?;
            if !key.is_empty() && !val.is_empty() {
                let key = header::HeaderName::from_bytes(key.as_bytes())?;
                let val = header::HeaderValue::from_bytes(val.as_bytes())?;

                map.append(key, val);
            }
        }
    }

    return Ok(map);
}

pub trait HeaderStringSplit {
    fn to_header(self) -> Result<(String, String), Box<dyn std::error::Error>>;
}

impl HeaderStringSplit for String {
    fn to_header(self) -> Result<(String, String), Box<dyn std::error::Error>> {
        let delim: char = delim_in(self.clone());
        match self.split_once(delim) {
            Some((name, value)) => {
                if name == "" {
                    return Err(format!("Name cannot be empty in '{}'", self).into());
                }
                return Ok((name.to_string(), value.to_string()));
            }
            None => return Err(format!("Header values cannot be empty in '{}'", self).into()),
        }
    }
}

fn delim_in(string: String) -> char {
    let current_char: char = SPLIT_HEADER_VALUE_CHAR[0];
    let string_chars: Vec<char> = string.chars().collect();
    for i_delim in 0..SPLIT_HEADER_VALUE_CHAR.len() {
        for i_char in 0..string_chars.len() {
            if SPLIT_HEADER_VALUE_CHAR[i_delim] == string_chars[i_char] {
                return SPLIT_HEADER_VALUE_CHAR[i_delim];
            }
        }
    }
    return current_char;
}

#[test]
fn to_header_from_string_test() {
    let good1 = "foo:bar".to_string().to_header();
    let good2 = "foo=bar".to_string().to_header();
    let fine = "foo:".to_string().to_header();
    let ugly = ":bar".to_string().to_header();
    let none = "".to_string().to_header();

    assert!(good1.is_ok());
    assert_eq!(good1.unwrap(), ("foo".to_string(), "bar".to_string()));
    assert!(good2.is_ok());
    assert_eq!(good2.unwrap(), ("foo".to_string(), "bar".to_string()));
    assert!(fine.is_ok());
    assert_eq!(fine.unwrap(), ("foo".to_string(), "".to_string()));
    assert!(ugly.is_err());
    assert!(none.is_err());
}
