// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Diagnostics, SourceLocation, WriteToFile};
use miette::IntoDiagnostic;

use crate::prelude as mu;
use mucad_lang_markdown::Markdown;

pub struct MarkdownDocument {
    location: SourceLocation,
    markdown: Markdown,
    diagnostics: Diagnostics,
}

impl MarkdownDocument {
    pub fn new(location: impl Into<SourceLocation>) -> mu::Result<Self> {
        let location = location.into();
        Ok(Self {
            markdown: Markdown::load(
                location
                    .path()
                    .ok_or(miette::miette!("Markdown has no file path"))?,
            )
            .into_diagnostic()?,
            location,
            diagnostics: Default::default(),
        })
    }
}

impl mu::commands::Format for mu::document::Markdown {
    fn format(&mut self, params: &mu::commands::FormatParameters) -> mu::Result<bool> {
        let mut formatted = false;
        let config = params;
        self.diagnostics.clear();

        self.markdown
            .code_blocks_mut()
            .filter(|code_block| code_block.can_format())
            .for_each(|code_block| {
                match mucad_lang_format::format_code(&code_block.code, config) {
                    Ok(code) => {
                        formatted |= code_block.code != code;
                        code_block.code = code;
                    }
                    Err(diag) => {
                        self.diagnostics.append(diag);
                    }
                }
            });

        if self.diagnostics.has_errors() {
            Err(miette::miette!("Error formatting markdown"))
        } else {
            Ok(formatted)
        }
    }
}

impl mu::commands::Sync for mu::document::Markdown {
    fn sync(&self) -> mu::Result {
        self.markdown
            .write_to_file(self.location.path().expect("Location must be a path"))
            .into_diagnostic()
    }
}
