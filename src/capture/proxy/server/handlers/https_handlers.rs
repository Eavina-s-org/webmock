use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::Request;
use hyper::{Response, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

use crate::capture::proxy::recorder::RequestRecorder;
use crate::capture::proxy::records::{RequestRecord, ResponseRecord};

pub async fn handle_connect_request(
    req: Request<Incoming>,
    recorder: Arc<RequestRecorder>,
) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    let uri = req.uri();
    let host_port = uri
        .authority()
        .map(|auth| auth.to_string())
        .unwrap_or_else(|| "unknown:443".to_string());

    debug!("Handling CONNECT request to: {}", host_port);

    // Parse host and port
    let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
        (h.to_string(), p.parse().unwrap_or(443))
    } else {
        (host_port.clone(), 443)
    };

    // Try to establish connection to target server first
    match TcpStream::connect(&host_port).await {
        Ok(target_stream) => {
            // Record successful CONNECT request
            let connect_record = RequestRecord::new(
                "CONNECT".to_string(),
                format!("https://{}:{}", host, port),
                HashMap::new(),
                None,
                ResponseRecord::new(
                    200,
                    {
                        let mut headers = HashMap::new();
                        headers.insert("Connection".to_string(), "established".to_string());
                        headers.insert("Proxy-Agent".to_string(), "WebMock-CLI/1.0".to_string());
                        headers
                    },
                    Vec::new(),
                    Some(&format!("https://{}:{}", host, port)),
                ),
            );

            recorder.record_request(connect_record).await;
            info!(
                "CONNECT request to {} recorded - tunnel established",
                host_port
            );

            // Start the tunnel in a separate task after the response is sent
            tokio::spawn(async move {
                if let Ok(upgraded) = hyper::upgrade::on(req).await {
                    if let Err(e) = crate::capture::proxy::server::handlers::tunnel::tunnel_data(
                        upgraded,
                        target_stream,
                    )
                    .await
                    {
                        error!("Tunnel error: {}", e);
                    }
                } else {
                    warn!("Failed to upgrade connection for tunnel");
                }
            });

            // Return 200 Connection Established
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Connection", "upgrade")
                .header("Proxy-Agent", "WebMock-CLI/1.0")
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
        Err(e) => {
            warn!("Failed to connect to {}: {}", host_port, e);

            // Record failed connection
            let error_record = RequestRecord::new(
                "CONNECT".to_string(),
                format!("https://{}:{}", host, port),
                HashMap::new(),
                None,
                ResponseRecord::new(
                    502,
                    HashMap::new(),
                    format!("Failed to connect to {}: {}", host_port, e).into_bytes(),
                    Some(&format!("https://{}:{}", host, port)),
                ),
            );

            recorder.record_request(error_record).await;

            Ok(Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .header("Connection", "close")
                .body(Full::new(Bytes::from(format!(
                    "Failed to connect to {}: {}",
                    host_port, e
                ))))
                .unwrap())
        }
    }
}
