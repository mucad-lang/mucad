// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Parse context for parsing specific types.

use mucad_lang_base::{Source, Span, SpanToSrcRef, SrcRef};

/// Context for parsing.
#[derive(Debug)]
pub struct ParseContext<'source> {
    /// The source
    pub source: &'source Source,
}

impl<'source> From<&'source Source> for ParseContext<'source> {
    fn from(source: &'source Source) -> Self {
        Self { source }
    }
}

impl<'source> SpanToSrcRef for ParseContext<'source> {
    fn span_to_src_ref(&self, span: &Span) -> SrcRef {
        self.source.span_to_src_ref(span)
    }
}
