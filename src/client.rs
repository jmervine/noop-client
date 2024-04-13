use reqwest::Client as RClient;
use reqwest::{Method, Url};
use reqwest::{Request, Response};
use std::str::FromStr;

use crate::*;

#[derive(Debug, Clone)]
pub struct Client {
    client: RClient,
    iterations: usize,

    method: Method,
    endpoint: Url,
}

impl Client {
    // TODO: Refactor to take headers as Vec
    pub fn new(
        method: &str,
        endpoint: &str,
        headers: Vec<String>,
        itr: usize,
    ) -> Result<Client, utils::Errors> {
        let mut builder = RClient::builder();
        if !headers.is_empty() {
            let header = header_map_from_vec(headers);
            if header.is_empty() {
                builder = builder.default_headers(header);
            }
        }

        let e = Url::parse(endpoint);
        if e.is_err() {
            return error!(e);
        }

        let m = Method::from_str(method);
        if m.is_err() {
            return error!(m);
        }

        let i = if itr == 0 { 1 } else { itr };

        let c = builder.build();
        if c.is_err() {
            return error!(c);
        }

        Ok(Client {
            client: c.unwrap(),
            method: m.unwrap(),
            endpoint: e.unwrap(),
            iterations: i,
        })
    }

    fn request(&self) -> Request {
        Request::new(self.method.clone(), self.endpoint.clone())
    }

    async fn exec(&self) -> Result<Response, utils::Errors> {
        let req = self.request();
        debug!(format!("{:?}", req));
        let res = self.client.execute(req).await;
        if res.is_err() {
            return error!(res);
        }

        let r = res.unwrap();
        debug!(format!("{:?}", r));

        Ok(r)
    }

    // Return a vector of tuples with response and optional error
    pub async fn run(&self) -> Vec<Result<Response, utils::Errors>> {
        let mut results: Vec<Result<Response, utils::Errors>> = vec![];
        for _ in 0..self.iterations {
            let result = self.exec().await;
            results.push(result);
        }

        results
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn new_test() {
        {
            let c = Client::new("GET", "http://localhost/", vec![], 0);
            assert!(!c.is_err());
        }

        {
            let c = Client::new("", "http://localhost/", vec![], 0);
            assert!(c.is_err())
        }
        {
            let c = Client::new("GET", "bad_url", vec![], 0);
            assert!(c.is_err());
        }
    }
}
