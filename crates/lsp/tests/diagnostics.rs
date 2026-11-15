// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Test document diagnostics with the LSP service.

mod common;

use tower_lsp::{LanguageServer, lsp_types as lsp};

/// The diagnostics of the document `code`.
async fn diagnostics(code: &str) -> Vec<lsp::Diagnostic> {
    let service = common::service();
    common::open(&service, code).await;

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
    report.full_document_diagnostic_report.items
}

/// An unresolved symbol is an error at its position.
///
/// Evaluation reports the call of the unresolved symbol as another error.
#[tokio::test]
async fn unresolved_symbol() {
    let diagnostics = diagnostics("// µ\n__mu::geo2d::Circle(radius = 1mm);\nUnknown();").await;
    let Some(diagnostic) = diagnostics.first() else {
        panic!("A diagnostic: {diagnostics:?}");
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

/// The use of a deprecated symbol is a warning, which editors show crossed out.
#[tokio::test]
async fn deprecated_symbol() {
    let diagnostics = diagnostics(
        "#[deprecated]\nfn size() {\n    return 1mm;\n}\n\n__mu::geo2d::Circle(radius = size());\n",
    )
    .await;
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("A diagnostic: {diagnostics:?}");
    };
    assert_eq!(diagnostic.severity, Some(lsp::DiagnosticSeverity::WARNING));
    assert_eq!(
        diagnostic.code,
        Some(lsp::NumberOrString::String("resolve::deprecated".into()))
    );
    assert_eq!(diagnostic.tags, Some(vec![lsp::DiagnosticTag::DEPRECATED]));
    assert_eq!(
        diagnostic.range,
        lsp::Range::new(lsp::Position::new(5, 29), lsp::Position::new(5, 33))
    );
}

/// Other warnings are not crossed out, e.g. of a name in a wrong case.
#[tokio::test]
async fn warning_without_tag() {
    let diagnostics =
        diagnostics("const small = 1mm;\n__mu::geo2d::Circle(radius = small);\n").await;
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("A diagnostic: {diagnostics:?}");
    };
    assert_eq!(diagnostic.severity, Some(lsp::DiagnosticSeverity::WARNING));
    assert_eq!(diagnostic.tags, None);
}
