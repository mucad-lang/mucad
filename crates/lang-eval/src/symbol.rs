// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_lang_base::{DisplayWithCtx, PushIssue, SrcRef, SrcReferrer};
use mucad_lang_resolve::library::symbol;
use mucad_lang_types::{ArgumentValueList, Value};

use crate::{
    Callable, Eval, EvalContext, EvalErrorKind, EvalResult,
    context::{BuiltinItemFrame, ContextScope},
};

impl Callable for symbol::SymbolDef {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        use symbol::SymbolDef::*;
        match self {
            Workbench(workbench) => workbench.call(args, context),
            Function(function) => function.call(args, context),
            // An alias stands for its target, e.g. `pub use __mu::debug::print;`.
            Alias(alias) => match alias.path.symbol_id() {
                Some(target) => target.call(args, context),
                None => context.catch(EvalErrorKind::SymbolCannotBeCalled {
                    path: alias.path.to_string(),
                    src_ref: SrcRef::none(),
                }),
            },
            def => context.catch(EvalErrorKind::SymbolCannotBeCalled {
                path: def.to_string(),
                src_ref: SrcRef::none(),
            }),
        }
    }
}

impl Eval for symbol::SymbolDef {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        use symbol::SymbolDef::*;

        match self {
            Root(source_file) => match source_file {
                Some(source_file) => source_file.eval(context),
                None => context.catch(EvalErrorKind::SourceFileNotLoaded),
            },

            Constant(constant) => match constant.value() {
                Some(value) => Ok(value.clone()),
                None => context.catch(EvalErrorKind::ConstantExpressionExpected {
                    src_ref: constant.expr.src_ref(),
                }),
            },
            SourceFile(source_file) => source_file.eval(context),
            // An alias stands for its target, e.g. `pub use __mu::math::PI;`.
            Alias(alias) => match alias.path.symbol_id() {
                Some(target) => target.eval(context),
                None => context.catch(EvalErrorKind::SymbolNotEvaluable {
                    symbol: alias.path.to_string(),
                }),
            },
            def => context.catch(EvalErrorKind::SymbolNotEvaluable {
                symbol: def.to_string(),
            }),
        }
    }
}

impl Callable for symbol::SymbolId {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        match &self {
            symbol::SymbolId::Builtin(builtin_id) => match context.builtins().get(*builtin_id) {
                Some(item) => {
                    let src_ref = context.current_symbol_src_ref();
                    context.scope(BuiltinItemFrame::new(src_ref, item), |context| {
                        item.call(args, context)
                    })
                }
                None => context.catch(EvalErrorKind::BuiltinNotFound {
                    name: builtin_id.to_string_with_ctx(context),
                    src_ref: context.current_symbol_src_ref(),
                }),
            },
            symbol::SymbolId::Item(id) => match context.lib_id() {
                Some(lib_id) => context.call_symbol(*lib_id, *id, args),
                None => context.catch(EvalErrorKind::NoLibraryToEvaluate),
            },
            symbol::SymbolId::External(external_id) => {
                context.call_symbol(external_id.lib_id, external_id.id, args)
            }
            symbol::SymbolId::Local(_) => context.catch(EvalErrorKind::SymbolCannotBeCalled {
                // The resolved name instead of an ID, e.g. `__mu::math::PI`.
                path: self.to_string_with_ctx(context),
                src_ref: context.current_symbol_src_ref(),
            }),
        }
    }
}

impl Eval for symbol::SymbolId {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        use crate::context::ContextScope;

        match &self {
            symbol::SymbolId::Builtin(builtin_id) => match context.builtins().get(*builtin_id) {
                Some(builtin) => builtin.eval(context),
                None => {
                    // TODO context.diag(BuiltinError::NoBuiltin { full_name: (), id: *builtin_id })
                    Ok(Value::None)
                }
            },
            symbol::SymbolId::Local(local_id) => match context.look_up_local(local_id) {
                Some(value) => Ok(value.clone()),
                None => context.catch(EvalErrorKind::LocalNotFound(local_id.clone())),
            },
            symbol::SymbolId::Item(id) => match context.lib_id() {
                Some(lib_id) => context.eval_symbol(*lib_id, *id),
                None => context.catch(EvalErrorKind::NoLibraryToEvaluate),
            },
            symbol::SymbolId::External(external_id) => {
                context.eval_symbol(external_id.lib_id, external_id.id)
            }
        }
    }
}

impl Eval for symbol::Symbol {
    fn eval(&self, context: &mut EvalContext) -> EvalResult<Value> {
        self.def.eval(context)
    }
}
