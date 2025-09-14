use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use rustls::ServerConfig;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error};

use crate::capture::proxy::client_pool::HttpClientPool;
use crate::capture::proxy::recorder::RequestRecorder;
use crate::capture::proxy::records::{RequestRecord, ResponseRecord};
use crate::capture::proxy::server::handlers::http_handlers::handle_request as handle_http_request;

pub async fn handle_connect_mitm(
    req: Request<Incoming>,
    recorder: Arc<RequestRecorder>,
    client_pool: Arc<HttpClientPool>,
    tls_config: Arc<ServerConfig>,
) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    let uri = req.uri();
    let host_port = uri
        .authority()
        .map(|auth| auth.to_string())
        .unwrap_or_else(|| "unknown:443".to_string());

    debug!("Handling CONNECT MITM request to: {}", host_port);

    // Parse host and port
    let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
        (h.to_string(), p.parse().unwrap_or(443))
    } else {
        (host_port.clone(), 443)
    };

    // Record the CONNECT request
    let host_port = format!("https://{}:{}", host, port);
    let connect_record = RequestRecord::new(
        "CONNECT".to_string(),
        host_port.clone(),
        HashMap::new(),
        None,
        ResponseRecord::new(
            200,
            {
                let mut headers = HashMap::new();
                headers.insert("Connection".to_string(), "established".to_string());
                headers
            },
            Vec::new(),
            Some(&host_port),
        ),
    );
    recorder.record_request(connect_record).await;

    // Create a response indicating connection established
    let response = Response::builder()
        .status(hyper::StatusCode::OK)
        .body(Full::new(Bytes::new()))
        .unwrap();

    // Upgrade the connection to handle TLS
    let upgraded = hyper::upgrade::on(req);
    let tls_config = Arc::clone(&tls_config);
    let recorder = Arc::clone(&recorder);
    let client_pool = Arc::clone(&client_pool);

    tokio::spawn(async move {
        match upgraded.await {
            Ok(upgraded) => {
                let upgraded = TokioIo::new(upgraded);

                // Perform TLS handshake with the client
                let acceptor = TlsAcceptor::from(tls_config);
                match acceptor.accept(upgraded).await {
                    Ok(tls_stream) => {
                        let tls_io = TokioIo::new(tls_stream);

                        // Create a service to handle the decrypted HTTPS traffic
                        let service = hyper::service::service_fn(move |mut req| {
                            let recorder = Arc::clone(&recorder);
                            let client_pool = Arc::clone(&client_pool);
                            let host = host.clone();
                            async move {
                                debug!(
                                    "Processing decrypted HTTPS request: {} {}",
                                    req.method(),
                                    req.uri()
                                );

                                // For HTTPS requests in MITM mode, we need to reconstruct the full URL
                                // since the client sends relative paths after CONNECT
                                let uri = req.uri();
                                let full_url = if uri.path_and_query().is_some() {
                                    let path = uri.path_and_query().unwrap().as_str();
                                    if path.starts_with("http") {
                                        // Already a full URL
                                        uri.to_string()
                                    } else {
                                        // Relative path, need to prepend https://host
                                        format!("https://{}{}", host, path)
                                    }
                                } else {
                                    // Just path
                                    format!("https://{}{}", host, uri.path())
                                };

                                // Update the request URI to be the full URL
                                let new_uri: hyper::Uri =
                                    full_url.parse().unwrap_or_else(|_| uri.clone());
                                *req.uri_mut() = new_uri;

                                debug!("Reconstructed HTTPS URL: {}", req.uri());
                                handle_http_request(
                                    req,
                                    recorder,
                                    client_pool,
                                    "127.0.0.1:0".parse().unwrap(),
                                )
                                .await
                            }
                        });

                        let builder = hyper_util::server::conn::auto::Builder::new(
                            hyper_util::rt::TokioExecutor::new(),
                        );
                        let conn = builder.serve_connection(tls_io, service);

                        if let Err(e) = conn.await {
                            let error_str = e.to_string().to_lowercase();
                            if !error_str.contains("connection closed")
                                && !error_str.contains("broken pipe")
                                && !error_str.contains("connection reset")
                                && !error_str.contains("eof")
                            {
                                error!("HTTPS MITM connection error: {}", e);
                            } else {
                                debug!("HTTPS connection closed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("TLS handshake failed: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to upgrade connection: {}", e);
            }
        }
    });

    Ok(response)
}
