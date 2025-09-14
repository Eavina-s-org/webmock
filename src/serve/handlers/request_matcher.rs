//! Request matching logic for mock server

use crate::capture::proxy::RequestRecord;
use crate::storage::Snapshot;
use hyper::Method;
use tracing::debug;
use url::Url;

/// Find a matching recorded request for the incoming request
pub fn find_matching_record<'a>(
    snapshot: &'a Snapshot,
    method: &Method,
    full_url: &str,
) -> Option<&'a RequestRecord> {
    let method_str = method.as_str();

    // Debug: show snapshot summary
    let method_counts: std::collections::HashMap<String, usize> =
        snapshot
            .requests
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, req| {
                *acc.entry(req.method.clone()).or_insert(0) += 1;
                acc
            });
    debug!(
        "Snapshot contains {} total requests: {:?}",
        snapshot.requests.len(),
        method_counts
    );

    // Handle CONNECT method specially
    if method == Method::CONNECT {
        debug!("Looking for CONNECT match for: {}", full_url);

        // Debug: show all CONNECT records in snapshot
        let connect_records: Vec<_> = snapshot
            .requests
            .iter()
            .filter(|r| r.method == "CONNECT")
            .collect();
        debug!(
            "Found {} CONNECT records in snapshot",
            connect_records.len()
        );
        for (i, record) in connect_records.iter().enumerate() {
            debug!("CONNECT record {}: {}", i, record.url);
        }

        for record in &snapshot.requests {
            if record.method == "CONNECT" {
                debug!("Comparing against recorded CONNECT: {}", record.url);

                // For CONNECT, we need to handle the special format
                // CONNECT requests are recorded as "https://host:port" but the actual request
                // might come as just "host:port"

                // Normalize both URLs to host:port format
                let normalize_connect_url = |url: &str| -> String {
                    // Handle both "https://host:port" and "host:port" formats
                    if url.starts_with("https://") {
                        url.replace("https://", "")
                    } else if url.starts_with("http://") {
                        url.replace("http://", "")
                    } else {
                        url.to_string()
                    }
                };

                let normalized_recorded = normalize_connect_url(&record.url);
                let normalized_request = normalize_connect_url(full_url);

                debug!(
                    "Normalized comparison: '{}' vs '{}'",
                    normalized_recorded, normalized_request
                );

                if normalized_recorded == normalized_request {
                    debug!("Found normalized CONNECT match!");
                    return Some(record);
                }

                // Also try host-only matching
                if let (Ok(recorded_url), Ok(request_url)) = (
                    Url::parse(&format!("https://{}", normalized_recorded)),
                    Url::parse(&format!("https://{}", normalized_request)),
                ) {
                    if recorded_url.host_str() == request_url.host_str() {
                        debug!("Found host-only CONNECT match!");
                        return Some(record);
                    }
                }
            }
        }

        debug!("No CONNECT match found for: {}", full_url);
        return None;
    }

    // Parse the request URL to extract components
    let request_url = match Url::parse(full_url) {
        Ok(url) => url,
        Err(e) => {
            debug!("Failed to parse request URL: {}", e);
            return None;
        }
    };

    let request_host = request_url.host_str().unwrap_or("");
    let request_path = request_url.path();
    let request_query = request_url.query().unwrap_or("");

    debug!(
        "Request components: host='{}', path='{}', query='{}'",
        request_host, request_path, request_query
    );

    // First, try exact URL + method match
    for record in &snapshot.requests {
        if record.method == method_str && record.url == full_url {
            debug!("Found exact match!");
            return Some(record);
        }
    }

    // Then try host + path + query string match (ignoring protocol)
    for record in &snapshot.requests {
        if record.method == method_str {
            if let Ok(recorded_url) = Url::parse(&record.url) {
                let recorded_host = recorded_url.host_str().unwrap_or("");
                let recorded_path = recorded_url.path();
                let recorded_query = recorded_url.query().unwrap_or("");

                debug!(
                    "Recorded components: host='{}', path='{}', query='{}'",
                    recorded_host, recorded_path, recorded_query
                );

                if recorded_host == request_host
                    && recorded_path == request_path
                    && recorded_query == request_query
                {
                    debug!("Found host+path+query match!");
                    return Some(record);
                }
            }
        }
    }

    // Try host + path match (ignoring query)
    for record in &snapshot.requests {
        if record.method == method_str {
            if let Ok(recorded_url) = Url::parse(&record.url) {
                let recorded_host = recorded_url.host_str().unwrap_or("");
                let recorded_path = recorded_url.path();

                if recorded_host == request_host && recorded_path == request_path {
                    debug!("Found host+path match!");
                    return Some(record);
                }
            }
        }
    }

    // Finally, try path-only match
    debug!("Trying path-only match: path='{}'", request_path);
    for record in &snapshot.requests {
        if record.method == method_str {
            if let Ok(recorded_url) = Url::parse(&record.url) {
                let recorded_path = recorded_url.path();

                if recorded_path == request_path {
                    debug!("Found path-only match!");
                    return Some(record);
                }
            }
        }
    }

    None
}
