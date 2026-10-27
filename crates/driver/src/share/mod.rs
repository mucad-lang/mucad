// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Links to share µcad code in the web viewer, which contain the code.
//!
//! A link contains URL-safe base64 of the [magic](MAGIC) `µ`, the [version](VERSION) of the
//! encoding as one byte and the text compressed with
//! [Brotli](https://www.rfc-editor.org/rfc/rfc7932).
//!
//! The text starts with an optional preamble in TOML between lines `+++`, followed by the µcad
//! code, which is formatted when it is decoded:
//!
//! ```text
//! +++
//! name = "Test"
//! +++
//! Circle(r = 1mm);
//! ```

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

/// The magic at the start of the encoding: `µ` in UTF-8, the bytes `0xc2 0xb5`.
const MAGIC: &[u8] = "µ".as_bytes();

/// The version of the encoding, the byte after the magic.
const VERSION: u8 = 1;

/// The magic and the version in front of the compressed text.
const HEADER: [u8; 3] = [MAGIC[0], MAGIC[1], VERSION];

/// The line before and after the preamble.
const PREAMBLE_DELIMITER: &str = "+++";

/// The maximum size of the encoding: Brotli adds a few bytes to text, which it cannot
/// compress.
const MAX_COMPRESSED_BYTES: usize = HEADER.len() + MAX_CODE_BYTES + 64;

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
    /// The preamble is no valid TOML, or cannot be written between its delimiters.
    #[error("the preamble is invalid: {0}")]
    InvalidPreamble(String),
}

/// Shared code with its preamble.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SharedCode {
    /// The preamble, e.g. `name = "Test"`, which is empty without a preamble.
    pub preamble: toml::Table,
    /// The µcad code after the preamble.
    pub code: String,
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

/// Whether `line` is the delimiter of the preamble, also with `\r\n` at its end.
fn is_delimiter(line: &str) -> bool {
    line.trim_end() == PREAMBLE_DELIMITER
}

/// The text to encode: the preamble between delimiters, if there is one, and the code.
///
/// Code, whose first line is a delimiter, gets an empty preamble, so that it is no preamble.
/// Fails, if a line of the preamble in TOML is a delimiter, e.g. in a multi-line string.
fn shared_text(preamble: &toml::Table, code: &str) -> Result<String, ShareError> {
    if preamble.is_empty() && !code.lines().next().is_some_and(is_delimiter) {
        return Ok(code.to_string());
    }
    let toml =
        toml::to_string(preamble).map_err(|err| ShareError::InvalidPreamble(err.to_string()))?;
    if toml.lines().any(is_delimiter) {
        return Err(ShareError::InvalidPreamble(format!(
            "a line of it is {PREAMBLE_DELIMITER}"
        )));
    }
    Ok(format!(
        "{PREAMBLE_DELIMITER}\n{toml}{PREAMBLE_DELIMITER}\n{code}"
    ))
}

/// The preamble and the code of decoded text, see [`shared_text`].
///
/// Fails, if the preamble is not closed or no valid TOML.
fn split_preamble(text: &str) -> Result<SharedCode, ShareError> {
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next().filter(|line| is_delimiter(line)) else {
        return Ok(SharedCode {
            preamble: toml::Table::new(),
            code: text.to_string(),
        });
    };
    let start = first.len();
    let mut end = start;
    for line in lines {
        if is_delimiter(line) {
            let preamble = text[start..end]
                .parse()
                .map_err(|err: toml::de::Error| ShareError::InvalidPreamble(err.to_string()))?;
            return Ok(SharedCode {
                preamble,
                code: text[end + line.len()..].to_string(),
            });
        }
        end += line.len();
    }
    Err(ShareError::InvalidData(format!(
        "the preamble is not closed by {PREAMBLE_DELIMITER}"
    )))
}

/// Compress `bytes` with Brotli behind the header.
fn compress(bytes: &[u8]) -> Vec<u8> {
    let mut writer =
        brotli::CompressorWriter::new(HEADER.to_vec(), BUFFER_SIZE, QUALITY, WINDOW_BITS);
    // Writing into a vector cannot fail.
    writer.write_all(bytes).expect("Compressed code");
    writer.into_inner()
}

