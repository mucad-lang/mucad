// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A markdown code block.

use mucad_lang_base::{HashMap, Identifier, Source, SourceKind, SourceLocation};

use crate::ParseError;

/// A code block header.
///
/// It has the following format: `mu,testname?param1=test&param2=foo&result`
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlockHeader {
    /// Code block language (e.g. `mu`)
    pub lang: Option<String>,
    /// Name of the code block, after comma.
    pub name: Option<Identifier>,
    /// Parameters
    pub query: HashMap<Identifier, Option<String>>,
}

/// A code block header with, e.g.: `µcad#ok(hires)`
impl CodeBlockHeader {
    /// Test banner to show test result and access logs.
    pub fn test_banner_string(name: &str) -> String {
        format!("[![test](.test/{name}.svg)](.test/{name}.log)")
    }

    pub(crate) fn is_test_banner(line: &str) -> bool {
        line.starts_with("[![test]")
    }

    /// Returns true if the info string of a fenced code block starts with a µcad language,
    /// e.g. `µcad,name#ok(hires)`.
    pub(crate) fn is_mucad(info: &str) -> bool {
        let language = info
            .split([',', '#', '('])
            .next()
            .unwrap_or_default()
            .trim();
        mucad_lang_base::MUCAD_EXTENSIONS.contains(&language)
    }
}
impl std::str::FromStr for CodeBlockHeader {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseError::MalformedHeader {
                input: s.to_string(),
            });
        }

        let mut lang = None;
        let mut fragment = None;
        let mut query = HashMap::default();

        // 1. Separate language from the fragment part at '#'
        let (lang_part, rest) = match s.split_once(',') {
            Some((l, r)) => (l, Some(r)),
            None => (s, None),
        };

        let lang_trimmed = lang_part.trim();
        if !lang_trimmed.is_empty() {
            lang = Some(lang_trimmed.to_string());
        }

        let Some(rest) = rest else {
            return Ok(CodeBlockHeader {
                lang,
                name: fragment,
                query,
            });
        };

        // 2. Separate test name from query parameters at '?' inside the fragment
        let (name_part, query_part) = match rest.split_once('?') {
            Some((n, q)) => (Some(n), Some(q)),
            None => (Some(rest), None),
        };

        if let Some(name_str) = name_part {
            let name_trimmed = name_str.trim();
            fragment = Some(Identifier::from_str(name_trimmed).map_err(|_| {
                ParseError::MalformedHeader {
                    input: s.to_string(),
                }
            })?);
        }

        let Some(query_str) = query_part else {
            return Ok(CodeBlockHeader {
                lang,
                name: fragment,
                query,
            });
        };

        // 3. Parse query-style parameters and flags separated by '&'
        for param_str in query_str.split('&') {
            let param_str = param_str.trim();
            if param_str.is_empty() {
                continue;
            }

            if let Some((key, val)) = param_str.split_once('=') {
                let id = Identifier::from_str(key.trim())
                    .map_err(|err| ParseError::InvalidParameter { param: err })?;
                let val = val.trim().to_string();
                query.insert(id, Some(val));
            } else {
                // Treat items without '=' as boolean flags (value = None)
                let id = Identifier::from_str(param_str)
                    .map_err(|err| ParseError::InvalidParameter { param: err })?;
                query.insert(id, None);
            }
        }

        Ok(CodeBlockHeader {
            lang,
            name: fragment,
            query,
        })
    }
}
impl std::fmt::Display for CodeBlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(lang) = &self.lang {
            write!(f, "{lang}")?;
        }
        if let Some(fragment) = &self.name {
            write!(f, ",{fragment}")?;
        }

        if !self.query.is_empty() {
            // Sort keys deterministically for clean output serialization
            let mut sorted_params: Vec<_> = self.query.iter().collect();
            sorted_params.sort_unstable_by_key(|(k, _)| *k);

            // Use a shifting separator to avoid manual index tracking
            let mut sep = '?';
            for (key, val) in sorted_params {
                match val {
                    Some(v) => write!(f, "{sep}{key}={v}")?,
                    None => write!(f, "{sep}{key}")?,
                }
                sep = '&';
            }
        }

        Ok(())
    }
}

/// A code block starting inside ```
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    /// The header of code block starting with ```
    pub header: CodeBlockHeader,
    /// The actual code.
    pub code: String,
    /// Line offset inside markdown file.
    pub line_offset: usize,
    /// Path of the markdown file, which contains this code block, if loaded from a file.
    pub path: Option<std::path::PathBuf>,
}

impl CodeBlock {
    /// Return the name of this code block.
    ///
    /// Must be unique within a markdown file.
    pub fn name(&self) -> &Option<Identifier> {
        &self.header.name
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns true if this code block can be formatted.
    ///
    /// A code block can be formatted if there is no `no_format` parameter given.
    pub fn can_format(&self) -> bool {
        !self
            .header
            .query
            .contains_key(&Identifier::from("no_format"))
    }

    /// The source of the code with its location in the markdown file.
    pub fn source(&self) -> Source {
        let location = match &self.path {
            Some(path) => SourceLocation::new(SourceKind::Path(path.clone()))
                .with_line_offset(self.line_offset as u32),
            None => SourceLocation::new(SourceKind::Str),
        };
        Source::new(location, self.code.clone())
    }
}

impl std::fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "```{}", self.header)?;
        if !self.code.is_empty() {
            writeln!(f, "{}", self.code)?;
        }
        write!(f, "```")
    }
}
