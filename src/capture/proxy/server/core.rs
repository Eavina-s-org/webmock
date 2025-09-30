use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::pki_types::PrivateKeyDer;
use rustls::server::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use super::handlers::{handle_connect_mitm, handle_request};
use crate::capture::proxy::client_pool::HttpClientPool;
use crate::capture::proxy::recorder::RequestRecorder;
use crate::capture::proxy::records::RequestRecord;
use crate::error::Result;

pub struct HttpProxy {
    port: u16,
    recorder: Arc<RequestRecorder>,
    client_pool: Arc<HttpClientPool>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    #[allow(dead_code)]
    tls_config: Option<Arc<ServerConfig>>,
}

impl HttpProxy {
    /// Generate a self-signed certificate for TLS MITM
    fn generate_tls_config() -> Result<Arc<ServerConfig>> {
        // Generate a self-signed certificate with broad domain coverage
        let subject_alt_names = [
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
            "*.com".to_string(),
            "*.org".to_string(),
            "*.net".to_string(),
            "*.io".to_string(),
            "*.co.uk".to_string(),
            "*.edu".to_string(),
            "*.gov".to_string(),
            "*.mil".to_string(),
            "*.info".to_string(),
            "*.biz".to_string(),
            "*.name".to_string(),
            "*.tv".to_string(),
            "*.me".to_string(),
            "*.dev".to_string(),
            "*.app".to_string(),
            "*.cloud".to_string(),
            "*.ai".to_string(),
            "*.cn".to_string(),
            "*.jp".to_string(),
            "*.de".to_string(),
            "*.fr".to_string(),
            "*.ru".to_string(),
            "*.in".to_string(),
            "*.br".to_string(),
            "*.ca".to_string(),
            "*.au".to_string(),
            "*.kr".to_string(),
            "*.it".to_string(),
            "*.es".to_string(),
            "*.mx".to_string(),
            "*.ar".to_string(),
            "*.za".to_string(),
            "*.ng".to_string(),
            "*.eg".to_string(),
            "*.sa".to_string(),
            "*.ae".to_string(),
            "*.tr".to_string(),
            "*.id".to_string(),
            "*.th".to_string(),
            "*.vn".to_string(),
            "*.ph".to_string(),
            "*.my".to_string(),
            "*.sg".to_string(),
            "*.tw".to_string(),
            "*.hk".to_string(),
            "*.mo".to_string(),
        ];

        let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
            .map_err(|e| {
                crate::error::WebMockError::config(format!(
                    "Failed to generate TLS certificate: {}",
                    e
                ))
            })?;

        // Convert to rustls format
        let cert_der = cert.der().clone();
        let key_der = PrivateKeyDer::try_from(key_pair.serialize_der())
            .map_err(|_| crate::error::WebMockError::config("Failed to serialize private key"))?;

        // Create TLS config with optimized settings
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .map_err(|e| {
                crate::error::WebMockError::config(format!("Failed to create TLS config: {}", e))
            })?;

        // Optimize for compatibility
        config.alpn_protocols = vec![b"http/1.1".to_vec(), b"h2".to_vec()];

        Ok(Arc::new(config))
    }

    pub async fn start(port: u16) -> Result<Self> {
        info!("Starting HTTP proxy on port: {}", port);

        let recorder = Arc::new(RequestRecorder::new());
        let client_pool = Arc::new(HttpClientPool::new());
        let tls_config = Self::generate_tls_config()?;
        let recorder_clone = Arc::clone(&recorder);
        let client_pool_clone = Arc::clone(&client_pool);
        let tls_config_clone = Arc::clone(&tls_config);

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr).await.map_err(|e| {
            crate::error::WebMockError::config(format!("Failed to bind to {}: {}", addr, e))
        })?;

        let server_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, remote_addr)) => {
                                let recorder = Arc::clone(&recorder_clone);
                                let client_pool = Arc::clone(&client_pool_clone);
                                let tls_config = Arc::clone(&tls_config_clone);

                                tokio::spawn(async move {
                                    let io = TokioIo::new(stream);

                                    // Create service that can handle both HTTP and HTTPS MITM
                                    let service = service_fn(move |req| {
                                        let recorder = Arc::clone(&recorder);
                                        let client_pool = Arc::clone(&client_pool);
                                        let tls_config = Arc::clone(&tls_config);
                                        async move {
                                            // Check if this is a CONNECT request for HTTPS
                                            if req.method() == hyper::Method::CONNECT {
                                                handle_connect_mitm(req, recorder, client_pool, tls_config).await
                                            } else {
                                                handle_request(req, recorder, client_pool, remote_addr).await
                                            }
                                        }
                                    });

                                    let builder = Builder::new(hyper_util::rt::TokioExecutor::new());

                                    // Configure connection with upgrades support for CONNECT tunneling
                                    let conn = builder.serve_connection_with_upgrades(io, service);

                                    if let Err(e) = conn.await {
                                        // Only log actual errors, not normal connection closures
                                        let error_str = e.to_string();
                                        if !error_str.contains("connection closed") &&
                                           !error_str.contains("broken pipe") &&
                                           !error_str.contains("Connection reset by peer") {
                                            tracing::warn!("Connection error: {}", e);
                                        } else {
                                            tracing::debug!("Connection closed normally: {}", e);
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Accept error: {}", e);
                                break;
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        info!("Received shutdown signal");
                        break;
                    }
                }
            }
        });

        info!("HTTP proxy started successfully on {}", addr);

        Ok(HttpProxy {
            port,
            recorder,
            client_pool,
            server_handle: Some(server_handle),
            shutdown_tx: Some(shutdown_tx),
            tls_config: Some(tls_config),
        })
    }

    pub async fn get_records(&self) -> Vec<RequestRecord> {
        self.recorder.get_records().await
    }

    pub async fn clear_records(&self) {
        self.recorder.clear_records().await
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub async fn stop(mut self) -> Result<()> {
        info!("Stopping HTTP proxy on port {}", self.port);

        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        // Clean up client pool
        self.client_pool.clear().await;

        info!("HTTP proxy stopped successfully");
        Ok(())
    }
}
