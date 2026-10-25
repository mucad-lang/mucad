// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Links to share µcad code in the web viewer, which contain the code.

use base64::{
    Engine,
    alphabet::URL_SAFE,
    engine::{
        DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig, general_purpose::URL_SAFE_NO_PAD,
    },
};

/// The web viewer, which shows the code of a link.
pub const PLAY_URL: &str = "https://play.mucad.org/";

/// The maximum size of the code in a link, in bytes of its UTF-8 encoding.
pub const MAX_CODE_BYTES: usize = 2048;

/// An error of sharing code.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ShareError {
    /// The code is too long for a link.
    #[error("the code has {0} bytes, a link can contain at most {MAX_CODE_BYTES} bytes")]
    TooLong(usize),
    /// The link contains no valid base64.
    #[error("the link contains no valid code: {0}")]
    InvalidBase64(String),
    /// The code of the link is no valid UTF-8.
    #[error("the code of the link is no valid UTF-8 text")]
    InvalidUtf8,
}

/// The link to the web viewer with `code`, which is embedded as URL-safe base64,
/// e.g. `https://play.mucad.org/Q2lyY2xlKHIgPSAxbW0pOw` for `Circle(r = 1mm);`.
///
/// Fails, if the code has more than [`MAX_CODE_BYTES`] bytes.
pub fn link(code: &str) -> Result<String, ShareError> {
    if code.len() > MAX_CODE_BYTES {
        return Err(ShareError::TooLong(code.len()));
    }
    Ok(format!("{PLAY_URL}{}", URL_SAFE_NO_PAD.encode(code)))
}

/// The code of a link, see [`link`].
///
/// The link is a complete link with the code in its path, after `#` or as `?code=`,
/// or only the link data, e.g. `Q2lyY2xlKHIgPSAxbW0pOw`. Padding with `=` is optional.
/// Fails, if the code has more than [`MAX_CODE_BYTES`] bytes.
pub fn decode(link: &str) -> Result<String, ShareError> {
    let link = link.trim();
    let data = if let Some((_, data)) = link.split_once('#') {
        data
    } else if let Some((_, query)) = link.split_once('?') {
        query
            .split('&')
            .find_map(|parameter| parameter.strip_prefix("code="))
            .unwrap_or(query)
    } else {
        link.strip_prefix(PLAY_URL).unwrap_or(link)
    };

    // A longer link contains more code than allowed, which is not decoded.
    if data.len() > MAX_CODE_BYTES.div_ceil(3) * 4 {
        return Err(ShareError::TooLong(data.len() / 4 * 3));
    }
    let engine = GeneralPurpose::new(
        &URL_SAFE,
        GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
    );
    let bytes = engine
        .decode(data)
        .map_err(|err| ShareError::InvalidBase64(err.to_string()))?;
    if bytes.len() > MAX_CODE_BYTES {
        return Err(ShareError::TooLong(bytes.len()));
    }
    String::from_utf8(bytes).map_err(|_| ShareError::InvalidUtf8)
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
    }

    /// Code with more than 2048 bytes is too long for a link.
    #[test]
    fn link_too_long() {
        assert!(link(&"a".repeat(MAX_CODE_BYTES)).is_ok());
        // `µ` has two bytes.
        assert_eq!(
            link(&"µ".repeat(MAX_CODE_BYTES / 2 + 1)),
            Err(ShareError::TooLong(2050))
        );
    }

    /// The code of a link is decoded, also from the fragment or query of a link.
    #[test]
    fn decode_link() {
        let code = "// µcad?>>\nCircle(r = 1mm);\n";
        let link = link(code).expect("A link");
        let data = link.strip_prefix(PLAY_URL).expect("The web viewer");

        for link in [
            link.clone(),
            data.to_string(),
            format!("https://play.mucad.org/#{data}"),
            format!("#{data}"),
            format!("https://play.mucad.org/?code={data}"),
            format!("?example=1&code={data}"),
            // With padding to a multiple of four characters.
            format!("{data}{}", "=".repeat((4 - data.len() % 4) % 4)),
        ] {
            assert_eq!(decode(&link).as_deref(), Ok(code), "{link}");
        }
    }

    /// Invalid links fail.
    #[test]
    fn decode_invalid() {
        assert!(matches!(
            decode("#a+b/c"),
            Err(ShareError::InvalidBase64(_))
        ));
        // `/w` is the byte 0xff, which is no UTF-8.
        assert_eq!(decode("_w"), Err(ShareError::InvalidUtf8));
        let too_long = URL_SAFE_NO_PAD.encode("a".repeat(MAX_CODE_BYTES + 3));
        assert!(matches!(decode(&too_long), Err(ShareError::TooLong(_))));
        // The longest code can be decoded.
        let longest = link(&"a".repeat(MAX_CODE_BYTES)).expect("A link");
        assert_eq!(decode(&longest).map(|code| code.len()), Ok(MAX_CODE_BYTES));
    }
}
