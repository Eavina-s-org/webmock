use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::response::ResponseRecord;
use super::serialization::optional_body_serialization;
use crate::capture::proxy::content_type::ContentTypeHelper;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRecord {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    #[serde(with = "optional_body_serialization")]
    pub body: Option<Vec<u8>>,
    pub response: ResponseRecord,
    pub timestamp: DateTime<Utc>,
}

impl RequestRecord {
    /// Create a new RequestRecord with the current timestamp
    pub fn new(
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
        response: ResponseRecord,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            body,
            response,
            timestamp: Utc::now(),
        }
    }

    /// Get the content type of the request body from headers
    pub fn get_request_content_type(&self) -> Option<Mime> {
        self.headers
            .get("content-type")
            .or_else(|| self.headers.get("Content-Type"))
            .and_then(|ct| ct.parse().ok())
    }

    /// Check if the request body is likely to be text-based
    pub fn is_request_body_text(&self) -> bool {
        match self.get_request_content_type() {
            Some(mime) => ContentTypeHelper::is_text_mime(&mime),
            None => false,
        }
    }

    /// Get the request body as a string if it's text-based
    pub fn get_request_body_as_string(&self) -> Option<String> {
        if let Some(body) = &self.body {
            if self.is_request_body_text() {
                return String::from_utf8(body.clone()).ok();
            }
        }
        None
    }

    /// Get the size of the request body in bytes
    pub fn get_request_body_size(&self) -> usize {
        self.body.as_ref().map(|b| b.len()).unwrap_or(0)
    }
}
