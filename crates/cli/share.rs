// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! QR codes of links to share µcad code, see [`mucad_driver::share`].

/// The QR code of `link` as text for a terminal with a dark background.
pub fn qr_code(link: &str) -> Result<String, String> {
    use qrcode::render::unicode::Dense1x2;

    // The lowest error correction holds the most data, e.g. the longest link of 2754 bytes.
    let code = qrcode::QrCode::new(link.as_bytes()).map_err(|err| err.to_string())?;
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
    use mucad_driver::share;

    /// Even the longest link fits into a QR code.
    #[test]
    fn qr_code_of_longest_link() {
        let link = share::link(&"a".repeat(share::MAX_CODE_BYTES)).expect("A link");
        assert!(qr_code(&link).is_ok());
    }
}
