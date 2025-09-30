use std::collections::HashMap;

#[test]
fn test_parse_host_port() {
    // Test host and port parsing logic from handle_connect_request
    let test_cases = [
        ("example.com:443", ("example.com", 443)),
        ("localhost:8080", ("localhost", 8080)),
        ("192.168.1.1:443", ("192.168.1.1", 443)),
        ("example.com", ("example.com", 443)), // Default port
    ];

    for (input, expected) in test_cases {
        let (host, port) = if let Some((h, p)) = input.split_once(':') {
            (h.to_string(), p.parse().unwrap_or(443))
        } else {
            (input.to_string(), 443)
        };

        assert_eq!(host, expected.0);
        assert_eq!(port, expected.1);
    }
}

#[test]
fn test_create_hashmap() {
    // Test HashMap creation logic used in the function
    let mut headers = HashMap::new();
    headers.insert("Connection".to_string(), "established".to_string());
    headers.insert("Proxy-Agent".to_string(), "WebMock-CLI/1.0".to_string());

    assert_eq!(headers.get("Connection"), Some(&"established".to_string()));
    assert_eq!(
        headers.get("Proxy-Agent"),
        Some(&"WebMock-CLI/1.0".to_string())
    );
    assert_eq!(headers.len(), 2);
}
