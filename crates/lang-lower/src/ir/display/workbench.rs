// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{DisplayOneLine, VersionAnnotation};

use crate::ir::{self, display::fmt_statements};

/// Write the version annotation of an initializer or a property behind it, e.g.
/// ` [deprecated (since 0.5.0)]`, nothing for the default.
fn fmt_ver(ver: &VersionAnnotation, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if *ver == VersionAnnotation::default() {
        Ok(())
    } else {
        write!(f, " [{ver}]")
    }
}

impl std::fmt::Display for ir::Init {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Display the documentation
        write!(f, "init({})", self.parameters)?;
        fmt_ver(&self.attr.ver, f)
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
        // TODO Display attributes and exports
        match self {
            ir::WorkbenchStatement::Expression(statement) => write!(f, "{}", statement.expression),
            ir::WorkbenchStatement::Prop(prop) => {
                write!(f, "prop {name}: {ty}", name = prop.name, ty = prop.ty)?;
                // Behind the name of a property, before its expression, which may be shortened.
                if prop.ver != VersionAnnotation::default() {
                    fmt_ver(&prop.ver, f)?;
                    write!(f, " ")?;
                }
                write!(f, "{}", prop.expression)
            }
            ir::WorkbenchStatement::Assignment(assignment) => write!(
                f,
                "{name}: {ty}{expression}",
                name = assignment.name,
                ty = assignment.ty,
                expression = assignment.expression
            ),
        }
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
                ir::WorkbenchExpression::Error(_) => String::new(),
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
        if !self.statements.is_empty() {
            writeln!(f)?;
            writeln!(f, "    statements:")?;
        }
        self.statements.iter().try_for_each(|statement| {
            // The documentation of a property above it, like the one of an item.
            if let ir::WorkbenchStatement::Prop(prop) = statement {
                prop.doc
                    .content
                    .lines()
                    .try_for_each(|line| writeln!(f, "    /// {line}"))?;
            }
            writeln!(f, "    - {}", statement.to_string_one_line())
        })
    }
}

impl DisplayOneLine for ir::Workbench {}
