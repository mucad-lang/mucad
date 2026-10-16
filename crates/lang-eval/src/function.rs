// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evaluate function

use microcad_builtin::BuiltinItem;
use microcad_lang_base::{DisplayWithCtx, PushIssue, SrcRef, SrcReferrer, ToCompactString};
use microcad_lang_resolve::library::symbol;
use microcad_lang_types::{ArgumentValue, ArgumentValueList, Value};

use crate::{
    Callable, Eval, EvalContext, EvalError, EvalErrorKind, EvalResult,
    context::{ContextScope, FunctionFrame, FunctionScopeFrame},
};

#[derive(Debug, Default)]
pub enum FlowSignal {
    /// When encountering a `Call` or `LocalAssignment` statement
    #[default]
    Continue,
    /// When encountering a `Body` or `If` statement or single expression
    Yield(Value),
    /// When encountering a `Return` statement
    Return(Value),
}

impl FlowSignal {
    /// Extract the inner `Value` if the signal is `Break` or `Return`.
    pub fn into_value(self) -> Value {
        match self {
            FlowSignal::Yield(val) | FlowSignal::Return(val) => val,
            FlowSignal::Continue => Value::None,
        }
    }

    /// Unwrap the inner `Value` or return an evaluation error if it was `Continue`.
    pub fn expect_value(self, src_ref: SrcRef) -> EvalResult {
        match self {
            FlowSignal::Yield(val) | FlowSignal::Return(val) => Ok(val),
            FlowSignal::Continue => Err(EvalErrorKind::ExpectedExpression { src_ref }.into()),
        }
    }
}

impl Eval<ArgumentValue> for symbol::function::Argument {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<ArgumentValue> {
        Ok(match self {
            symbol::function::Argument::Unnamed(expr) => {
                ArgumentValue::new(expr.eval(context)?.into_value(), None)
            }
            symbol::function::Argument::Named { name, expr, .. }
            | symbol::function::Argument::Inline { name, expr, .. } => {
                ArgumentValue::new(expr.eval(context)?.into_value(), Some(name.clone()))
            }
        })
    }
}

impl Eval<ArgumentValueList> for symbol::function::ArgumentList {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<ArgumentValueList> {
        let map: Vec<ArgumentValue> = self
            .args
            .iter()
            .map(|arg| arg.eval(context))
            .collect::<EvalResult<Vec<_>>>()?;

        Ok(ArgumentValueList {
            args: map,
            src_ref: self.src_ref,
        })
    }
}

impl Eval<FlowSignal> for symbol::function::Scope {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        context.scope(FunctionScopeFrame::new(), |context| {
            self.statements.eval(context)
        })
    }
}

impl Eval<FlowSignal> for symbol::function::FunctionIf {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        // 1. Evaluate condition expression
        let cond_signal = self.cond.eval(context)?;
        let cond_val: bool = cond_signal.expect_value(self.src_ref)?.try_into()?;

        // 2. Dispatch based on condition boolean
        if cond_val {
            self.body.eval(context)
        } else if let Some(next_if) = &self.next_if {
            // Handle `else if ...` chain
            next_if.eval(context)
        } else if let Some(body_else) = &self.body_else {
            // Handle `else { ... }` block
            body_else.eval(context)
        } else {
            // If condition is false and no else/else-if branch exists,
            // continue execution without yielding a value.
            Ok(FlowSignal::Continue)
        }
    }
}

impl Eval<FlowSignal> for symbol::function::FunctionCall {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        use crate::argument_match::ArgumentMatch;
        match &self.path {
            symbol::Path::Resolved(symbol::SymbolId::Builtin(builtin_id)) => {
                let args = self.args.eval(context)?;

                match context.builtins().get(*builtin_id) {
                    Some(BuiltinItem::Function(f)) => {
                        let args = f.argument_match(&args).map_err(|err| {
                            EvalError::argument_match(self, f.info.item_name(), err)
                        })?;

                        let value =
                            context.with_builtin_context(self.src_ref, |ctx| f.call(args, ctx))?;
                        Ok(FlowSignal::Yield(value))
                    }
                    Some(_) => context.catch(EvalErrorKind::SymbolCannotBeCalled {
                        path: context.current_symbol_name().unwrap_or_default(),
                        src_ref: self.src_ref,
                    }),
                    None => context.catch(EvalErrorKind::BuiltinNotFound {
                        name: builtin_id.to_string_with_ctx(context),
                        src_ref: self.src_ref,
                    }),
                }
            }
            path => context.catch(EvalError::new(EvalErrorKind::SymbolCannotBeCalled {
                path: context.current_symbol_name().unwrap_or_default(),
                src_ref: self.src_ref,
            })),
        }
    }
}

