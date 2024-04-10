use std::str::FromStr;
use reqwest::blocking::Client as RClient;
use reqwest::blocking::{Request, Response};
use reqwest::header::HeaderMap;
use reqwest::{Url,Method};

use crate::*;

#[derive(Debug)]
pub enum ClientError {
    NewClientError(String),
    ClientRequestError(String),
}

#[derive(Debug)]
pub struct Client {
    client: RClient,
    iterations: u32,

    method: Method,
    endpoint: Url,
}

impl Client {
    // TODO: Refactor to take headers as Vec
    pub fn new(method: &str, endpoint: &str, headers: HeaderMap, itr: u32) -> Result<Client, ClientError> {
        let mut builder = RClient::builder();
        if !headers.is_empty() {
            builder = builder.default_headers(headers);
        }

        let e = Url::parse(endpoint);
        if e.is_err() {
            return Err(ClientError::NewClientError(format!("{:} (for value: '{}')", e.unwrap_err(), endpoint)))
        }

        let m = Method::from_str(method);
        if m.is_err() {
            return Err(ClientError::NewClientError(format!("{:} (for value: '{}')", m.unwrap_err(), endpoint)))
        }

        let i = if itr == 0 { 1 } else { itr };

        let c = builder.build();
        if c.is_err() {
            return Err(ClientError::NewClientError(e.unwrap_err().to_string()))
        }

        Ok(Client {
            client: c.unwrap(),
            method: m.unwrap(),
            endpoint: e.unwrap(),
            iterations: i
        })
    }

    fn request(&self) -> Request {
        Request::new(self.method.clone(), self.endpoint.clone())
    }

    fn exec(&self) -> Result<Response, ClientError> {
        let req = self.request();
        debug!(format!("{:?}", req));
        let res: Response = match self.client.execute(req) {
            Ok(res) => res,
            Err(e)  => return Err(ClientError::ClientRequestError(e.to_string()))
        };
        debug!(format!("{:?}", res));

        Ok(res)
    }

    // Return a vector of tuples with response and optional error
    pub fn run(&self) -> Vec<(Option<Response>, Option<ClientError>)> {
        let mut results: Vec<(Option<Response>, Option<ClientError>)> = vec![];

        for _ in 0..self.iterations {
            let result = self.exec();
            match result {
                Ok(res) => results.push((Some(res), None)),
                Err(e)  => results.push((None, Some(e)))
            };
        }

        results
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn new_test() {
        let c = Client::new("GET", "http://localhost/", HeaderMap::new(), 0);
        assert!(!c.is_err());

        let c1 = Client::new("", "http://localhost/", HeaderMap::new(), 0);
        assert!(c1.is_err(), "invalid method");

        let c2 = Client::new("GET", "bad_url", HeaderMap::new(), 0);
        assert!(c2.is_err(), "invalid url");
    }
}