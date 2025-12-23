use crate::error::Result;
use crate::http::{Method, Request};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration: Duration,
}

pub struct Client {
    inner: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            inner: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn execute(&self, request: &Request) -> Result<Response> {
        let start = Instant::now();

        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        };

        let mut req_builder = self.inner.request(method, &request.url);

        let mut headers = HeaderMap::new();
        for (key, value) in &request.headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::try_from(key.as_str()),
                HeaderValue::try_from(value.as_str()),
            ) {
                headers.insert(name, val);
            }
        }
        req_builder = req_builder.headers(headers);

        if let Some(ref body) = request.body {
            req_builder = req_builder.body(body.clone());
        }

        let response = req_builder.send().await?;
        let duration = start.elapsed();

        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.text().await.unwrap_or_default();

        Ok(Response {
            status,
            status_text,
            headers,
            body,
            duration,
        })
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
