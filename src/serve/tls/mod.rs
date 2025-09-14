use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::pki_types::PrivateKeyDer;
use rustls::ServerConfig;
use std::sync::Arc as StdArc;
use tracing::debug;

/// TLS configuration and certificate management for HTTPS proxy
pub struct TlsConfig;

impl TlsConfig {
    /// Generate a self-signed certificate for TLS
    pub fn generate_tls_config(
    ) -> std::result::Result<StdArc<ServerConfig>, Box<dyn std::error::Error + Send + Sync>> {
        // Generate a self-signed certificate with more domains to reduce warnings
        let subject_alt_names = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "*.baidu.com".to_string(),
            "www.baidu.com".to_string(),
            "*.google.com".to_string(),
            "*.github.com".to_string(),
            // Add more common domains as needed
        ];
        let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Convert to rustls format
        let cert_der = cert.der().clone();
        let key_der = PrivateKeyDer::try_from(key_pair.serialize_der())
            .map_err(|_| "Failed to serialize private key")?;

        // Create TLS config
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert_der], key_der)
            .map_err(|e| format!("Failed to create TLS config: {}", e))?;

        debug!("TLS configuration generated successfully");
        Ok(StdArc::new(config))
    }
}
