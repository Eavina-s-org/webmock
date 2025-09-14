use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::debug;

use crate::capture::metrics::RequestTimer;
use crate::capture::proxy::client_pool::HttpClientPool;
use crate::capture::proxy::recorder::RequestRecorder;
use crate::capture::proxy::records::{RequestRecord, ResponseRecord};
use crate::capture::proxy::server::utils::forward_request_with_pool;

pub async fn handle_request(
    req: Request<Incoming>,
    recorder: Arc<RequestRecorder>,
    client_pool: Arc<HttpClientPool>,
    _remote_addr: SocketAddr,
) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    let _timer = RequestTimer::start();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let version = req.version();
    let headers = req.headers().clone();

    debug!("Handling request: {} {}", method, uri);

    // Handle CONNECT method for HTTPS tunneling
    if method == Method::CONNECT {
        return crate::capture::proxy::server::handlers::https_handlers::handle_connect_request(
            req, recorder,
        )
        .await;
    }

    // Extract request body
    let (_parts, body) = req.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes().to_vec(),
        Err(e) => {
            tracing::warn!("Failed to read request body for {} {}: {}", method, uri, e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Connection", "close")
                .body(Full::new(Bytes::from("Failed to read request body")))
                .unwrap());
        }
    };

    // Convert headers to HashMap
    let mut header_map = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(name.to_string(), value_str.to_string());
        }
    }

    // Determine the target URL
    let target_url = if uri.scheme().is_some() {
        uri.to_string()
    } else {
        // For HTTP/1.1 requests without scheme, construct from Host header
        let host = header_map
            .get("host")
            .or_else(|| header_map.get("Host"))
            .map(|s| s.as_str())
            .unwrap_or("localhost");

        // Determine protocol based on port or use HTTPS for MITM
        let is_https = host.contains(":443") || !host.contains(":80") && !host.contains(":8080");
        let protocol = if is_https { "https" } else { "http" };

        format!(
            "{}://{}{}",
            protocol,
            host,
            uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
        )
    };

    debug!("Forwarding request to: {}", target_url);

    // Forward the request to the target server using connection pool
    let response = forward_request_with_pool(
        method.clone(),
        &target_url,
        header_map.clone(),
        body_bytes.clone(),
        version,
        client_pool,
    )
    .await;

    match response {
        Ok((status, response_headers, response_body)) => {
            // Create response record
            let response_record = ResponseRecord::new(
                status.as_u16(),
                response_headers.clone(),
                response_body.clone(),
                Some(&target_url),
            );

            // Create request record
            let request_record = RequestRecord::new(
                method.to_string(),
                target_url,
                header_map,
                if body_bytes.is_empty() {
                    None
                } else {
                    Some(body_bytes)
                },
                response_record,
            );

            // Record the request
            recorder.record_request(request_record).await;

            // Build response
            let mut response_builder = Response::builder().status(status);

            // Add response headers
            for (name, value) in response_headers {
                if let (Ok(header_name), Ok(header_value)) = (
                    hyper::header::HeaderName::from_bytes(name.as_bytes()),
                    hyper::header::HeaderValue::from_str(&value),
                ) {
                    response_builder = response_builder.header(header_name, header_value);
                }
            }

            Ok(response_builder
                .body(Full::new(Bytes::from(response_body)))
                .unwrap())
        }
        Err(e) => {
            tracing::error!("Failed to forward request: {}", e);

            // Create error response record
            let error_response = ResponseRecord::new(
                502,
                HashMap::new(),
                format!("Proxy Error: {}", e).into_bytes(),
                Some(&target_url),
            );

            let request_record = RequestRecord::new(
                method.to_string(),
                target_url,
                header_map,
                if body_bytes.is_empty() {
                    None
                } else {
                    Some(body_bytes)
                },
                error_response,
            );

            recorder.record_request(request_record).await;

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Full::new(Bytes::from(format!("Proxy Error: {}", e))))
                .unwrap())
        }
    }
}
