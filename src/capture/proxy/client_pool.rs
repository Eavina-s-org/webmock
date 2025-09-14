use bytes::Bytes;
use http_body_util::Full;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

type HttpsClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;
type SharedClient = Arc<HttpsClient>;
type ClientMap = HashMap<String, SharedClient>;

/// HTTP client pool for efficient connection reuse
pub struct HttpClientPool {
    clients: Arc<RwLock<ClientMap>>,
    max_idle_per_host: usize,
}

impl HttpClientPool {
    /// Create a new HTTP client pool
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            max_idle_per_host: 10,
        }
    }

    /// Get or create a client for the given host
    pub async fn get_client(
        &self,
        host: &str,
    ) -> Arc<Client<HttpsConnector<HttpConnector>, Full<Bytes>>> {
        // Check if we already have a client for this host
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(host) {
                return Arc::clone(client);
            }
        }

        // Create a new client with optimized settings
        let https = HttpsConnector::new();
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(self.max_idle_per_host)
            .http2_only(false)
            .http2_keep_alive_timeout(Duration::from_secs(30))
            .http2_keep_alive_interval(Some(Duration::from_secs(15)))
            .build(https);

        let client = Arc::new(client);

        // Store the client for reuse
        {
            let mut clients = self.clients.write().await;
            clients.insert(host.to_string(), Arc::clone(&client));
        }

        client
    }

    /// Clear all cached clients (useful for cleanup)
    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        clients.clear();
    }

    /// Get the number of cached clients
    pub async fn client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
}

impl Default for HttpClientPool {
    fn default() -> Self {
        Self::new()
    }
}
