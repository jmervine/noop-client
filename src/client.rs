use crate::config;
use crate::errors::ClientError;
use ureq;

static SPLIT_HEADER_VALUE_CHAR: [char; 2] = [':', '='];

#[derive(Debug, Clone)]
pub struct Client {
    pub method: String,
    pub endpoint: String,
    headers: Vec<(String, String)>,
    debug: bool,
}

impl Client {
    pub fn new(config: config::Config) -> Result<Client, ClientError> {
        let mut headers = Vec::<(String, String)>::new();

        for header in config.headers() {
            let header = header.to_header()?;
            if header.0 != "" {
                headers.push(header);
            }
        }

        return Ok(Client {
            method: config.method.clone(),
            endpoint: config.endpoint(),
            headers: headers,
            debug: config.debug,
        });
    }

    // Only returning status code or error right now.
    pub fn execute(&self) -> Result<u16, ClientError> {
        let client = ureq::agent();

        let mut request = client.request(&self.method, &self.endpoint);

        for (key, val) in &self.headers {
            request = request.set(key, val);
        }

        if self.debug {
            println!("DEBUG:: {:?}", request);
        }

        match request.call() {
            Ok(response) => {
                if self.debug {
                    println!("DEBUG:: {:?}", response);
                }
                return Ok(response.status());
            }
            Err(err) => return Err(ClientError::HTTPError(err.to_string())),
        }
    }
}

pub trait HeaderStringSplit {
    fn to_header(self) -> Result<(String, String), ClientError>;
}

impl HeaderStringSplit for String {
    fn to_header(self) -> Result<(String, String), ClientError> {
        if self.is_empty() {
            return Ok((String::new(), String::new()));
        }

        let delim: char = delim_in(self.clone());
        match self.split_once(delim) {
            Some((name, value)) => {
                if name == "" {
                    return Err(ClientError::HeaderError(format!(
                        "Name cannot be empty in '{}'",
                        self
                    )));
                }
                return Ok((name.to_string(), value.to_string()));
            }
            None => {
                return Err(ClientError::HeaderError(format!(
                    "Header values cannot be empty in '{}'",
                    self
                )))
            }
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
fn client_new_test() {
    let mut cfg = config::Config {
        headers: vec!["Foo=bar".to_string(), "Bah:boo".to_string()],
        method: "GET".to_string(),
        endpoint: "http://localhost:3000".to_string(),
        script: String::new(),
        iterations: 1,
        sleep: 0,
        pool_size: 1,
        verbose: false,
        debug: false,
        errors: false,
        output: "default".to_string(),
        randomize: true,
    };

    let cli = Client::new(cfg.clone());
    assert!(!cli.is_err());

    let cli = cli.unwrap();
    assert_eq!(cli.headers[0], ("Foo".to_string(), "bar".to_string()));
    assert_eq!(cli.headers[1], ("Bah".to_string(), "boo".to_string()));

    cfg.headers = vec!["=ack".to_string()];
    let cli = Client::new(cfg);
    assert!(cli.is_err());
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
    assert!(none.is_ok());
}
