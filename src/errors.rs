use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    ConfigError(String),
    HTTPError(String),
    HeaderError(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::ConfigError(s) => return write!(f, "{}", s),
            ClientError::HTTPError(s) => return write!(f, "{}", s),
            ClientError::HeaderError(s) => return write!(f, "{}", s),
        };
    }
}
