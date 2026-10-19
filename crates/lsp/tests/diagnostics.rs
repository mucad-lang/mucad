// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test document diagnostics with the LSP service.

mod common;

use tower_lsp::{LanguageServer, lsp_types as lsp};

/// An unresolved symbol is an error at its position.
///
/// Evaluation reports the call of the unresolved symbol as another error.
#[tokio::test]
async fn unresolved_symbol() {
    let service = common::service();
    common::open(
        &service,
        "// µ\n__mu::geo2d::Circle(radius = 1mm);\nUnknown();",
    )
    .await;

    let report = service
        .inner()
        .diagnostic(lsp::DocumentDiagnosticParams {
            text_document: lsp::TextDocumentIdentifier::new(common::url()),
            identifier: None,
            previous_result_id: None,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .await
        .expect("No LSP error");

    let lsp::DocumentDiagnosticReportResult::Report(lsp::DocumentDiagnosticReport::Full(report)) =
        report
    else {
        panic!("A full report");
    };
    let Some(diagnostic) = report.full_document_diagnostic_report.items.first() else {
        panic!("A diagnostic: {report:?}");
    };
    assert_eq!(diagnostic.severity, Some(lsp::DiagnosticSeverity::ERROR));
    assert_eq!(
        diagnostic.range,
        lsp::Range::new(lsp::Position::new(2, 0), lsp::Position::new(2, 7))
    );
    assert!(
        diagnostic.message.contains("Unknown"),
        "{}",
        diagnostic.message
    );
}