impl Eval<FlowSignal> for symbol::Path {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        match self {
            symbol::Path::Resolved(symbol::SymbolId::Local(identifier)) => {
                use crate::context::StackRead;
                match context.get_local(identifier) {
                    Some(local) => Ok(FlowSignal::Yield(local.clone())),
                    None => context.catch(EvalErrorKind::LocalNotFound(identifier.clone())),
                }
            }
            symbol::Path::Resolved(symbol::SymbolId::Builtin(builtin)) => {
                match context.builtins().get(*builtin) {
                    Some(BuiltinItem::Constant(c)) => Ok(FlowSignal::Yield(c.value())),
                    Some(item) => context.catch(EvalErrorKind::BuiltinConstantExpected {
                        name: item.name().to_string(),
                        src_ref: self.src_ref(),
                    }),
                    None => context.catch(EvalErrorKind::BuiltinNotFound {
                        name: builtin.to_string_with_ctx(context),
                        src_ref: self.src_ref(),
                    }),
                }
            }
            symbol::Path::Resolved(id) | symbol::Path::HumanReadable { id, .. } => {
                Ok(FlowSignal::Yield(id.eval(context)?))
            }
            symbol::Path::Unresolved(unresolved_path) => {
                context.catch(EvalErrorKind::UnresolvedPath {
                    path: unresolved_path.to_string(),
                    src_ref: unresolved_path.src_ref,
                })
            }
        }
    }
}

impl Eval<FlowSignal> for symbol::FunctionExpression {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        use symbol::FunctionExpression as Expr;
        match self {
            // The error has already been reported when lowering.
            Expr::Invalid => Ok(FlowSignal::Yield(Value::None)),
            Expr::Value(literal) => Ok(FlowSignal::Yield(literal.value().clone())),
            Expr::Path(name) => name.eval(context),
            Expr::Scope(s) => s.eval(context),
            Expr::If(if_) => if_.eval(context),
            Expr::Call(call) => call.eval(context),
            _ => todo!(),
        }
    }
}

impl Eval<FlowSignal> for symbol::FunctionStatement {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        use symbol::FunctionStatement as Stmt;
        match self {
            Stmt::Scope(scope) => scope.eval(context),
            Stmt::Call(call) => {
                let value = call.eval(context)?.expect_value(call.src_ref)?;
                if !value.is_none() {
                    context.push_err(EvalErrorKind::CallReturnValueIgnored(call.src_ref))
                }

                Ok(FlowSignal::Continue)
            }
            Stmt::Return(stmt) => match &stmt.expr {
                Some(expr) => Ok(FlowSignal::Return(
                    expr.eval(context)?.expect_value(self.src_ref())?,
                )),
                None => Ok(FlowSignal::Continue),
            },
            Stmt::Tail(stmt) => stmt.eval(context),
            Stmt::If(if_stmt) => if_stmt.eval(context),
            Stmt::Local(l) => {
                match l.expression.eval(context)? {
                    FlowSignal::Continue => {
                        context.catch(EvalErrorKind::LocalExpressionDidNotProduceAValue {
                            id: l.id.clone(),
                            src_ref: l.src_ref,
                            expr_src_ref: l.expression.src_ref(),
                        })
                    }
                    // Save yielded value in local table
                    FlowSignal::Yield(value) => {
                        context.top_mut().put_local(l.id.to_compact_string(), value);

                        Ok(FlowSignal::Continue)
                    }
                    // Return value immediately
                    FlowSignal::Return(value) => Ok(FlowSignal::Return(value)),
                }
            }
            _ => todo!(),
        }
    }
}

impl Eval<FlowSignal> for Box<[symbol::FunctionStatement]> {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<FlowSignal> {
        for stmt in self {
            match stmt.eval(context) {
                Ok(FlowSignal::Continue) => {
                    continue;
                }
                ret => {
                    return ret;
                }
            }
        }

        Ok(FlowSignal::Continue)
    }
}

impl Callable for symbol::Function {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        use crate::ArgumentMatch;
        match self.argument_match(args) {
            Ok(args) => context.scope(FunctionFrame::new(args), |context| {
                Ok(self.statements.eval(context)?.into_value())
            }),
            Err(err) => Err(EvalError::argument_match(
                context.current_symbol_src_ref(),
                context.current_symbol_name().unwrap_or_default(),
                err,
            )),
        }
    }
}
