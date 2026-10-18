// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::DisplayOneLine;

use crate::ir;

impl std::fmt::Display for ir::FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}){}",
            self.parameters,
            if let Some(ret) = &self.return_type {
                format!("-> {ret}")
            } else {
                String::default()
            }
        )
    }
}

impl DisplayOneLine for ir::FunctionSignature {}

impl std::fmt::Display for ir::Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        super::fmt_statements(&self.statements, f)
    }
}

impl DisplayOneLine for ir::Scope {}

impl std::fmt::Display for ir::FunctionExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        write!(
            f,
            "{name}({expr})",
            expr = match &self {
                ir::FunctionExpression::Invalid => String::new(),
                ir::FunctionExpression::Value(constant_value) =>
                    constant_value.to_string_one_line(),
                ir::FunctionExpression::Path(path) => path.to_string_one_line(),
                ir::FunctionExpression::Scope(scope) => scope.to_string_one_line(),
                ir::FunctionExpression::If(if_) => if_.to_string_one_line(),
                ir::FunctionExpression::Call(call) => call.to_string_one_line(),
            }
        )
    }
}

impl DisplayOneLine for ir::FunctionExpression {}

impl std::fmt::Display for ir::ReturnStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return")?;
        match &self.expr {
            Some(expr) => write!(f, " {expr}"),
            None => Ok(()),
        }
    }
}

impl std::fmt::Display for ir::FunctionStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ir::FunctionStatement::Local(local_assignment) => local_assignment.to_string(),
                ir::FunctionStatement::Scope(scope) => scope.to_string(),
                ir::FunctionStatement::Call(call) => call.to_string(),
                ir::FunctionStatement::If(if_) => if_.to_string(),
                ir::FunctionStatement::Tail(function_expression) => function_expression.to_string(),
                ir::FunctionStatement::Return(return_statement) => return_statement.to_string(),
            }
        )
    }
}

impl DisplayOneLine for ir::FunctionStatement {}

impl std::fmt::Display for ir::Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.signature)?;
        super::fmt_statements(&self.statements, f)
    }
}

impl DisplayOneLine for ir::Function {}
