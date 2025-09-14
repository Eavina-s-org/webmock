use super::image::ImageDetector;
use super::text::TextDetector;
use mime::Mime;

/// Helper struct for content type detection and handling
pub struct ContentTypeHelper;

impl ContentTypeHelper {
    /// Check if a MIME type represents text content
    pub fn is_text_mime(mime: &Mime) -> bool {
        match mime.type_() {
            mime::TEXT => true,
            mime::APPLICATION => {
                let subtype = mime.subtype();
                subtype == mime::JSON
                    || subtype.as_str().ends_with("json")
                    || subtype.as_str().ends_with("xml")
                    || subtype.as_str().ends_with("javascript")
                    || subtype == "x-www-form-urlencoded"
            }
            _ => false,
        }
    }

    /// Detect content type from body content using heuristics
    pub fn detect_from_body(body: &[u8]) -> String {
        if body.is_empty() {
            return "application/octet-stream".to_string();
        }

        // Check for common file signatures
        if Self::is_likely_html(body) {
            return "text/html".to_string();
        }

        if Self::is_likely_json(body) {
            return "application/json".to_string();
        }

        if Self::is_likely_xml(body) {
            return "application/xml".to_string();
        }

        if ImageDetector::is_likely_image(body) {
            return ImageDetector::detect_image_type(body);
        }

        if TextDetector::is_likely_text(body) {
            return "text/plain".to_string();
        }

        "application/octet-stream".to_string()
    }

    /// Check if body content is likely to be HTML
    pub fn is_likely_html(body: &[u8]) -> bool {
        TextDetector::is_likely_html(body)
    }

    /// Check if body content is likely to be JSON
    pub fn is_likely_json(body: &[u8]) -> bool {
        TextDetector::is_likely_json(body)
    }

    /// Check if body content is likely to be XML
    pub fn is_likely_xml(body: &[u8]) -> bool {
        TextDetector::is_likely_xml(body)
    }

    /// Check if body content is likely to be an image
    pub fn is_likely_image(body: &[u8]) -> bool {
        ImageDetector::is_likely_image(body)
    }

    /// Detect specific image type from body content
    pub fn detect_image_type(body: &[u8]) -> String {
        ImageDetector::detect_image_type(body)
    }

    /// Check if body content is likely to be text (UTF-8)
    pub fn is_likely_text(body: &[u8]) -> bool {
        TextDetector::is_likely_text(body)
    }
}
