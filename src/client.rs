use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
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
    headers: Vec<String>,
}

impl Client {
    // TODO: Refactor to take headers as Vec
    pub fn new(
        method: &str,
        endpoint: &str,
        headers: Vec<String>,
        itr: usize,
    ) -> Result<Client, utils::Errors> {
        let builder = RClient::builder(); //.default_headers(map);

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
            headers: headers,
        })
    }

    async fn exec(&self) -> Result<Response, utils::Errors> {
        let mut req = Request::new(self.method.clone(), self.endpoint.clone());

        let headers = req.headers_mut();
        let mapped = self.headers.clone().header_map()?;

        for (name, val) in mapped {
            headers.insert(name.unwrap(), val);
        }

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

pub trait HeaderMapForClientRequest {
    fn header_map(&self) -> Result<HeaderMap, utils::Errors>;
}

impl HeaderMapForClientRequest for Vec<String> {
    fn header_map(&self) -> Result<HeaderMap, utils::Errors> {
        let mut map = HeaderMap::new();

        // Inject headers
        for header in self {
            let (name, value) = header.clone().to_header()?;

            // Forgoing error checking here, because I validate in 'to_header()' already.
            map.insert(
                HeaderName::from_str(&name).unwrap(),
                HeaderValue::from_str(&value).unwrap(),
            );
        }

        Ok(map)
    }
}

mod tests {
    // For some reason this doesn't show as being used, even though it is.
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn new_test() {
        let good = Client::new("GET", "http://localhost/", vec![], 0);
        let err1 = Client::new("", "http://localhost/", vec![], 0);
        let err2 = Client::new("GET", "bad_url", vec![], 0);

        assert!(good.is_ok());
        assert!(err1.is_err());
        assert!(err2.is_err());
    }

    #[test]
    fn header_map_test() {
        let good: Vec<String> = vec!["foo:bar".to_string()];
        let fine: Vec<String> = vec!["foo:".to_string()];
        let ugly: Vec<String> = vec![":bar".to_string()];
        let none: Vec<String> = vec!["".to_string()];

        assert!(good.header_map().is_ok());
        assert!(fine.header_map().is_ok());
        assert!(ugly.header_map().is_err());
        assert!(none.header_map().is_err());
    }
}