/// Decompress text, see [`compress`].
///
/// Fails, if the bytes do not start with the magic and the version, are no Brotli data, the
/// text is no UTF-8 or longer than allowed.
fn decompress(bytes: &[u8]) -> Result<String, ShareError> {
    let bytes = bytes
        .strip_prefix(MAGIC)
        .ok_or_else(|| ShareError::InvalidData("no µcad code, the magic µ is missing".into()))?;
    let bytes = match bytes.split_first() {
        Some((&VERSION, bytes)) => bytes,
        Some((version, _)) => {
            return Err(ShareError::InvalidData(format!(
                "the version {version} of the encoding is not supported, only {VERSION}"
            )));
        }
        None => return Err(ShareError::InvalidData("the version is missing".into())),
    };
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

/// The link to the web viewer with `code` without preamble, see [`link_with_preamble`],
/// e.g. `https://play.mucad.org/#wrUBQTwABENpcmNsZShyID0gMW1tKTsD` for `Circle(r = 1mm);`.
pub fn link(code: &str, strip: bool) -> Result<String, ShareError> {
    link_with_preamble(code, &toml::Table::new(), strip)
}

/// The link to the web viewer with `code` behind the `preamble`, if it is not empty, which are
/// compressed and embedded as URL-safe base64, see the [module](self).
///
/// With `strip`, the link contains the code without comments. Fails, if the preamble and the
/// code have more than [`MAX_CODE_BYTES`] bytes, without comments with `strip`, or if the
/// preamble cannot be written, see [`ShareError::InvalidPreamble`].
pub fn link_with_preamble(
    code: &str,
    preamble: &toml::Table,
    strip: bool,
) -> Result<String, ShareError> {
    let text = shared_text(preamble, &shared_code(code, strip))?;
    if text.len() > MAX_CODE_BYTES {
        return Err(ShareError::TooLong(text.len()));
    }
    Ok(format!(
        "{PLAY_URL}{}",
        URL_SAFE_NO_PAD.encode(compress(text.as_bytes()))
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

/// The formatted code of a link without its preamble, see [`decode_with_preamble`].
pub fn decode(link: &str) -> Result<String, ShareError> {
    Ok(decode_with_preamble(link)?.code)
}

/// The preamble and the formatted code of a link, see [`link_with_preamble`].
///
/// The link is a complete link with the code after `#`, as `?code=` or in its path, or only
/// the link data, e.g. `wrUBQTwABENpcmNsZShyID0gMW1tKTsD`. Padding with `=` is optional.
///
/// The code is parsed and formatted, code with errors is returned unformatted.
pub fn decode_with_preamble(link: &str) -> Result<SharedCode, ShareError> {
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
    let mut shared = split_preamble(&decompress(&bytes)?)?;
    let source = mu::Source::from(shared.code.as_str());
    if let Ok((ast, _)) = mu::parse::parse(&source) {
        shared.code = mu::format::format_ast(&ast, &mu::FormatConfig::default());
    }
    Ok(shared)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The link of compressed `text`, e.g. with a preamble, which [`link`] does not write.
    fn link_of_text(text: &str) -> String {
        format!(
            "{PLAY_URL}{}",
            URL_SAFE_NO_PAD.encode(compress(text.as_bytes()))
        )
    }

    /// The compressed code starts with the magic `µ` and the version.
    #[test]
    fn header() {
        assert_eq!(HEADER, [0xc2, 0xb5, 1]);
        assert!(compress(b"x = 1;").starts_with(&HEADER));
        let link = link("x = 1;", false).expect("A link");
        let data = link.strip_prefix(PLAY_URL).expect("The web viewer");
        let bytes = URL_SAFE_NO_PAD.decode(data).expect("Base64");
        assert!(bytes.starts_with(&HEADER));
    }

    /// Another version of the encoding fails, also without a version.
    #[test]
    fn version() {
        let mut bytes = compress(b"x = 1;");
        bytes[MAGIC.len()] = VERSION + 1;
        let message = |bytes: &[u8]| match decompress(bytes) {
            Err(ShareError::InvalidData(message)) => message,
            result => panic!("Invalid data: {result:?}"),
        };
        assert_eq!(
            message(&bytes),
            "the version 2 of the encoding is not supported, only 1"
        );
        assert_eq!(message(MAGIC), "the version is missing");
    }

    /// The preamble is written between delimiters in front of the code, and read again.
    #[test]
    fn preamble() {
        let preamble: toml::Table = "name = \"Test\"".parse().expect("TOML");
        assert_eq!(
            shared_text(&preamble, "x = 1;").as_deref(),
            Ok("+++\nname = \"Test\"\n+++\nx = 1;")
        );
        let link = link_with_preamble("// c\nx=1;\n", &preamble, true).expect("A link");
        assert_eq!(
            decode_with_preamble(&link),
            Ok(SharedCode {
                preamble: preamble.clone(),
                code: "x = 1;\n".into()
            })
        );
        assert_eq!(decode(&link).as_deref(), Ok("x = 1;\n"));

        // Without preamble, nothing is written in front of the code.
        assert_eq!(
            shared_text(&toml::Table::new(), "x = 1;").as_deref(),
            Ok("x = 1;")
        );
        let shared = decode_with_preamble(&super::link("x = 1;", false).expect("A link"));
        assert_eq!(shared.map(|shared| shared.preamble), Ok(toml::Table::new()));

        // Code, which starts with a delimiter, gets an empty preamble.
        let text = shared_text(&toml::Table::new(), "+++\nx").expect("A text");
        assert_eq!(text, "+++\n+++\n+++\nx");
        assert_eq!(
            split_preamble(&text).map(|shared| shared.code).as_deref(),
            Ok("+++\nx")
        );

        // Lines may end with `\r\n`.
        let shared = split_preamble("+++\r\nname = \"Test\"\r\n+++\r\nx = 1;").expect("Shared");
        assert_eq!(shared.preamble, preamble);
        assert_eq!(shared.code, "x = 1;");
    }

    /// A preamble, which is not closed or no TOML, fails, also one with a line of a delimiter.
    #[test]
    fn invalid_preamble() {
        assert!(matches!(
            decode(&link_of_text("+++\nname = \"Test\"\nx = 1;")),
            Err(ShareError::InvalidData(_))
        ));
        assert!(matches!(
            decode(&link_of_text("+++\nname = \n+++\nx = 1;")),
            Err(ShareError::InvalidPreamble(_))
        ));
        // A string with a delimiter line is written escaped, or the preamble fails.
        let preamble: toml::Table = "text = \"a\\n+++\\nb\"".parse().expect("TOML");
        match link_with_preamble("x = 1;", &preamble, false) {
            Ok(link) => assert_eq!(
                decode_with_preamble(&link).map(|shared| shared.preamble),
                Ok(preamble)
            ),
            Err(err) => assert!(matches!(err, ShareError::InvalidPreamble(_)), "{err}"),
        }
    }

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
            Ok("https://play.mucad.org/#wrUBQTwABENpcmNsZShyID0gMW1tKTsD".to_string())
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
        // No magic, e.g. Brotli data without it.
        let data = URL_SAFE_NO_PAD.encode("Circle(r = 1mm);");
        assert!(matches!(decode(&data), Err(ShareError::InvalidData(_))));
        let bytes = compress(b"x = 1;");
        assert!(matches!(
            decompress(&bytes[MAGIC.len()..]),
            Err(ShareError::InvalidData(_))
        ));
        assert!(matches!(decompress(b""), Err(ShareError::InvalidData(_))));
        assert!(matches!(
            decompress(&MAGIC[..1]),
            Err(ShareError::InvalidData(_))
        ));
        // The header without Brotli data.
        assert!(matches!(
            decompress(&HEADER),
            Err(ShareError::InvalidData(_))
        ));
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
