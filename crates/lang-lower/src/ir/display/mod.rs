// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Display implementations of IR items

use mucad_lang_base::{DisplayOneLine, VersionAnnotation};

mod function;
mod source;
mod workbench;

use crate::ir::{self, ConstantExpression};

impl std::fmt::Display for ir::DocBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.content
            .lines()
            .try_for_each(|line| writeln!(f, "/// {line}"))
    }
}

impl DisplayOneLine for ir::DocBlock {}

impl std::fmt::Display for ir::Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.vis {
            mucad_lang_base::element::Visibility::Public => {
                write!(f, "pub ")?;
            }
            mucad_lang_base::element::Visibility::Private => {}
        };

        if let Some(name) = &self.name {
            write!(f, "{name}:")?;
        }
        Ok(())
    }
}

pub(crate) fn fmt_statements<T>(
    statements: &[T],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result
where
    T: DisplayOneLine,
{
    if !statements.is_empty() {
        writeln!(f)?;
        writeln!(f, "    statements:")?;

        statements
            .iter()
            .try_for_each(|stmt| writeln!(f, "    - {}", stmt.to_string_one_line()))
    } else {
        Ok(())
    }
}

impl DisplayOneLine for ir::Path {}

impl DisplayOneLine for ir::ParameterList {}

impl DisplayOneLine for ir::ConstantValue {}

impl std::fmt::Display for ir::Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl DisplayOneLine for ir::Alias {}

impl std::fmt::Display for ir::Wildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::*", self.path)
    }
}

impl DisplayOneLine for ir::Wildcard {}

impl std::fmt::Display for ir::Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{ty}: {expr}", ty = self.ty, expr = self.expr)
    }
}

impl DisplayOneLine for ir::Constant {}

impl std::fmt::Display for ir::Def {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into(); // "Function"

        write!(f, "{name}")?;

        let def = match self {
            ir::Def::Source(source) => source.to_string(),
            ir::Def::Workbench(workbench) => workbench.to_string(),
            ir::Def::Function(function) => function.to_string(),
            ir::Def::Constant(constant) => constant.to_string_one_line(),
            ir::Def::Alias(alias) => alias.to_string_one_line(),
            ir::Def::Wildcard(wildcard) => wildcard.to_string_one_line(),
            _ => String::new(),
        };

        if !def.is_empty() {
            write!(f, " = {def}")
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Display for ir::Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.doc.to_string_one_line();
        if !doc.is_empty() {
            writeln!(f, "{doc}")?;
        }
        let ver = self.ver.to_string_one_line();
        if self.ver != VersionAnnotation::default() && !ver.is_empty() {
            writeln!(f, "{ver}")?;
        }
        let meta = self.meta.to_string();
        if !meta.is_empty() {
            write!(f, "{meta} ")?;
        }
        writeln!(f, "{def}", def = self.def)
    }
}

impl<Expr: ir::ExprSpec> std::fmt::Display for ir::ArgumentList<Expr>
where
    Expr: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", {
            self.args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        })
    }
}

impl std::fmt::Display for ir::Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{id}", id = self.id)?;
        if let Some(ty) = &self.ty {
            write!(f, ": {ty}")?;
        }
        match &self.default_value {
            Some(v) => write!(f, " = {v}"),
            _ => Ok(()),
        }
    }
}

impl std::fmt::Display for ir::ParameterList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.parameters
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<Expr: ir::ExprSpec> std::fmt::Display for ir::If<Expr>
where
    Expr: std::fmt::Display,
    Expr::Body: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "if {cond} {body}", cond = self.cond, body = self.body)?;
        if let Some(next) = &self.next_if {
            writeln!(f, "else {next}")?;
        }
        if let Some(body) = &self.body_else {
            writeln!(f, "else {body}")?;
        }
        Ok(())
    }
}

impl<Expr: ir::ExprSpec> DisplayOneLine for ir::If<Expr>
where
    Expr: DisplayOneLine,
    Expr::Body: DisplayOneLine,
{
}

impl<Expr: ir::ExprSpec> DisplayOneLine for ir::Call<Expr> where Expr: DisplayOneLine {}

impl std::fmt::Display for ir::ConstantExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(
            f,
            "{name}({expr})",
            expr = match &self {
                ir::ConstantExpression::Invalid => String::new(),
                ir::ConstantExpression::Value(constant_value) =>
                    constant_value.to_string_one_line(),
                ir::ConstantExpression::Path(path) => path.to_string_one_line(),
                ir::ConstantExpression::Call(call) => call.to_string_one_line(),
            }
        )
    }
}

impl DisplayOneLine for ConstantExpression {}

impl<Expr> std::fmt::Display for ir::LocalAssignment<Expr>
where
    Expr: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{id}: {ty} = {expr}",
            id = self.id,
            ty = self.ty,
            expr = self.expression
        )
    }
}

impl std::fmt::Display for ir::Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root())
    }
}

impl std::fmt::Display for crate::Ir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree)
    }
}
