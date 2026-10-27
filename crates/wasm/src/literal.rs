// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WASM API to step literals in code, e.g. `10mm` to `11mm`.

use mucad_driver::prelude as mu;
use wasm_bindgen::prelude::wasm_bindgen;

use mu::types::Scalar;

use crate::{completion, diagnostic};

/// The range of a literal in a line of code, e.g. under the cursor.
///
/// Lines and columns start at 1, columns are counted in UTF-16 code units like in JavaScript.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiteralRange {
    line_no: u32,
    start_column: u32,
    end_column: u32,
}

#[wasm_bindgen]
impl LiteralRange {
    /// The line of the literal, from 1.
    #[wasm_bindgen(getter, js_name = lineNo)]
    pub fn line_no(&self) -> u32 {
        self.line_no
    }

    /// The column of the start, from 1.
    #[wasm_bindgen(getter, js_name = startColumn)]
    pub fn start_column(&self) -> u32 {
        self.start_column
    }

    /// The column after the end, from 1.
    #[wasm_bindgen(getter, js_name = endColumn)]
    pub fn end_column(&self) -> u32 {
        self.end_column
    }
}

/// The literal `literal` increased by `step` in its unit, see [`mucad_driver::step_literal`].
pub fn step(literal: &str, step: f64) -> Option<String> {
    mucad_driver::step_literal(literal, Scalar::checked_from_num(step)?)
}

/// The range of the literal at a line and column (from 1, in UTF-16 code units) in `code`,
/// see [`mucad_driver::literal_at`].
pub fn literal_at(code: &str, line_no: u32, column: u32) -> Option<LiteralRange> {
    let (span, _) = mucad_driver::literal_at(code, completion::offset(code, line_no, column))?;
    let (line_no, start_column) = diagnostic::line_column(code, span.start);
    let (_, end_column) = diagnostic::line_column(code, span.end);
    Some(LiteralRange {
        line_no,
        start_column,
        end_column,
    })
}
