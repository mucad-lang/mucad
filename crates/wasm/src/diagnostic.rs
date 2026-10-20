// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WASM diagnostic API.

use mucad_driver::prelude as mu;
use wasm_bindgen::prelude::wasm_bindgen;

/// The severity of a diagnostic.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Advice,
}

/// An error, warning or advice with its range in the code, e.g. for an editor marker.
///
/// Lines and columns start at 1, columns are counted in UTF-16 code units like in JavaScript.
/// A diagnostic without position in the code, e.g. of the standard library, is at its start.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Diagnostic {
    message: String,
    start_line_no: u32,
    start_column: u32,
    end_line_no: u32,
    end_column: u32,
    severity: Severity,
}

#[wasm_bindgen]
impl Diagnostic {
    /// The message with a help, if there is one.
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// The line of the start, from 1.
    #[wasm_bindgen(getter, js_name = startLineNo)]
    pub fn start_line_no(&self) -> u32 {
        self.start_line_no
    }

    /// The column of the start, from 1.
    #[wasm_bindgen(getter, js_name = startColumn)]
    pub fn start_column(&self) -> u32 {
        self.start_column
    }

    /// The line of the end, from 1.
    #[wasm_bindgen(getter, js_name = endLineNo)]
    pub fn end_line_no(&self) -> u32 {
        self.end_line_no
    }

    /// The column after the end, from 1.
    #[wasm_bindgen(getter, js_name = endColumn)]
    pub fn end_column(&self) -> u32 {
        self.end_column
    }

    /// Whether this is an error, a warning or an advice.
    #[wasm_bindgen(getter)]
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

impl Diagnostic {
    /// The diagnostic of the compiler with its range in `source`, the compiled code.
    ///
    /// The code is needed for the end of the range and for the columns in UTF-16 code units.
    pub fn new(diagnostic: &mu::Diagnostic, source: &mu::Source) -> Self {
        let code = source.code();
        let src_ref = &diagnostic.src_ref;
        let (start, end) = if src_ref.is_some() && src_ref.source_hash() == source.hash_id() {
            (src_ref.start, src_ref.end)
        } else {
            (0, 0)
        };
        let (start_line_no, start_column) = line_column(code, start);
        let (end_line_no, end_column) = line_column(code, end);

        Self {
            message: match diagnostic.help() {
                Some(help) => format!("{}\n{help}", diagnostic.report),
                None => diagnostic.report.to_string(),
            },
            start_line_no,
            start_column,
            end_line_no,
            end_column,
            severity: match diagnostic.severity() {
                Some(mu::base::Severity::Advice) => Severity::Advice,
                Some(mu::base::Severity::Warning) => Severity::Warning,
                Some(mu::base::Severity::Error) | None => Severity::Error,
            },
        }
    }
}

/// The line and column (from 1, in UTF-16 code units) of the byte `offset` in `code`.
fn line_column(code: &str, offset: usize) -> (u32, u32) {
    let before = code.get(..offset).unwrap_or(code);
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    (
        before.matches('\n').count() as u32 + 1,
        before[line_start..].encode_utf16().count() as u32 + 1,
    )
}
