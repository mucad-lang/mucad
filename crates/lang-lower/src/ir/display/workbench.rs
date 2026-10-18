// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{DisplayOneLine, element::Visibility};

use crate::ir::{self, display::fmt_statements};

impl std::fmt::Display for ir::Init {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Display attributes
        write!(f, "init({})", self.parameters)
    }
}

impl DisplayOneLine for ir::Init {}

impl std::fmt::Display for ir::WorkbenchSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{kind}({parameters})",
            kind = self.kind,
            parameters = self.parameters.to_string_one_line()
        )?;
        writeln!(f, "    inits:")?;
        self.inits
            .iter()
            .try_for_each(|init| writeln!(f, "    - {}", init.to_string_one_line()))
    }
}

impl std::fmt::Display for ir::WorkbenchStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.visibility {
            Visibility::Public => write!(f, "prop ")?,
            Visibility::Private => {}
        };

        // TODO Display attributes and exports
        if let Some(name) = &self.name {
            write!(f, "{name}: {ty}", ty = self.ty)?;
        };
        write!(f, "{}", self.expression)
    }
}

impl DisplayOneLine for ir::WorkbenchStatement {}

impl std::fmt::Display for ir::Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_statements(&self.statements, f)
    }
}

impl DisplayOneLine for ir::Group {}

impl DisplayOneLine for ir::Marker {}

impl std::fmt::Display for ir::WorkbenchExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(
            f,
            "{name}({expr})",
            expr = match &self {
                ir::WorkbenchExpression::Invalid => String::new(),
                ir::WorkbenchExpression::Value(constant_value) =>
                    constant_value.to_string_one_line(),
                ir::WorkbenchExpression::Path(path) => path.to_string_one_line(),
                ir::WorkbenchExpression::Group(group) => group.to_string_one_line(),
                ir::WorkbenchExpression::If(if_) => if_.to_string_one_line(),
                ir::WorkbenchExpression::Call(call) => call.to_string_one_line(),
                ir::WorkbenchExpression::Marker(marker) => marker.to_string_one_line(),
            }
        )
    }
}

impl DisplayOneLine for ir::WorkbenchExpression {}

impl std::fmt::Display for ir::Workbench {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.signature)?;
        fmt_statements(&self.statements, f)
    }
}

impl DisplayOneLine for ir::Workbench {}
