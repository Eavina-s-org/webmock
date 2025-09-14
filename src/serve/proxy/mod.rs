use std::convert::Infallible;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::upgrade::Upgraded;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};

use crate::serve::handlers::{
    create_404_response, create_response_from_record, find_matching_record,
};
use crate::serve::tls::TlsConfig;
use crate::storage::Snapshot;

/// HTTP/HTTPS proxy request handler
pub struct ProxyHandler;

impl ProxyHandler {
    /// Handle regular HTTP requests through proxy
    pub async fn handle_http_request(
        snapshot: Arc<Snapshot>,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let method = req.method().clone();
        let uri = req.uri().clone();

        // For regular HTTP requests through proxy
        let full_url = {
            let uri_str = uri.to_string();
            if uri_str.starts_with("http") {
                uri_str
            } else {
                format!("http://unknown{}", uri_str)
            }
        };

        debug!("Handling HTTP request: {} {}", method, full_url);

        // Find matching request record
        match find_matching_record(&snapshot, &method, &full_url) {
            Some(record) => {
                let status_icon = if record.response.status >= 200 && record.response.status < 300 {
                    "✅"
                } else if record.response.status >= 400 {
                    "❌"
                } else {
                    "ℹ️"
                };

                println!(
                    "{} {} {} → {} ({})",
                    status_icon,
                    method,
                    full_url,
                    record.response.status,
                    record
                        .response
                        .headers
                        .get("content-type")
                        .unwrap_or(&"unknown".to_string())
                );

                Ok(create_response_from_record(record))
            }
            None => {
                println!("❌ {} {} → 404 (not found in snapshot)", method, full_url);
                warn!("Request not in snapshot: {} {}", method, full_url);

                Ok(create_404_response(&full_url))
            }
        }
    }

    /// Handle CONNECT requests for HTTPS tunneling
    pub async fn handle_connect_request(
        snapshot: Arc<Snapshot>,
        req: Request<Incoming>,
        host_port: String,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let method = req.method().clone();

        debug!("Handling CONNECT request for: {}", host_port);

        // Check if we have this CONNECT request in our snapshot
        let connect_url = format!("https://{}", host_port);

        match find_matching_record(&snapshot, &method, &connect_url) {
            Some(record) => {
                println!(
                    "✅ CONNECT {} → {} (tunnel established)",
                    host_port, record.response.status
                );
                info!(
                    "Found CONNECT record, establishing tunnel for: {}",
                    connect_url
                );

                // Return success response and handle tunnel in background
                Self::spawn_tunnel_handler(req, snapshot, host_port).await
            }
            None => {
                println!("❌ CONNECT {} → 502 (not found in snapshot)", host_port);
                warn!("CONNECT request not in snapshot: {}", connect_url);

                Ok(Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .header("Connection", "close")
                    .body(Full::new(Bytes::from(format!(
                        "CONNECT request not found in snapshot: {}",
                        host_port
                    ))))
                    .unwrap())
            }
        }
    }

    /// Spawn tunnel handler for HTTPS requests
    async fn spawn_tunnel_handler(
        req: Request<Incoming>,
        snapshot: Arc<Snapshot>,
        host_port: String,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        // Spawn a task to handle the tunnel
        let snapshot_clone = Arc::clone(&snapshot);
        let host_port_clone = host_port.clone();

        tokio::spawn(async move {
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    info!("Tunnel upgraded successfully for: {}", host_port_clone);
                    if let Err(e) =
                        Self::handle_https_tunnel(upgraded, snapshot_clone, host_port_clone).await
                    {
                        error!("Tunnel error: {}", e);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to upgrade CONNECT tunnel for {}: {}",
                        host_port_clone, e
                    );
                }
            }
        });

        // Return successful CONNECT response
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Connection", "upgrade")
            .header("Proxy-Agent", "WebMock-CLI/1.0")
            .body(Full::new(Bytes::new()))
            .unwrap())
    }

    /// Handle HTTPS tunnel with TLS termination
    async fn handle_https_tunnel(
        upgraded: Upgraded,
        snapshot: Arc<Snapshot>,
        host_port: String,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting HTTPS tunnel with TLS termination for: {}",
            host_port
        );

        // Generate TLS configuration
        let tls_config = match TlsConfig::generate_tls_config() {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to generate TLS config: {}", e);
                return Err(e);
            }
        };

        let acceptor = TlsAcceptor::from(tls_config);
        let io = TokioIo::new(upgraded);

        // Perform TLS handshake
        match acceptor.accept(io).await {
            Ok(tls_stream) => {
                info!("TLS handshake completed for: {}", host_port);

                // Create HTTP service for the TLS connection
                let service = hyper::service::service_fn(move |req: Request<Incoming>| {
                    let snapshot = Arc::clone(&snapshot);
                    let host_port = host_port.clone();
                    async move { Self::handle_tunneled_request(snapshot, req, host_port).await }
                });

                // Serve HTTP over TLS
                let builder = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                );
                if let Err(e) = builder
                    .serve_connection(TokioIo::new(tls_stream), service)
                    .await
                {
                    // Only log actual errors, not normal connection closures
                    let error_str = e.to_string();
                    if !error_str.contains("connection closed")
                        && !error_str.contains("broken pipe")
                        && !error_str.contains("Connection reset by peer")
                    {
                        error!("TLS tunnel connection error: {}", e);
                    } else {
                        debug!("TLS tunnel connection closed normally: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("TLS handshake failed for {}: {}", host_port, e);
                return Err(Box::new(e));
            }
        }

        Ok(())
    }

    /// Handle HTTP requests that come through the HTTPS tunnel
    async fn handle_tunneled_request(
        snapshot: Arc<Snapshot>,
        req: Request<Incoming>,
        host_port: String,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let method = req.method().clone();
        let uri = req.uri().clone();

        // Reconstruct the full HTTPS URL
        let full_url = if uri.to_string().starts_with("http") {
            uri.to_string()
        } else {
            // For tunneled requests, reconstruct the full HTTPS URL
            let path_query = if let Some(query) = uri.query() {
                format!("{}?{}", uri.path(), query)
            } else {
                uri.path().to_string()
            };
            format!("https://{}{}", host_port, path_query)
        };

        debug!("Handling tunneled request: {} {}", method, full_url);

        // Find matching request record
        match find_matching_record(&snapshot, &method, &full_url) {
            Some(record) => {
                let status_icon = if record.response.status >= 200 && record.response.status < 300 {
                    "✅"
                } else if record.response.status >= 400 {
                    "❌"
                } else {
                    "ℹ️"
                };

                println!(
                    "{} {} {} → {} ({})",
                    status_icon,
                    method,
                    full_url,
                    record.response.status,
                    record
                        .response
                        .headers
                        .get("content-type")
                        .unwrap_or(&"unknown".to_string())
                );

                info!(
                    "Found matching tunneled record: {} {} → {}",
                    method, full_url, record.response.status
                );
                Ok(create_response_from_record(record))
            }
            None => {
                println!("❌ {} {} → 404 (not found in snapshot)", method, full_url);
                warn!("Tunneled request not in snapshot: {} {}", method, full_url);

                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Connection", "close")
                    .body(Full::new(Bytes::from(format!(
                        "Tunneled request not found in snapshot: {} {}",
                        method, full_url
                    ))))
                    .unwrap())
            }
        }
    }
}
