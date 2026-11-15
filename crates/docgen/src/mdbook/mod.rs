// Copyright © 2026 The µcad authors <info@ucad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Markdown book generators for built-in and compiled libraries.

mod builtin;
mod library;

use mucad_lang_base::{Stability, VersionAnnotation};
use mucad_lang_markdown::{Block, BlockQuote, Markdown};

/// Trait to fetch markdown from a symbol definition.
pub(crate) trait ToMd {
    fn to_md(&self) -> Markdown;
}

/// The stability of a symbol above its documentation, like in the Rust docs, `None` for a
/// stable symbol without a version, in which it was introduced:
///
/// ```md
/// > 👎 Deprecated since 0.5.0, will be removed in 0.7.0: Use `g()` instead
/// ```
///
/// ```md
/// > 🔬 This is an experimental API, which may change or be removed in any version.
/// >
/// > Introduced in 0.6.0
/// ```
pub(crate) fn stability_block(ver: &VersionAnnotation) -> Option<Block> {
    let stability = match &ver.stability {
        Stability::Stable => None,
        Stability::Experimental => Some(
            "🔬 This is an experimental API, which may change or be removed in any version."
                .to_string(),
        ),
        deprecated @ Stability::Deprecated { .. } => {
            let deprecation = deprecated.deprecation().unwrap_or_default();
            Some(match deprecated.deprecation_note() {
                Some(note) => format!("👎 {deprecation}: {note}"),
                None => format!("👎 {deprecation}"),
            })
        }
    };
    let introduced = ver
        .introduced
        .as_ref()
        .map(|version| format!("Introduced in {version}"));

    let paragraphs: Vec<_> = stability.into_iter().chain(introduced).collect();
    if paragraphs.is_empty() {
        return None;
    }
    // The paragraphs of the quote are separated by an empty quote line.
    let lines = paragraphs
        .iter()
        .enumerate()
        .flat_map(|(index, paragraph)| {
            (index > 0)
                .then(|| ">".to_string())
                .into_iter()
                .chain(std::iter::once(format!("> {paragraph}")))
        });
    Some(Block::BlockQuote(lines.collect::<BlockQuote>()))
}

/// The stability of a symbol behind its name in a list, like in the Rust docs, e.g.
/// `👎 *Deprecated*`, `None` for a stable symbol.
pub(crate) fn stability_tag(ver: &VersionAnnotation) -> Option<&'static str> {
    match ver.stability {
        Stability::Stable => None,
        Stability::Experimental => Some("🔬 *Experimental*"),
        Stability::Deprecated { .. } => Some("👎 *Deprecated*"),
    }
}

/// Documentation text as written, if there is any.
pub(crate) fn doc_block(doc: Option<&str>) -> Option<Block> {
    doc.map(str::trim)
        .filter(|doc| !doc.is_empty())
        .map(|doc| Block::Paragraph(doc.to_string()))
}
