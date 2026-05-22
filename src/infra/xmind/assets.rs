#![allow(dead_code)]

pub fn detect_supported_image_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.starts_with(b"\xff\xd8\xff") {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if looks_like_svg(bytes) {
        return Some("image/svg+xml");
    }

    None
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let text = text.trim_start_matches('\u{feff}').trim_start();
    text.starts_with("<svg") || text.starts_with("<?xml") && text.contains("<svg")
}

#[cfg(test)]
mod tests {
    use super::detect_supported_image_media_type;

    #[test]
    fn detects_png_jpeg_gif_and_svg_media_types() {
        assert_eq!(
            detect_supported_image_media_type(b"\x89PNG\r\n\x1a\nrest"),
            Some("image/png")
        );
        assert_eq!(
            detect_supported_image_media_type(b"\xff\xd8\xff\xe0rest"),
            Some("image/jpeg")
        );
        assert_eq!(
            detect_supported_image_media_type(b"GIF89arest"),
            Some("image/gif")
        );
        assert_eq!(
            detect_supported_image_media_type(br#"<?xml version="1.0"?><svg></svg>"#),
            Some("image/svg+xml")
        );
    }

    #[test]
    fn rejects_unknown_image_media_type() {
        assert_eq!(detect_supported_image_media_type(b"BMbitmap"), None);
    }
}
