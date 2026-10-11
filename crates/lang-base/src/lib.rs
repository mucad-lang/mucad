// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad language base components for error handling etc.

use miette::{MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};

pub use miette::Severity;

mod artifact;
mod diag;
pub mod display;
pub mod element;
mod fs;
mod id;
mod output;
mod shared;
mod source;
mod src_ref;
pub mod tree;
mod version;

pub use compact_str::{CompactString, ToCompactString};

/// URL to locate sources.
pub use url::Url;

pub use microcad_hash::{HashMap, HashSet, Hashed, Hasher, ToHash};

/// List of valid µcad extensions.
pub const MICROCAD_EXTENSIONS: &[&str] = &["mu", "µcad", "mcad", "ucad"];

/// Default extension for µcad files.
pub const MICROCAD_EXTENSION: &str = "mu";

pub use artifact::{Artifact, ArtifactError, ArtifactHeader, ArtifactKind, StageResult};
pub use diag::{
    DiagRenderOptions, Diagnostic, Diagnostics, Issue, IssueList, PushIssue, ToDiagnostics,
};
pub use display::DisplayOneLine;
pub use element::{Identifier, IdentifierList};
pub use fs::{FileSystem, VirtualFileSystem};
pub use id::{
    BuiltinId, BuiltinInfo, DefaultContext, DisplayWithCtx, DisplayWithCtxHelper, ExternalId,
    HashId, LibraryId, LibraryInfo, LookUpName, Name, SymbolId, hash_id,
};
pub use output::{Capture, Output, Stdout};
pub use shared::{Shared, SharedMut};
pub use source::{Source, SourceKind, SourceLocation, SourceMap, TextEdit};
pub use src_ref::{LineCol, LineIndex, Refer, Span, SpanToSrcRef, Spanned, SrcRef, SrcReferrer};
pub use tree::{TreeDisplay, TreeState};
pub use version::{LanguageVersion, MICROCAD_VERSION, Stability, Version, VersionAnnotation};

impl SourceCode for Source {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let inner_contents =
            self.code
                .read_span(span, context_lines_before, context_lines_after)?;
        let contents = MietteSpanContents::new_named(
            self.location.kind.display_name(),
            inner_contents.data(),
            *inner_contents.span(),
            inner_contents.line() + self.location.line_offset.unwrap_or_default() as usize,
            inner_contents.column(),
            inner_contents.line_count(),
        )
        .with_language("µcad");
        Ok(Box::new(contents))
    }
}

/// Trait that can fetch for a file by it's hash value.
pub trait GetSourceByHash {
    /// Get a source string by it's hash value.
    fn get_source_by_hash(&'_ self, hash: HashId) -> Option<&Source>;
}

/// Check if the element only includes one identifier
pub trait SingleIdentifier {
    /// If the element only includes one identifier, return it
    fn single_identifier(&self) -> Option<&Identifier>;

    /// Returns true if the element only includes a single identifier.
    fn is_single_identifier(&self) -> bool {
        self.single_identifier().is_some()
    }
}

/// Identifier accessor.
pub trait Identifiable {
    /// Get clone of the identifier.
    fn id(&self) -> Identifier {
        self.id_ref().clone()
    }

    /// Get reference to the identifier.
    fn id_ref(&self) -> &Identifier;

    /// Get identifier as string.
    fn id_as_str(&self) -> &str {
        self.id_ref().0.as_str()
    }
}

/// Result of a single compiler stage: the artifact bundled with the issues of that stage.
///
/// - `Ok`: The stage produced an artifact. The issues may contain errors the stage recovered from.
/// - `Err`: The stage produced no artifact. The issues contain at least one error.
pub type IssueResult<T, I> = Result<(T, IssueList<I>), IssueList<I>>;

/// Write the display output of type to file.
#[cfg(feature = "io")]
pub trait WriteToFile: std::fmt::Display {
    /// Writes the `Display` output to a file at the specified path.
    fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()>;
}

/// Default implementation for all types that implement display.
#[cfg(feature = "io")]
impl<T> WriteToFile for T
where
    T: std::fmt::Display,
{
    fn write_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        use std::io::Write;

        let path = path.as_ref();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        write!(writer, "{self}")?;
        writer.flush()
    }
}

/// Helper function construct a `Box<[T]>` for in iterator of `T`.
pub fn boxed<T>(iter: impl IntoIterator<Item = T>) -> Box<[T]> {
    iter.into_iter().collect::<Vec<_>>().into_boxed_slice()
}
