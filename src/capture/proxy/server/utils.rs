use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Method, Request, StatusCode, Version};
use hyper_util::client::legacy::Client;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

use crate::capture::proxy::client_pool::HttpClientPool;
use crate::error::{Result, WebMockError};

pub async fn forward_request(
    method: Method,
    url: &str,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    _version: Version,
) -> Result<(StatusCode, HashMap<String, String>, Vec<u8>)> {
    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();

    // Parse the target URL
    let uri: hyper::Uri = url
        .parse()
        .map_err(|e| WebMockError::Proxy(format!("Invalid URL: {}", e)))?;

    // Build the request
    let mut request_builder = Request::builder().method(method).uri(uri);

    // Add headers (excluding hop-by-hop headers)
    for (name, value) in headers {
        // Skip hop-by-hop headers that shouldn't be forwarded
        let name_lower = name.to_lowercase();
        if !is_hop_by_hop_header(&name_lower) {
            if let (Ok(header_name), Ok(header_value)) = (
                hyper::header::HeaderName::from_bytes(name.as_bytes()),
                hyper::header::HeaderValue::from_str(&value),
            ) {
                request_builder = request_builder.header(header_name, header_value);
            }
        }
    }

    let request = request_builder
        .body(Full::new(Bytes::from(body)))
        .map_err(|e| WebMockError::Proxy(format!("Failed to build request: {}", e)))?;

    // Send the request
    let response = client
        .request(request)
        .await
        .map_err(|e| WebMockError::Proxy(format!("Request failed: {}", e)))?;

    let status = response.status();

    // Extract response headers
    let mut response_headers = HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Read response body
    let response_body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| WebMockError::Proxy(format!("Failed to read response body: {}", e)))?
        .to_bytes()
        .to_vec();

    Ok((status, response_headers, response_body))
}

pub async fn forward_request_with_pool(
    method: Method,
    url: &str,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    _version: Version,
    client_pool: Arc<HttpClientPool>,
) -> Result<(StatusCode, HashMap<String, String>, Vec<u8>)> {
    // Parse the target URL to extract host
    let parsed_url =
        Url::parse(url).map_err(|e| WebMockError::Proxy(format!("Invalid URL: {}", e)))?;

    let host = parsed_url
        .host_str()
        .ok_or_else(|| WebMockError::Proxy("No host in URL".to_string()))?;

    // Get client from pool
    let client = client_pool.get_client(host).await;

    // Parse the target URL for hyper
    let uri: hyper::Uri = url
        .parse()
        .map_err(|e| WebMockError::Proxy(format!("Invalid URL: {}", e)))?;

    // Build the request
    let mut request_builder = Request::builder().method(method).uri(uri);

    // Add headers (excluding hop-by-hop headers)
    for (name, value) in headers {
        // Skip hop-by-hop headers that shouldn't be forwarded
        let name_lower = name.to_lowercase();
        if !is_hop_by_hop_header(&name_lower) {
            if let (Ok(header_name), Ok(header_value)) = (
                hyper::header::HeaderName::from_bytes(name.as_bytes()),
                hyper::header::HeaderValue::from_str(&value),
            ) {
                request_builder = request_builder.header(header_name, header_value);
            }
        }
    }

    let request = request_builder
        .body(Full::new(Bytes::from(body)))
        .map_err(|e| WebMockError::Proxy(format!("Failed to build request: {}", e)))?;

    // Send the request using pooled client
    let response = client
        .request(request)
        .await
        .map_err(|e| WebMockError::Proxy(format!("Request failed: {}", e)))?;

    let status = response.status();

    // Extract response headers
    let mut response_headers = HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Read response body
    let response_body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| WebMockError::Proxy(format!("Failed to read response body: {}", e)))?
        .to_bytes()
        .to_vec();

    Ok((status, response_headers, response_body))
}

pub fn is_hop_by_hop_header(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}
