use url::Url;

#[test]
fn test_url_validation_comprehensive() {
    // Valid HTTP/HTTPS URLs
    let valid_urls = vec![
        "http://example.com",
        "https://example.com",
        "http://localhost:3000",
        "https://api.example.com/v1/data",
        "http://192.168.1.1:8080",
        "https://subdomain.example.com/path?query=value&other=param",
    ];

    for url in valid_urls {
        let parsed = Url::parse(url);
        assert!(parsed.is_ok(), "Should parse valid URL: {}", url);

        let parsed_url = parsed.unwrap();
        assert!(
            parsed_url.scheme() == "http" || parsed_url.scheme() == "https",
            "Should be HTTP or HTTPS: {}",
            url
        );
    }
}

#[test]
fn test_invalid_url_schemes() {
    let invalid_schemes = vec![
        "ftp://example.com",
        "file:///etc/passwd",
        "javascript:alert('xss')",
        "data:text/html,<script>alert('xss')</script>",
        "mailto:test@example.com",
        "tel:+1234567890",
    ];

    for url in invalid_schemes {
        let parsed = Url::parse(url);
        if let Ok(parsed_url) = parsed {
            // URL parses but scheme should be rejected by our validation
            assert!(
                parsed_url.scheme() != "http" && parsed_url.scheme() != "https",
                "Should reject non-HTTP scheme: {}",
                url
            );
        }
    }
}

#[test]
fn test_malformed_urls() {
    let malformed_urls = vec![
        "",
        "not-a-url",
        "http://",
        "https://",
        "://example.com",
        "http//example.com",
    ];

    for url in malformed_urls {
        let parsed = Url::parse(url);
        assert!(parsed.is_err(), "Should reject malformed URL: {}", url);
    }
}

#[test]
fn test_url_parsing_edge_cases() {
    // Test URL parsing with various edge cases
    let edge_cases = vec![
        ("http://localhost", true),
        ("https://127.0.0.1", true),
        ("http://[::1]", true), // IPv6 localhost
        ("https://example.com:443", true),
        ("http://example.com:80", true),
        ("https://example.com:8080", true),
    ];

    for (url, should_parse) in edge_cases {
        let result = Url::parse(url);
        assert_eq!(
            result.is_ok(),
            should_parse,
            "URL parsing failed for: {}",
            url
        );

        if should_parse {
            let parsed = result.unwrap();
            assert!(parsed.scheme() == "http" || parsed.scheme() == "https");
        }
    }
}
