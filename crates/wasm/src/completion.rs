// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WASM completion API.

use mucad_driver::prelude as mu;
use wasm_bindgen::prelude::wasm_bindgen;

/// A symbol to complete a path with, e.g. `Circle` for `std::geo2d::Ci`.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Completion {
    name: String,
    kind: String,
    doc: Option<String>,
}

#[wasm_bindgen]
impl Completion {
    /// The name of the symbol, e.g. `Circle`.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// The kind of the symbol: `library`, `mod`, `sketch`, `part`, `op`, `fn`, `const` or
    /// `built-in`.
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    /// The documentation of the symbol in Markdown, if it has some.
    #[wasm_bindgen(getter)]
    pub fn doc(&self) -> Option<String> {
        self.doc.clone()
    }
}

impl From<mu::resolve::Completion> for Completion {
    fn from(completion: mu::resolve::Completion) -> Self {
        use mu::resolve::CompletionKind;

        Self {
            name: completion.name.to_string(),
            kind: match completion.kind {
                CompletionKind::Library => "library".into(),
                CompletionKind::Module => "mod".into(),
                CompletionKind::Workbench(kind) => kind.to_string(),
                CompletionKind::Function => "fn".into(),
                CompletionKind::Constant => "const".into(),
                CompletionKind::Builtin => "built-in".into(),
            },
            doc: completion.doc,
        }
    }
}

/// The byte offset of a line and column (from 1, in UTF-16 code units) in `code`.
///
/// A position after the end of a line is at its end, after the end of the code at its end.
pub fn offset(code: &str, line_no: u32, column: u32) -> usize {
    let mut line_start = 0;
    for _ in 1..line_no {
        match code[line_start..].find('\n') {
            Some(index) => line_start += index + 1,
            None => return code.len(),
        }
    }
    let line = code[line_start..].split('\n').next().unwrap_or_default();
    let mut units = 0;
    for (index, ch) in line.char_indices() {
        if units + 1 >= column as usize {
            return line_start + index;
        }
        units += ch.len_utf16();
    }
    line_start + line.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lines and columns are converted into byte offsets, also after characters outside of
    /// ASCII, which have more bytes than UTF-16 code units.
    #[test]
    fn offsets() {
        let code = "ab\nµ🚀x\n";
        assert_eq!(offset(code, 1, 1), 0);
        assert_eq!(offset(code, 1, 3), 2);
        assert_eq!(offset(code, 2, 1), 3);
        // `µ` has 2 bytes and 1 code unit, `🚀` 4 bytes and 2 code units.
        assert_eq!(offset(code, 2, 2), 5);
        assert_eq!(offset(code, 2, 4), 9);
        // After the end of a line and of the code.
        assert_eq!(offset(code, 1, 99), 2);
        assert_eq!(offset(code, 9, 1), code.len());
    }
}
