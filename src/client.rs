use std::str::FromStr;
use reqwest::Client as RClient;
use reqwest::{Request, Response};
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
    iterations: usize,

    method: Method,
    endpoint: Url,
}

macro_rules! chk_eclient {
    ($e:expr, $v:expr) => {
        if $e.is_err() {
            return Err(ClientError::NewClientError(format!("{:} (for value: '{:}')", $e.unwrap_err().to_string(), $v)))
        }
    };
}

impl Client {
    // TODO: Refactor to take headers as Vec
    pub fn new(method: &str, endpoint: &str, headers: Vec<String>, itr: usize) -> Result<Client, ClientError> {
        let mut builder = RClient::builder();
        if !headers.is_empty() {
            let h = header_map_from_vec(headers);
            chk_eclient!(h, "headers: Vec<String>");

            builder = builder.default_headers(h.unwrap());
        }

        let e = Url::parse(endpoint);
        chk_eclient!(e, endpoint);

        let m = Method::from_str(method);
        chk_eclient!(m, method);

        let i = if itr == 0 { 1 } else { itr };

        let c = builder.build();
        chk_eclient!(c, "builder.build()");

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

    async fn exec(&self) -> Result<Response, ClientError> {
        let req = self.request();
        debug!(format!("{:?}", req));
        let res: Response = match self.client.execute(req).await {
            Ok(res) => res,
            Err(e)  => return Err(ClientError::ClientRequestError(e.to_string()))
        };
        debug!(format!("{:?}", res));

        Ok(res)
    }

    // Return a vector of tuples with response and optional error
    pub async fn run(&self) -> Vec<(Option<Response>, Option<ClientError>)> {
        let mut results: Vec<(Option<Response>, Option<ClientError>)> = vec![];
        for _ in 0..self.iterations {
            let result = self.exec().await;
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
        let c = Client::new("GET", "http://localhost/", vec![], 0);
        assert!(!c.is_err());

        let c1 = Client::new("", "http://localhost/", vec![], 0);
        assert!(c1.is_err(), "invalid method");

        let c2 = Client::new("GET", "bad_url", vec![], 0);
        assert!(c2.is_err(), "invalid url");
    }
}