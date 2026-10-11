// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A markdown code block.

use microcad_lang_base::{Source, SourceLocation, Url};

/// A code block header.
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlockHeader {
    /// Name of the code block.
    pub name: Option<String>,
    /// An optional fragment, e.g. to be used for test results: `#fragment`
    pub fragment: Option<String>,
    /// Parameters of the code block inside `()`
    pub parameters: Vec<String>,
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
    pub(crate) fn is_microcad(info: &str) -> bool {
        let language = info
            .split([',', '#', '('])
            .next()
            .unwrap_or_default()
            .trim();
        microcad_lang_base::MICROCAD_EXTENSIONS.contains(&language)
    }
}

/// Parse the info string of a fenced code block, e.g. `µcad,name#fragment(param1,param2)`.
impl std::str::FromStr for CodeBlockHeader {
    type Err = crate::ParseError;

    fn from_str(info: &str) -> Result<Self, Self::Err> {
        let info = info.trim();

        // 1. Locate structural delimiters
        let hash_pos = info.find('#');
        let paren_pos = info.find('(');

        // 2. Parse name (`µcad,my_name`)
        let name_end = hash_pos.or(paren_pos).unwrap_or(info.len());
        let name_part = info[..name_end].trim();
        let name = name_part
            .find(',')
            .map(|comma_idx| name_part[comma_idx + 1..].trim().to_string());

        // 3. Parse fragment (`#ok`, `#fail`, etc.)
        let fragment = hash_pos.map(|start| {
            let end = paren_pos.unwrap_or(info.len());
            info[start + 1..end].trim().to_string()
        });

        // 4. Parse parameters (`(hires, lowres)`)
        let parameters = match paren_pos {
            Some(start) => {
                let end = info.find(')').ok_or(crate::ParseError::MalformedHeader)?;
                info[start + 1..end]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }
            None => Vec::new(),
        };

        Ok(Self {
            name,
            fragment,
            parameters,
        })
    }
}

impl std::fmt::Display for CodeBlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            writeln!(f, "{}\n", Self::test_banner_string(name))?
        }

        write!(f, "```{ext}", ext = microcad_lang_base::MICROCAD_EXTENSION)?;
        if let Some(name) = &self.name {
            write!(f, ",{name}")?
        }

        match &self.fragment {
            None => {}
            Some(fragment) => {
                write!(f, "#{fragment}")?;
            }
        };
        if !self.parameters.is_empty() {
            write!(f, "({})", self.parameters.join(","))?;
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
}

impl CodeBlock {
    /// Return the name of this code block.
    ///
    /// Must be unique within a markdown file.
    pub fn name(&self) -> &Option<String> {
        &self.header.name
    }

    /// Return test result.
    pub fn fragment(&self) -> &Option<String> {
        &self.header.fragment
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns true if this code block can be formatted.
    ///
    /// A code block can be formatted if there is no `no_format` parameter given.
    pub fn can_format(&self) -> bool {
        !self.header.parameters.contains(&String::from("no_format"))
    }

    /// Get source for a code block
    pub fn source(&self, mut url: Url) -> Source {
        url.set_fragment(self.header.name.as_deref());
        Source::new(
            SourceLocation::new(url).with_line_offset(self.line_offset as u32),
            self.code.clone(),
        )
    }
}

impl std::fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.header)?;
        if !self.code.is_empty() {
            writeln!(f, "{}", self.code)?;
        }
        write!(f, "```")
    }
}
