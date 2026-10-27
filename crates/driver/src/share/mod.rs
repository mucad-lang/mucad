// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Links to share µcad code in the web viewer, which contain the code.
//!
//! The code is compressed with [Brotli](https://www.rfc-editor.org/rfc/rfc7932) and embedded
//! as URL-safe base64. The code of a link is formatted, when it is decoded.

use std::io::{Read, Write};

use base64::{
    Engine,
    alphabet::URL_SAFE,
    engine::{
        DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig, general_purpose::URL_SAFE_NO_PAD,
    },
};

use crate::prelude as mu;

use mu::parse::token::Token;

/// The web viewer, which shows the code of a link in its fragment.
///
/// The fragment is not sent to the server, so the web viewer can be a static page, and the code
/// is not logged by the server.
pub const PLAY_URL: &str = "https://play.mucad.org/#";

/// The maximum size of the code in a link, in bytes of its UTF-8 encoding.
pub const MAX_CODE_BYTES: usize = 2048;

/// The maximum size of compressed code: Brotli adds a few bytes to code, which it cannot
/// compress.
const MAX_COMPRESSED_BYTES: usize = MAX_CODE_BYTES + 64;

/// The Brotli quality, the best compression.
const QUALITY: u32 = 11;

/// The Brotli window of 2^12 bytes, the smallest one, which covers code of the maximum size.
const WINDOW_BITS: u32 = 12;

/// The size of the buffers of the Brotli compressor and decompressor.
const BUFFER_SIZE: usize = 4096;

/// An error of sharing code.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ShareError {
    /// The code is too long for a link.
    #[error("the code has {0} bytes, a link can contain at most {MAX_CODE_BYTES} bytes")]
    TooLong(usize),
    /// The link contains no valid base64.
    #[error("the link contains no valid code: {0}")]
    InvalidBase64(String),
    /// The link contains no valid compressed code.
    #[error("the link contains no valid code: {0}")]
    InvalidData(String),
}

/// The code to share, with `strip` without comments and whitespace at the start.
///
/// A comment is replaced by a space, so the tokens before and after it stay separated, the
/// code is formatted when it is decoded.
fn shared_code(code: &str, strip: bool) -> String {
    if !strip {
        return code.to_string();
    }
    let mut shared = String::with_capacity(code.len());
    let mut end = 0;
    for token in mu::parse::lex(code) {
        if matches!(
            token.value,
            Token::SingleLineComment(_)
                | Token::MultiLineComment(_)
                | Token::DocComment(_)
                | Token::InnerDocComment(_)
        ) {
            shared.push_str(&code[end..token.span.start]);
            shared.push(' ');
            end = token.span.end;
        }
    }
    shared.push_str(&code[end..]);
    shared.trim_start().to_string()
}

/// Compress `bytes` with Brotli.
fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut writer = brotli::CompressorWriter::new(Vec::new(), BUFFER_SIZE, QUALITY, WINDOW_BITS);
    // Writing into a vector cannot fail.
    writer.write_all(bytes).expect("Compressed code");
    writer.into_inner()
}

/// Decompress code, see [`compress`].
///
/// Fails, if the bytes are no Brotli data, the code is no UTF-8 or longer than allowed.
fn decompress(bytes: &[u8]) -> Result<String, ShareError> {
    let mut code = Vec::new();
    // At most one byte more than allowed, so that a longer code is not decompressed completely.
    brotli::Decompressor::new(bytes, BUFFER_SIZE)
        .take(MAX_CODE_BYTES as u64 + 1)
        .read_to_end(&mut code)
        .map_err(|err| ShareError::InvalidData(err.to_string()))?;
    if code.len() > MAX_CODE_BYTES {
        return Err(ShareError::TooLong(code.len()));
    }
    String::from_utf8(code).map_err(|err| ShareError::InvalidData(err.to_string()))
}

/// The link to the web viewer with `code`, which is compressed and embedded as URL-safe base64,
/// e.g. `https://play.mucad.org/#QTwABENpcmNsZShyID0gMW1tKTsD` for `Circle(r = 1mm);`.
///
/// With `strip`, the link contains the code without comments. Fails, if the code has more than
/// [`MAX_CODE_BYTES`] bytes, without comments with `strip`.
pub fn link(code: &str, strip: bool) -> Result<String, ShareError> {
    let code = shared_code(code, strip);
    if code.len() > MAX_CODE_BYTES {
        return Err(ShareError::TooLong(code.len()));
    }
    Ok(format!(
        "{PLAY_URL}{}",
        URL_SAFE_NO_PAD.encode(compress(code.as_bytes()))
    ))
}

/// The link data of a link: after `#`, as `?code=`, in the path or the link is the data.
fn link_data(link: &str) -> &str {
    let link = link.trim();
    if let Some((_, data)) = link.split_once('#') {
        data
    } else if let Some((_, query)) = link.split_once('?') {
        query
            .split('&')
            .find_map(|parameter| parameter.strip_prefix("code="))
            .unwrap_or(query)
    } else {
        // Former links have the code in their path.
        link.strip_prefix(PLAY_URL.trim_end_matches('#'))
            .unwrap_or(link)
    }
}

