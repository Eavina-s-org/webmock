/// Helper for image content type detection
pub struct ImageDetector;

impl ImageDetector {
    /// Check if body content is likely to be an image
    pub fn is_likely_image(body: &[u8]) -> bool {
        if body.len() < 4 {
            return false;
        }

        // Check for common image file signatures
        matches!(
            body[0..4],
            [0xFF, 0xD8, 0xFF, _] |  // JPEG
            [0x89, 0x50, 0x4E, 0x47] |  // PNG
            [0x47, 0x49, 0x46, 0x38] |  // GIF
            [0x52, 0x49, 0x46, 0x46] |  // WebP (RIFF)
            [0x42, 0x4D, _, _] // BMP
        )
    }

    /// Detect specific image type from body content
    pub fn detect_image_type(body: &[u8]) -> String {
        if body.len() < 4 {
            return "image/unknown".to_string();
        }

        match &body[0..4] {
            [0xFF, 0xD8, 0xFF, _] => "image/jpeg".to_string(),
            [0x89, 0x50, 0x4E, 0x47] => "image/png".to_string(),
            [0x47, 0x49, 0x46, 0x38] => "image/gif".to_string(),
            [0x52, 0x49, 0x46, 0x46] => {
                // Check for WebP signature
                if body.len() >= 12 && &body[8..12] == b"WEBP" {
                    "image/webp".to_string()
                } else {
                    "image/unknown".to_string()
                }
            }
            [0x42, 0x4D, _, _] => "image/bmp".to_string(),
            _ => "image/unknown".to_string(),
        }
    }
}
