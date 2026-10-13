// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{GetSourceByHash, LineCol, LineIndex, Name, SpanToSrcRef, SrcRef, SrcReferrer};
use microcad_hash::{HashId, Hashed, ToHash, hash_id};
use serde::Serialize;

mod location;
mod map;

pub use location::{SourceKind, SourceLocation};

pub use map::SourceMap;

/// Unparsed source code with a location.
///
/// A [`Source`] is a textual input for the compiler.
//////
/// Additionally, a unique hash of the source code computed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Source {
    /// The source location
    pub location: SourceLocation,
    /// The original hashed code
    pub code: Hashed<String>,
    /// A line index to get lines from spans
    line_index: LineIndex,
}

impl SrcReferrer for Source {
    fn src_ref(&self) -> SrcRef {
        SrcRef::new(&(0..self.code().len()), LineCol::default(), self.hash_id())
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{loc}[{hash}]",
            loc = self.location,
            hash = self.hash_id()
        )
    }
}

/// A text edit, the result of comparing two sources
#[derive(Debug, PartialEq)]
pub struct TextEdit {
    /// SrcRef of a TextEdit
    pub src_ref: SrcRef,
    /// New text.
    pub new_text: String,
}

impl Source {
    /// Create a new source.
    pub fn new(location: impl Into<SourceLocation>, code: String) -> Self {
        let location = location.into();
        let line_index = LineIndex::new(&code, location.line_offset.unwrap_or_default());

        Self {
            location,
            code: Hashed::new(code),
            line_index,
        }
    }

    pub fn hash_id(&self) -> HashId {
        hash_id!(self.location.to_hash(), self.code.to_hash())
    }

    pub fn code(&self) -> &str {
        self.code.inner_ref()
    }

    pub fn set_code(&mut self, code: String) {
        self.code = Hashed::new(code);
    }

    pub fn path(&self) -> Option<std::path::PathBuf> {
        self.location.path()
    }

    /// Compare two sources and return a vector of TextEdits.
    pub fn compare(&self, other: &Self) -> Vec<TextEdit> {
        use dissimilar::Chunk;
        let chunks = dissimilar::diff(&self.code, &other.code);
        let mut edits = Vec::new();
        let source_hash = self.hash_id();

        // Track the current position in the *old_str*
        let mut current = LineCol::default();
        let mut byte_offset = 0;

        for chunk in chunks {
            match chunk {
                Chunk::Equal(text) => {
                    // Just move the cursor forward based on the matching text
                    for ch in text.chars() {
                        byte_offset += ch.len_utf8();

                        if ch == '\n' {
                            current.line += 1;
                            current.col = 0;
                        } else {
                            current.col += 1;
                        }
                    }
                }
                Chunk::Delete(text) => {
                    // Define the start position of the deletion
                    let start = current;
                    let start_byte = byte_offset;

                    // Calculate the end position by walking through the deleted text
                    for ch in text.chars() {
                        byte_offset += ch.len_utf8();

                        if ch == '\n' {
                            current.line += 1;
                            current.col = 0;
                        } else {
                            current.col += 1;
                        }
                    }

                    // A deletion replaces the range with an empty string
                    edits.push(TextEdit {
                        src_ref: SrcRef::new(&(start_byte..byte_offset), start, source_hash),
                        new_text: String::new(),
                    });
                }
                Chunk::Insert(text) => {
                    // An insertion happens at the *current* position without moving the cursor forward
                    // (since the inserted text doesn't exist in the original document)
                    edits.push(TextEdit {
                        src_ref: SrcRef::new(&(byte_offset..byte_offset), current, source_hash),
                        new_text: text.to_string(),
                    });
                }
            }
        }

        edits
    }

    /// Return file module name for this source.
    pub fn file_module_name(&self) -> Option<Name> {
        self.location.kind.file_module_name()
    }
}

#[cfg(feature = "io")]
impl Source {
    /// Load a Source from a file.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        let code = std::fs::read_to_string(&path)?;
        Ok(Self::new(SourceLocation::new(path), code))
    }

    /// Load a Source from a stdin.
    pub fn stdin() -> Result<Self, std::io::Error> {
        let mut full_buffer = String::new();
        use std::io::Read;
        // Read all contents from stdin until EOF is reached
        std::io::stdin().read_to_string(&mut full_buffer)?;

        Ok(Source::new(SourceKind::Stdin, full_buffer))
    }

    /// Save a source to file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        std::fs::write(path, self.code().as_bytes())
    }

    /// Sync a source.
    pub fn sync(&self) -> Result<(), std::io::Error> {
        match self.path() {
            Some(path) => self.save(path),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Cannot sync a source without a file path",
            )),
        }
    }
}

impl<'a> From<&'a str> for Source {
    fn from(s: &'a str) -> Self {
        Source::new(SourceKind::Str, s.to_string())
    }
}

impl SpanToSrcRef for &Source {
    fn span_to_src_ref(&self, span: &crate::Span) -> SrcRef {
        // Refer to the source by `hash_id`, which source caches use as key.
        SrcRef::new(
            span,
            self.line_index.line_col(self.code(), span.start),
            self.hash_id(),
        )
    }
}

impl GetSourceByHash for Source {
    fn get_source_by_hash(&self, hash: HashId) -> Option<&Self> {
        if hash == self.hash_id() {
            Some(self)
        } else {
            None
        }
    }
}

impl<'a> GetSourceByHash for &'a Source {
    fn get_source_by_hash(&self, hash: HashId) -> Option<&'a Source> {
        if hash == self.hash_id() {
            Some(self)
        } else {
            None
        }
    }
}
