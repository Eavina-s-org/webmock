/// Helper for text content type detection
pub struct TextDetector;

impl TextDetector {
    /// Check if body content is likely to be HTML
    pub fn is_likely_html(body: &[u8]) -> bool {
        if body.len() < 5 {
            return false;
        }

        let text = String::from_utf8_lossy(body).to_lowercase();
        text.starts_with("<!doctype html")
            || text.starts_with("<html")
            || text.contains("<html>")
            || text.contains("</html>")
    }

    /// Check if body content is likely to be JSON
    pub fn is_likely_json(body: &[u8]) -> bool {
        if body.is_empty() {
            return false;
        }

        let trimmed = body
            .iter()
            .skip_while(|&&b| b.is_ascii_whitespace())
            .take_while(|&&b| !b.is_ascii_whitespace() || body.len() < 2)
            .copied()
            .collect::<Vec<u8>>();

        if trimmed.is_empty() {
            return false;
        }

        (trimmed[0] == b'{' || trimmed[0] == b'[')
            && String::from_utf8_lossy(body)
                .parse::<serde_json::Value>()
                .is_ok()
    }

    /// Check if body content is likely to be XML
    pub fn is_likely_xml(body: &[u8]) -> bool {
        if body.len() < 5 {
            return false;
        }

        let text = String::from_utf8_lossy(body);
        text.trim_start().starts_with("<?xml")
            || (text.trim_start().starts_with('<') && text.contains("</"))
    }

    /// Check if body content is likely to be text (UTF-8)
    pub fn is_likely_text(body: &[u8]) -> bool {
        if body.is_empty() {
            return true;
        }

        // Check if the content is valid UTF-8 and contains mostly printable characters
        if let Ok(text) = std::str::from_utf8(body) {
            let printable_count = text
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .count();

            let total_chars = text.chars().count();
            if total_chars == 0 {
                return true;
            }

            // Consider it text if at least 80% of characters are printable
            (printable_count as f64 / total_chars as f64) >= 0.8
        } else {
            false
        }
    }
}
