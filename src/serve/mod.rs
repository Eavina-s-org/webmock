#[cfg(test)]
mod tests;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

use crate::error::{Result, WebMockError};
use crate::storage::Snapshot;

mod handlers;
mod proxy;
mod tls;

use proxy::ProxyHandler;

pub struct MockServer {
    snapshot: Arc<Snapshot>,
}

impl MockServer {
    pub fn new(snapshot: Snapshot) -> Self {
        info!("Creating mock server for snapshot: {}", snapshot.name);
        Self {
            snapshot: Arc::new(snapshot),
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        info!("Starting mock server on port: {}", port);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let snapshot = Arc::clone(&self.snapshot);

        // Report successful startup
        println!("✅ Mock proxy server started successfully!");
        println!("   📡 Proxy listening on: http://{}", addr);
        println!(
            "   📦 Snapshot: {} ({} requests)",
            self.snapshot.name,
            self.snapshot.requests.len()
        );
        println!("   🌍 Original URL: {}", self.snapshot.url);
        println!(
            "   🔧 HTTP Usage: curl -x http://{} http://www.baidu.com/",
            addr
        );
        println!(
            "   🔐 HTTPS Usage: curl -x http://{} https://www.baidu.com/ --insecure",
            addr
        );
        println!("   ⚠️  Use --insecure/-k with curl due to self-signed certificate");
        println!(
            "   💡 Alternative: curl -x http://{} https://www.baidu.com/ -k",
            addr
        );

        // Debug: show first few URLs in snapshot
        println!("   📋 Sample URLs in snapshot:");
        for (i, request) in self.snapshot.requests.iter().take(5).enumerate() {
            println!("      {}. {} {}", i + 1, request.method, request.url);
        }
        if self.snapshot.requests.len() > 5 {
            println!("      ... and {} more", self.snapshot.requests.len() - 5);
        }
        println!();

        info!("Mock proxy server running on http://{}", addr);
        info!(
            "Serving snapshot: {} (captured from: {})",
            self.snapshot.name, self.snapshot.url
        );

        // Bind to the address
        let listener = match TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind to {}: {}", addr, e);

                // Provide user-friendly error messages for common issues
                let error_msg = if e.to_string().contains("Address already in use") {
                    format!("Port {} is already in use by another process", port)
                } else if e.to_string().contains("Permission denied") {
                    format!(
                        "Permission denied to bind to port {}. Try using a port number above 1024",
                        port
                    )
                } else {
                    format!("Server failed to start: {}", e)
                };

                return Err(WebMockError::Proxy(error_msg));
            }
        };

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let snapshot = Arc::clone(&snapshot);
                    tokio::spawn(async move {
                        let io = TokioIo::new(stream);
                        let service = service_fn(move |req| {
                            let snapshot = Arc::clone(&snapshot);
                            Self::handle_request_internal(snapshot, req)
                        });

                        let builder = Builder::new(hyper_util::rt::TokioExecutor::new());
                        // Use serve_connection_with_upgrades to support CONNECT tunneling
                        if let Err(e) = builder.serve_connection_with_upgrades(io, service).await {
                            // Only log actual errors, not normal connection closures
                            let error_str = e.to_string();
                            if !error_str.contains("connection closed")
                                && !error_str.contains("broken pipe")
                                && !error_str.contains("Connection reset by peer")
                            {
                                error!("Connection error: {}", e);
                            } else {
                                debug!("Connection closed normally: {}", e);
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    return Err(WebMockError::Proxy(format!("Accept error: {}", e)));
                }
            }
        }
    }

    async fn handle_request_internal(
        snapshot: Arc<Snapshot>,
        req: Request<Incoming>,
    ) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
        let method = req.method().clone();

        // Route to appropriate handler
        if method == hyper::Method::CONNECT {
            let host_port = req
                .uri()
                .authority()
                .map(|auth| auth.to_string())
                .unwrap_or_else(|| "unknown:443".to_string());

            ProxyHandler::handle_connect_request(snapshot, req, host_port).await
        } else {
            ProxyHandler::handle_http_request(snapshot, req).await
        }
    }
}
