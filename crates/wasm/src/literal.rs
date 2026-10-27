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

/// A literal after a step, see `Mucad.stepLiteralAt`.
///
/// Positions are counted in UTF-16 code units from the start of the literal, like in JavaScript.
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub struct SteppedLiteral {
    text: String,
    digit: Option<(u32, u32)>,
}

#[wasm_bindgen]
impl SteppedLiteral {
    /// The literal, e.g. `1.60mm`.
    #[wasm_bindgen(getter)]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// The start of the stepped digit, e.g. to select it, `undefined` without a digit, e.g. of a
    /// bool.
    #[wasm_bindgen(getter, js_name = digitStart)]
    pub fn digit_start(&self) -> Option<u32> {
        self.digit.map(|(start, _)| start)
    }

    /// The end of the stepped digit.
    #[wasm_bindgen(getter, js_name = digitEnd)]
    pub fn digit_end(&self) -> Option<u32> {
        self.digit.map(|(_, end)| end)
    }
}

/// The `literal` stepped by `factor` times the place of the digit at `cursor` (in UTF-16 code
/// units), see [`mucad_driver::step_literal_at`].
pub fn step_at(literal: &str, cursor: u32, factor: f64) -> Option<SteppedLiteral> {
    let utf16 = |text: &str| text.encode_utf16().count() as u32;
    let cursor = (literal.char_indices())
        .find(|(index, _)| utf16(&literal[..*index]) >= cursor)
        .map_or(literal.len(), |(index, _)| index);
    let stepped =
        mucad_driver::step_literal_at(literal, cursor, Scalar::checked_from_num(factor)?)?;
    let digit = (stepped.digit).map(|digit| {
        (
            utf16(&stepped.text[..digit.start]),
            utf16(&stepped.text[..digit.end]),
        )
    });
    Some(SteppedLiteral {
        text: stepped.text,
        digit,
    })
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
