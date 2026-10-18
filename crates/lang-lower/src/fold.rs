// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Fold constant values.

use mucad_builtin::{BuiltinEvalContext, BuiltinItem};
use mucad_lang_base::SymbolId;
use mucad_lang_types::{Arguments, Value};

use crate::{LowerContext, ir};

/// A visitor to fold expressions into constant values.
pub struct Fold<'a, 'source> {
    context: &'a mut LowerContext<'source>,
}

impl<'a, 'source> Fold<'a, 'source> {
    /// Create a new fold visitor.
    pub fn new(context: &'a mut LowerContext<'source>) -> Self {
        Self { context }
    }

    /// Try to get the value from a path `__mu::math::PI` => `3.14159...`
    fn path_to_value(&mut self, path: &mut ir::Path) -> Option<Value> {
        match path.symbol_id() {
            Some(SymbolId::Builtin(builtin_id)) => match self.context.builtins.get(*builtin_id) {
                Some(BuiltinItem::Constant(constant)) => Some(constant.value()),
                _ => None,
            },
            _ => None,
        }
    }

    /// Try to compute the value from a call.
    fn call_to_value<Expr: ir::ExprSpec, F>(
        &mut self,
        call: &mut ir::Call<Expr>,
        mut f: F,
    ) -> Option<Value>
    where
        F: FnMut(&mut Fold<'a, 'source>, &mut Expr),
    {
        call.args.args.iter_mut().for_each(|arg| match arg {
            ir::Argument::Unnamed(expr) => f(self, expr),
            ir::Argument::Named { expr, .. } | ir::Argument::Inline { expr, .. } => {
                f(self, expr);
            }
        });

        let builtin_fn = match &call.path.symbol_id() {
            Some(mucad_lang_base::SymbolId::Builtin(builtin_id)) => {
                match self.context.builtins.get(*builtin_id) {
                    Some(BuiltinItem::Function(builtin_fn)) if !builtin_fn.is_impure() => {
                        builtin_fn
                    }
                    _ => return None,
                }
            }
            _ => return None,
        };

        match call.args.into_arguments() {
            Some(arguments) => builtin_fn
                .call(arguments, &mut BuiltinEvalContext::default())
                .ok(),
            None => None,
        }
    }

    fn expr<Expr: ir::ExprSpec>(expr: &mut Expr, value: Option<Value>) {
        if let Some(value) = value {
            *expr = Expr::from(value);
        }
    }
}

impl<Expr: ir::ExprSpec> ir::ArgumentList<Expr> {
    pub(crate) fn into_arguments(&self) -> Option<Arguments> {
        let mut args = Vec::new();

        for arg in &self.args {
            let value = arg.expr().value()?;
            args.push(match arg {
                ir::Argument::Named { name, .. } => (name.clone(), value.clone()),
                _ => return None,
            })
        }

        Some(Arguments::from_iter(args))
    }
}

impl<'a, 'source> ir::visitor::LeafVisitorMut for Fold<'a, 'source> {}

impl<'a, 'source> ir::visitor::ConstantVisitorMut for Fold<'a, 'source> {
    fn visit_constant_expr(&mut self, expr: &mut ir::ConstantExpression) {
        let value = match expr {
            ir::ConstantExpression::Invalid | ir::ConstantExpression::Value(_) => None,
            ir::ConstantExpression::Path(path) => self.path_to_value(path),
            ir::ConstantExpression::Call(call) => {
                self.call_to_value(call, |fold, expr| fold.visit_constant_expr(expr))
            }
        };
        Self::expr(expr, value)
    }
}

// Sub-trait implementations (inherit default traversal behavior)
impl<'a, 'source> ir::visitor::WorkbenchExpressionVisitorMut for Fold<'a, 'source> {
    fn visit_workbench_expr(&mut self, expr: &mut ir::workbench::WorkbenchExpression) {
        let value = match expr {
            ir::WorkbenchExpression::Invalid | ir::WorkbenchExpression::Value(_) => None,
            ir::WorkbenchExpression::Path(path) => self.path_to_value(path),
            ir::WorkbenchExpression::Call(call) => {
                self.call_to_value(call, |fold, expr| fold.visit_workbench_expr(expr))
            }
            ir::WorkbenchExpression::Group(group) => {
                return self.visit_workbench_group(group);
            }
            ir::WorkbenchExpression::If(if_) => {
                return self.visit_workbench_if(if_);
            }
            ir::WorkbenchExpression::Marker(_) => {
                return;
            }
        };
        Self::expr(expr, value)
    }
}

impl<'a, 'source> ir::visitor::WorkbenchVisitorMut for Fold<'a, 'source> {}

impl<'a, 'source> ir::visitor::FnVisitorMut for Fold<'a, 'source> {
    fn visit_fn_expr(&mut self, expr: &mut ir::function::FunctionExpression) {
        let value = match expr {
            ir::FunctionExpression::Invalid | ir::FunctionExpression::Value(_) => None,
            ir::FunctionExpression::Path(path) => self.path_to_value(path),
            ir::FunctionExpression::Call(call) => {
                self.call_to_value(call, |fold, expr| fold.visit_fn_expr(expr))
            }
            ir::FunctionExpression::Scope(scope) => {
                return self.visit_fn_scope(scope);
            }
            ir::FunctionExpression::If(if_) => {
                return self.visit_fn_if(if_);
            }
        };
        Self::expr(expr, value)
    }
}

impl<'a, 'source> ir::visitor::SourceVisitorMut for Fold<'a, 'source> {}
impl<'a, 'source> ir::visitor::VisitorMut for Fold<'a, 'source> {}
