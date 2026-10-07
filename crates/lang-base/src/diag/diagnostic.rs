// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{GetSourceByHash, Source};
use crate::{diag::*, src_ref::*};
use derive_more::{Deref, Display};
use miette::SourceCode;

#[derive(Deref, Debug, Display)]
#[display("Error: {report} | Source: {src_ref}")]
pub struct Diagnostic {
    #[deref]
    pub report: miette::Report,
    pub src_ref: SrcRef,
}

impl Diagnostic {
    /// Pretty print the diagnostic.
    pub fn render(
        &self,
        mut f: &mut dyn std::fmt::Write,
        source_by_hash: &impl GetSourceByHash,
        options: &DiagRenderOptions,
    ) -> std::fmt::Result {
        let src_ref = self.src_ref;
        let hash = src_ref.source_hash();

        match src_ref.is_none() {
            true => writeln!(f, "{}", self.report)?,
            false => {
                if let Some(source) = source_by_hash.get_source_by_hash(hash) {
                    let handler = miette::GraphicalReportHandler::new_themed(options.theme());
                    handler.render_report(
                        &mut f,
                        &DiagnosticWrapper {
                            diagnostic: self,
                            source,
                        },
                    )?
                };
            }
        }

        Ok(())
    }

    /// Render the diagnostics to a string, see `pretty_print` for more information
    pub fn render_to_string(
        &self,
        source_by_hash: &impl GetSourceByHash,
        options: &DiagRenderOptions,
    ) -> String {
        let mut buff = String::new();
        self.render(&mut buff, source_by_hash, options)
            .expect("format to string can't fail");
        buff
    }
}

impl From<Diagnostic> for miette::Report {
    fn from(value: Diagnostic) -> Self {
        value.report
    }
}

impl From<&'static str> for Diagnostic {
    fn from(value: &'static str) -> Self {
        Self {
            report: miette::miette!(value),
            src_ref: SrcRef::none(),
        }
    }
}

impl SrcReferrer for Diagnostic {
    fn src_ref(&self) -> SrcRef {
        self.src_ref
    }
}

#[derive(Display, Debug)]
#[display("{diagnostic}")]
struct DiagnosticWrapper<'a> {
    diagnostic: &'a Diagnostic,
    source: &'a Source,
}

impl std::error::Error for DiagnosticWrapper<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.diagnostic.source()
    }
}

impl miette::Diagnostic for DiagnosticWrapper<'_> {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.diagnostic.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.diagnostic.severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.diagnostic.help()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(self.source)
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        self.diagnostic.diagnostic_source()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.diagnostic.labels().or_else(|| {
            let label = miette::LabeledSpan::new_with_span(
                Some(self.diagnostic.to_string()),
                self.diagnostic.src_ref.span(),
            );
            Some(Box::new(std::iter::once(label)))
        })
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        self.diagnostic.related()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.diagnostic.url()
    }
}
