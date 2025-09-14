use crate::capture::proxy::content_type::ContentTypeHelper;
use crate::capture::proxy::records::ResponseRecord;
use std::collections::HashMap;

#[test]
fn test_content_type_helper_text_mime() {
    assert!(ContentTypeHelper::is_text_mime(&mime::TEXT_PLAIN));
    assert!(ContentTypeHelper::is_text_mime(&mime::TEXT_HTML));
    assert!(ContentTypeHelper::is_text_mime(&mime::APPLICATION_JSON));
    assert!(!ContentTypeHelper::is_text_mime(&mime::IMAGE_PNG));
    assert!(!ContentTypeHelper::is_text_mime(
        &mime::APPLICATION_OCTET_STREAM
    ));
}

#[test]
fn test_content_type_helper_html_detection() {
    assert!(ContentTypeHelper::is_likely_html(
        b"<!DOCTYPE html><html></html>"
    ));
    assert!(ContentTypeHelper::is_likely_html(
        b"<html><body>Test</body></html>"
    ));
    assert!(ContentTypeHelper::is_likely_html(
        b"<HTML><BODY>Test</BODY></HTML>"
    ));
    assert!(!ContentTypeHelper::is_likely_html(b"Just plain text"));
    assert!(!ContentTypeHelper::is_likely_html(b"{\"json\": true}"));
}

#[test]
fn test_content_type_helper_json_detection() {
    assert!(ContentTypeHelper::is_likely_json(b"{\"key\": \"value\"}"));
    assert!(ContentTypeHelper::is_likely_json(b"[1, 2, 3]"));
    assert!(ContentTypeHelper::is_likely_json(b"  {\"spaced\": true}  "));
    assert!(!ContentTypeHelper::is_likely_json(b"<html></html>"));
    assert!(!ContentTypeHelper::is_likely_json(b"plain text"));
    assert!(!ContentTypeHelper::is_likely_json(b""));
}

#[test]
fn test_content_type_helper_xml_detection() {
    assert!(ContentTypeHelper::is_likely_xml(
        b"<?xml version=\"1.0\"?><root></root>"
    ));
    assert!(ContentTypeHelper::is_likely_xml(
        b"<root><child>value</child></root>"
    ));
    assert!(!ContentTypeHelper::is_likely_xml(b"{\"json\": true}"));
    assert!(!ContentTypeHelper::is_likely_xml(b"plain text"));
}

#[test]
fn test_content_type_helper_image_detection() {
    // JPEG signature
    let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    assert!(ContentTypeHelper::is_likely_image(&jpeg_data));
    assert_eq!(
        ContentTypeHelper::detect_image_type(&jpeg_data),
        "image/jpeg"
    );

    // PNG signature
    let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert!(ContentTypeHelper::is_likely_image(&png_data));
    assert_eq!(ContentTypeHelper::detect_image_type(&png_data), "image/png");

    // GIF signature
    let gif_data = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61];
    assert!(ContentTypeHelper::is_likely_image(&gif_data));
    assert_eq!(ContentTypeHelper::detect_image_type(&gif_data), "image/gif");

    // Not an image
    assert!(!ContentTypeHelper::is_likely_image(b"text data"));
}

#[test]
fn test_content_type_helper_text_detection() {
    assert!(ContentTypeHelper::is_likely_text(b"Hello, World!"));
    assert!(ContentTypeHelper::is_likely_text(b"Multi\nline\ntext"));
    assert!(ContentTypeHelper::is_likely_text(b"Text with numbers 123"));
    assert!(ContentTypeHelper::is_likely_text(b""));

    // Binary data should not be detected as text
    let binary_data = [0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE];
    assert!(!ContentTypeHelper::is_likely_text(&binary_data));
}

#[test]
fn test_content_type_detection_from_body() {
    assert_eq!(
        ContentTypeHelper::detect_from_body(b"<html></html>"),
        "text/html"
    );
    assert_eq!(
        ContentTypeHelper::detect_from_body(b"{\"json\": true}"),
        "application/json"
    );
    assert_eq!(
        ContentTypeHelper::detect_from_body(b"<?xml version=\"1.0\"?><root/>"),
        "application/xml"
    );
    assert_eq!(
        ContentTypeHelper::detect_from_body(b"plain text"),
        "text/plain"
    );

    let jpeg_data = [0xFF, 0xD8, 0xFF, 0xE0];
    assert_eq!(
        ContentTypeHelper::detect_from_body(&jpeg_data),
        "image/jpeg"
    );

    let binary_data = [0x00, 0x01, 0x02, 0x03];
    assert_eq!(
        ContentTypeHelper::detect_from_body(&binary_data),
        "application/octet-stream"
    );

    assert_eq!(
        ContentTypeHelper::detect_from_body(b""),
        "application/octet-stream"
    );
}

#[test]
fn test_webp_image_detection() {
    // WebP signature: RIFF....WEBP
    let webp_data = [
        0x52, 0x49, 0x46, 0x46, // RIFF
        0x00, 0x00, 0x00, 0x00, // file size (placeholder)
        0x57, 0x45, 0x42, 0x50, // WEBP
    ];
    assert!(ContentTypeHelper::is_likely_image(&webp_data));
    assert_eq!(
        ContentTypeHelper::detect_image_type(&webp_data),
        "image/webp"
    );
}

#[test]
fn test_bmp_image_detection() {
    // BMP signature: BM
    let bmp_data = [0x42, 0x4D, 0x00, 0x00, 0x00, 0x00];
    assert!(ContentTypeHelper::is_likely_image(&bmp_data));
    assert_eq!(ContentTypeHelper::detect_image_type(&bmp_data), "image/bmp");
}

#[test]
fn test_response_record_detect_content_type_priority() {
    // Test that header content-type takes priority over URL guessing
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let content_type = ResponseRecord::detect_content_type(
        &headers,
        b"<html></html>",                      // HTML body
        Some("https://example.com/file.html"), // HTML URL
    );

    // Should use header content-type, not guess from URL or body
    assert_eq!(content_type, "application/json");
}

#[test]
fn test_response_record_detect_content_type_from_url() {
    let headers = HashMap::new();

    let content_type = ResponseRecord::detect_content_type(
        &headers,
        b"some content",
        Some("https://example.com/image.png"),
    );

    // Should guess from URL extension
    assert_eq!(content_type, "image/png");
}

#[test]
fn test_response_record_detect_content_type_from_body() {
    let headers = HashMap::new();

    let content_type =
        ResponseRecord::detect_content_type(&headers, b"<html><body>Test</body></html>", None);

    // Should detect from body content
    assert_eq!(content_type, "text/html");
}
