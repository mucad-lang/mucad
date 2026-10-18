// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Various type conversion from µcad to tower_lsp's types.

use tower_lsp::lsp_types as lsp;

use mucad_driver::prelude as mu;
use mu::traits::*;

pub trait ToLsp {
    type Output;

    fn to_lsp(&self) -> Self::Output;
}

impl ToLsp for mu::SrcRef {
    type Output = Option<lsp::Range>;

    fn to_lsp(&self) -> Self::Output {
        match self.is_some() {
            true => {
                let start = lsp::Position::new(self.at.line, self.at.col - 1);
                let end = lsp::Position::new(self.at.line, (self.at.col + self.len() as u32) - 1);

                Some(lsp::Range::new(start, end))
            }
            false => None,
        }
    }
}

impl ToLsp for mu::base::DiagLevel {
    type Output = lsp::DiagnosticSeverity;

    fn to_lsp(&self) -> Self::Output {
        use mu::base::DiagLevel::*;
        match &self {
            Trace => lsp::DiagnosticSeverity::HINT,
            Info => lsp::DiagnosticSeverity::INFORMATION,
            Warning => lsp::DiagnosticSeverity::WARNING,
            Error => lsp::DiagnosticSeverity::ERROR,
        }
    }
}

impl ToLsp for mu::Diagnostics {
    type Output = lsp::FullDocumentDiagnosticReport;

    fn to_lsp(&self) -> Self::Output {
        Self::Output {
            result_id: None,
            items: self
                .iter()
                .filter_map(|diag| {
                    let message = diag.message();
                    diag.src_ref().to_lsp().map(|range| {
                        lsp::Diagnostic::new(
                            range,
                            Some(diag.level().to_lsp()),
                            None,
                            None,
                            message,
                            None,
                            None,
                        )
                    })
                })
                .collect(),
        }
    }
}

impl ToLsp for mu::TextEdit {
    type Output = lsp::TextEdit;

    fn to_lsp(&self) -> Self::Output {
        Self::Output {
            range: self.src_ref.to_lsp().unwrap_or_default(),
            new_text: self.new_text.clone(),
        }
    }
}
