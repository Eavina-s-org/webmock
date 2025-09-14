use mime::Mime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serialization::body_serialization;
use crate::capture::proxy::content_type::ContentTypeHelper;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRecord {
    pub status: u16,
    pub headers: HashMap<String, String>,
    #[serde(with = "body_serialization")]
    pub body: Vec<u8>,
    pub content_type: String,
}

impl ResponseRecord {
    /// Create a new ResponseRecord with content type detection
    pub fn new(
        status: u16,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        url: Option<&str>,
    ) -> Self {
        let content_type = Self::detect_content_type(&headers, &body, url);

        Self {
            status,
            headers,
            body,
            content_type,
        }
    }

    /// Detect content type from headers, body content, or URL extension
    pub fn detect_content_type(
        headers: &HashMap<String, String>,
        body: &[u8],
        url: Option<&str>,
    ) -> String {
        // First, try to get content type from headers
        if let Some(ct) = headers
            .get("content-type")
            .or_else(|| headers.get("Content-Type"))
        {
            return ct.clone();
        }

        // If no content-type header, try to guess from URL extension
        if let Some(url_str) = url {
            if let Ok(url_parsed) = url::Url::parse(url_str) {
                let path = url_parsed.path();
                if let Some(guessed) = mime_guess::from_path(path).first() {
                    return guessed.to_string();
                }
            }
        }

        // If still no content type, try to detect from body content
        ContentTypeHelper::detect_from_body(body)
    }

    /// Get the parsed MIME type of the response
    pub fn get_mime_type(&self) -> Option<Mime> {
        self.content_type.parse().ok()
    }

    /// Check if the response body is text-based
    pub fn is_text_content(&self) -> bool {
        match self.get_mime_type() {
            Some(mime) => ContentTypeHelper::is_text_mime(&mime),
            None => ContentTypeHelper::is_likely_text(&self.body),
        }
    }

    /// Check if the response is an image
    pub fn is_image(&self) -> bool {
        match self.get_mime_type() {
            Some(mime) => mime.type_() == mime::IMAGE,
            None => ContentTypeHelper::is_likely_image(&self.body),
        }
    }

    /// Check if the response is JSON
    pub fn is_json(&self) -> bool {
        match self.get_mime_type() {
            Some(mime) => {
                mime == mime::APPLICATION_JSON
                    || (mime.type_() == mime::APPLICATION
                        && mime.subtype().as_str().ends_with("json"))
            }
            None => ContentTypeHelper::is_likely_json(&self.body),
        }
    }

    /// Check if the response is HTML
    pub fn is_html(&self) -> bool {
        match self.get_mime_type() {
            Some(mime) => mime == mime::TEXT_HTML,
            None => ContentTypeHelper::is_likely_html(&self.body),
        }
    }

    /// Get the response body as a string if it's text-based
    pub fn get_body_as_string(&self) -> Option<String> {
        if self.is_text_content() {
            String::from_utf8(self.body.clone()).ok()
        } else {
            None
        }
    }

    /// Get the size of the response body in bytes
    pub fn get_body_size(&self) -> usize {
        self.body.len()
    }

    /// Check if the response indicates success (2xx status codes)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Check if the response is a redirect (3xx status codes)
    pub fn is_redirect(&self) -> bool {
        self.status >= 300 && self.status < 400
    }

    /// Check if the response is a client error (4xx status codes)
    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    /// Check if the response is a server error (5xx status codes)
    pub fn is_server_error(&self) -> bool {
        self.status >= 500 && self.status < 600
    }
}
