// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{Diagnostics, SourceLocation, Url};
use mucad_lang_markdown::MdBook;
use miette::IntoDiagnostic;

use crate::{Result, commands, document};

/// A markdown book document
pub struct MdBookDocument {
    /// Location of the `book.toml`
    pub location: SourceLocation,
    /// The loaded MdBook
    mdbook: MdBook,
    /// Diagnostics
    diags: Diagnostics,
}

impl MdBookDocument {
    pub fn new(url: Url) -> Result<Self> {
        let location = SourceLocation::new(url);
        Ok(Self {
            mdbook: MdBook::load(location.path().ok_or(miette::miette!("Not a file path"))?)
                .into_diagnostic()?,
            location,
            diags: Default::default(),
        })
    }
}

impl commands::Format for document::MdBook {
    fn format(&mut self, params: &commands::FormatParameters) -> Result<bool> {
        let mut formatted = false;
        let config = params;
        self.diags.clear();
        let mut diags = Diagnostics::default();

        self.mdbook
            .code_blocks_mut()
            .filter(|(_, code_block)| code_block.can_format())
            .for_each(|(_, code_block)| {
                match mucad_lang_format::format_code(&code_block.code, config) {
                    Ok(code) => {
                        formatted |= code_block.code != code;
                        code_block.code = code;
                    }
                    Err(diag) => {
                        diags.append(diag);
                    }
                }
            });

        self.diags.append(diags);

        if self.diags.has_errors() {
            Err(miette::miette!("Error formatting mdbook"))
        } else {
            Ok(formatted)
        }
    }
}

impl commands::Sync for document::MdBook {
    fn sync(&self) -> Result {
        todo!()
    }
}
