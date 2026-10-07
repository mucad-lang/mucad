// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_lang_base::{PushIssue, SrcReferrer};
use microcad_lang_resolve::library::symbol;
use microcad_lang_types::{ArgumentValueList, Value};

use crate::{Callable, Eval, EvalContext, EvalErrorKind, EvalResult};

impl Callable for symbol::SymbolDef {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        use symbol::SymbolDef::*;
        match self {
            Workbench(workbench) => workbench.call(args, context),
            Function(function) => function.call(args, context),
            _ => todo!("This symbol cannot be called"),
        }
    }
}

impl Eval for symbol::SymbolDef {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        use symbol::SymbolDef::*;

        match self {
            Root(source_file) => match source_file {
                Some(source_file) => source_file.eval(context),
                None => todo!("Source File not loaded"),
            },

            Constant(constant) => match constant.value() {
                Some(value) => Ok(value.clone()),
                None => context.catch(EvalErrorKind::ConstantExpressionExpected {
                    src_ref: constant.expr.src_ref(),
                }),
            },
            SourceFile(source_file) => source_file.eval(context),
            def => todo!("Error handling: {def} not evaluated"),
        }
    }
}

impl Eval for symbol::SymbolId {
    fn eval(&self, context: &mut EvalContext) -> EvalResult {
        use crate::context::ContextScope;

        match &self {
            symbol::SymbolId::Builtin(builtin_id) => match context.builtins.get(*builtin_id) {
                Some(builtin) => builtin.eval(context),
                None => {
                    // TODO context.diag(BuiltinError::NoBuiltin { full_name: (), id: *builtin_id })
                    Ok(Value::None)
                }
            },
            symbol::SymbolId::Local(local_id) => match context.look_up_local(local_id) {
                Some(value) => Ok(value.clone()),
                None => {
                    todo!("Error handling: Local '{local_id}' not found")
                }
            },
            symbol::SymbolId::Item(id) => context.eval_symbol_node(context.library(), *id),
            symbol::SymbolId::External(external_id) => context.eval_symbol_node(
                context.external_library(&external_id.lib_id),
                external_id.id,
            ),
        }
    }
}

impl Eval for symbol::Symbol {
    fn eval(&self, _context: &mut EvalContext) -> EvalResult<Value> {
        todo!()
    }
}
