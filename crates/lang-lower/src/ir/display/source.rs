// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::DisplayOneLine;

use crate::ir;

impl std::fmt::Display for ir::SourceStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO Display attributes and exports
        if let Some(name) = &self.name {
            write!(f, "{name}: {ty} = ", ty = self.ty)?;
        }

        write!(f, "{}", self.expression)
    }
}

impl DisplayOneLine for ir::SourceStatement {}

impl std::fmt::Display for ir::Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::fmt_statements(&self.statements, f)
    }
}

impl DisplayOneLine for ir::Source {
    fn to_string_one_line(&self) -> String {
        String::new()
    }
}
