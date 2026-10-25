// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Links to share µcad code in the web viewer, e.g. as QR code.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// The web viewer, which shows the code of a link.
pub const PLAY_URL: &str = "https://play.mucad.org/";

/// The maximum size of the code in a link, in bytes of its UTF-8 encoding.
pub const MAX_CODE_BYTES: usize = 2048;

/// The link to the web viewer with `code`, which is embedded as URL-safe base64,
/// e.g. `https://play.mucad.org/Q2lyY2xlKHIgPSAxbW0pOw` for `Circle(r = 1mm);`.
///
/// Fails, if the code has more than [`MAX_CODE_BYTES`] bytes.
pub fn link(code: &str) -> Result<String, String> {
    if code.len() > MAX_CODE_BYTES {
        return Err(format!(
            "the file has {} bytes, a link can contain at most {MAX_CODE_BYTES} bytes",
            code.len()
        ));
    }
    Ok(format!("{PLAY_URL}{}", URL_SAFE_NO_PAD.encode(code)))
}

/// The QR code of `link` as text for a terminal with a dark background.
pub fn qr_code(link: &str) -> Result<String, String> {
    use qrcode::render::unicode::Dense1x2;

    // The lowest error correction holds the most data, e.g. the longest link of 2754 bytes.
    let code = qrcode::QrCode::with_error_correction_level(link.as_bytes(), qrcode::EcLevel::L)
        .map_err(|err| err.to_string())?;
    // On a dark background, the dark modules are the background and the light ones are drawn.
    Ok(code
        .render::<Dense1x2>()
        .dark_color(Dense1x2::Light)
        .light_color(Dense1x2::Dark)
        .quiet_zone(true)
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The code is embedded as URL-safe base64 without padding.
    #[test]
    fn link_with_code() {
        assert_eq!(
            link("Circle(r = 1mm);"),
            Ok("https://play.mucad.org/Q2lyY2xlKHIgPSAxbW0pOw".to_string())
        );
        // UTF-8 encoded, e.g. `µ`, with URL-safe characters only.
        let link = link("// µcad?>>").expect("A link");
        let data = link.strip_prefix(PLAY_URL).expect("The web viewer");
        assert!(
            data.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert_eq!(
            URL_SAFE_NO_PAD.decode(data).expect("Base64"),
            "// µcad?>>".as_bytes()
        );
    }

    /// Code with more than 2048 bytes is too long for a link.
    #[test]
    fn link_too_long() {
        assert!(link(&"a".repeat(MAX_CODE_BYTES)).is_ok());
        // `µ` has two bytes.
        let error = link(&"µ".repeat(MAX_CODE_BYTES / 2 + 1)).expect_err("Too long");
        assert!(error.contains("2050 bytes"), "{error}");
    }

    /// Even the longest link fits into a QR code.
    #[test]
    fn qr_code_of_longest_link() {
        let link = link(&"a".repeat(MAX_CODE_BYTES)).expect("A link");
        assert!(qr_code(&link).is_ok());
    }
}