/// The formatted code of a link, see [`link`].
///
/// The link is a complete link with the code after `#`, as `?code=` or in its path, or only
/// the link data, e.g. `QTwABENpcmNsZShyID0gMW1tKTsD`. Padding with `=` is optional.
///
/// The code is parsed and formatted, code with errors is returned unformatted.
pub fn decode(link: &str) -> Result<String, ShareError> {
    let data = link_data(link);
    // A longer link contains more code than allowed, which is not decoded.
    if data.len() > MAX_COMPRESSED_BYTES.div_ceil(3) * 4 {
        return Err(ShareError::TooLong(data.len() / 4 * 3));
    }
    let engine = GeneralPurpose::new(
        &URL_SAFE,
        GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
    );
    let bytes = engine
        .decode(data)
        .map_err(|err| ShareError::InvalidBase64(err.to_string()))?;
    let code = decompress(&bytes)?;
    let source = mu::Source::from(code.as_str());
    Ok(match mu::parse::parse(&source) {
        Ok((ast, _)) => mu::format::format_ast(&ast, &mu::FormatConfig::default()),
        Err(_) => code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Code is compressed and decompressed unchanged, also unknown characters and lexer errors.
    #[test]
    fn compress_code() {
        for code in [
            "",
            "// µcad\nsketch S(r: Length) {\n    Circle(r);\n}\n",
            "s = \"a {x:05.2} b {{ }} \\n\";",
            "x = 1 ~ 2 $ ☃;",
            "a = \"unclosed",
        ] {
            assert_eq!(decompress(&compress(code.as_bytes())).as_deref(), Ok(code));
        }
    }

    /// The code of every µcad file of the examples and the standard library can be shared.
    #[test]
    fn compress_files() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for dir in ["examples", "crates/std/lib/std"] {
            for entry in std::fs::read_dir(root.join(dir)).expect("A directory") {
                let path = entry.expect("An entry").path();
                if path.extension().is_some_and(|ext| ext == "mu") {
                    let code = std::fs::read_to_string(&path).expect("A file");
                    if code.len() <= MAX_CODE_BYTES {
                        let compressed = compress(code.as_bytes());
                        assert_eq!(decompress(&compressed).as_deref(), Ok(code.as_str()));
                    }
                }
            }
        }
    }

    /// With `strip`, comments are removed, also before the length of the code is checked.
    #[test]
    fn strip_comments() {
        assert_eq!(
            shared_code("// c\nx = 1/* y */;\n/// d\n", true),
            "x = 1 ;\n \n"
        );
        // The format of a format string is kept.
        assert_eq!(
            shared_code("s = \"{x:05.2}\"; // c\n", true),
            "s = \"{x:05.2}\";  \n"
        );

        let code = "// c\nx = 1/* y */;\n";
        let stripped = link(code, true).expect("A link");
        assert!(stripped.len() < link(code, false).expect("A link").len());
        assert_eq!(decode(&stripped).as_deref(), Ok("x = 1;\n"));

        let long = format!("// {}\nCircle(r = 10mm);\n", "x".repeat(MAX_CODE_BYTES));
        assert!(matches!(link(&long, false), Err(ShareError::TooLong(2070))));
        assert!(link(&long, true).is_ok());
    }

    /// Code with more than 2048 bytes is too long for a link.
    #[test]
    fn link_too_long() {
        assert!(link(&"a".repeat(MAX_CODE_BYTES), false).is_ok());
        // `µ` has two bytes.
        assert_eq!(
            link(&"µ".repeat(MAX_CODE_BYTES / 2 + 1), false),
            Err(ShareError::TooLong(2050))
        );
    }

    /// The code of a link is decoded formatted, also from the fragment or query of a link.
    #[test]
    fn decode_link() {
        assert_eq!(
            link("Circle(r = 1mm);", false),
            Ok("https://play.mucad.org/#QTwABENpcmNsZShyID0gMW1tKTsD".to_string())
        );
        let link = link("// µcad?>>\nCircle( r=1mm );\n", false).expect("A link");
        let data = link.strip_prefix(PLAY_URL).expect("The web viewer");
        assert!(
            data.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );

        let formatted = "// µcad?>>\nCircle(r = 1mm);\n";
        for link in [
            link.clone(),
            data.to_string(),
            format!("https://play.mucad.org/{data}"),
            format!("#{data}"),
            format!("https://play.mucad.org/?code={data}"),
            format!("?example=1&code={data}"),
            // With padding to a multiple of four characters.
            format!("{data}{}", "=".repeat((4 - data.len() % 4) % 4)),
        ] {
            assert_eq!(decode(&link).as_deref(), Ok(formatted), "{link}");
        }

        // Code with errors is not formatted.
        let link = super::link("Circle( r= ;", false).expect("A link");
        assert_eq!(decode(&link).as_deref(), Ok("Circle( r= ;"));
    }

    /// Invalid links fail.
    #[test]
    fn decode_invalid() {
        assert!(matches!(
            decode("#a+b/c"),
            Err(ShareError::InvalidBase64(_))
        ));
        // No Brotli data.
        let data = URL_SAFE_NO_PAD.encode("Circle(r = 1mm);");
        assert!(matches!(decode(&data), Err(ShareError::InvalidData(_))));
        // Truncated data.
        let bytes = compress(b"x = 1;");
        assert!(decompress(&bytes[..bytes.len() - 1]).is_err());
        // No UTF-8.
        assert!(matches!(
            decompress(&compress(&[0xff, 0xfe])),
            Err(ShareError::InvalidData(_))
        ));
        // Too long, also when it is compressed small.
        let data = URL_SAFE_NO_PAD.encode(vec![0; MAX_COMPRESSED_BYTES + 3]);
        assert!(matches!(decode(&data), Err(ShareError::TooLong(_))));
        let bytes = compress("x".repeat(MAX_CODE_BYTES + 1).as_bytes());
        assert_eq!(
            decompress(&bytes),
            Err(ShareError::TooLong(MAX_CODE_BYTES + 1))
        );
    }
}
