// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_builtin::{
    BuiltinEvalContext, BuiltinFunction, BuiltinItem, BuiltinOperation, BuiltinPrimitive,
};
use microcad_lang_base::PushIssue;
use microcad_lang_types::{ArgumentValueList, ModelTree};

use crate::{
    ArgumentMatch, Callable, EvalContext, EvalError, EvalErrorKind, EvalResult,
    context::ContextScope,
};

impl Callable for BuiltinPrimitive {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        use crate::ArgumentMatch;

        let multi_args = self.argument_multi_match(args).map_err(|err| {
            EvalError::argument_match(context.current_symbol_src_ref(), self.info.item_name(), err)
        })?;
        let mut models = Vec::new();
        for args in multi_args {
            models.push(ModelTree::from((self.f)(
                args,
                &mut BuiltinEvalContext::default(),
            )?));
        }

        Ok(ModelTree::to_multiplicity(models).into())
    }
}

impl Callable for BuiltinFunction {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        let src_ref = context.current_symbol_src_ref();
        let name = context.current_symbol_name().unwrap_or_default();
        let args = self
            .argument_match(args)
            .map_err(|err| EvalError::argument_match(src_ref, name, err))?;

        Ok(self.call_isolated(args)?)
    }
}

impl Callable for BuiltinOperation {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        let src_ref = context.current_symbol_src_ref();
        let name = context.current_symbol_name().unwrap_or_default();
        let multi_args = self
            .argument_multi_match(args)
            .map_err(|err| EvalError::argument_match(src_ref, name, err))?;
        let mut models = Vec::new();
        for args in multi_args {
            models.push((self.f)(args, &mut BuiltinEvalContext::default())?);
        }

        Ok(ModelTree::to_multiplicity(models).into())
    }
}

impl Callable for BuiltinItem {
    fn call(&self, args: &ArgumentValueList, context: &mut EvalContext) -> EvalResult {
        match self {
            BuiltinItem::Function(f) => f.call(args, context),
            BuiltinItem::Primitive(p) => p.call(args, context),
            BuiltinItem::Operation(op) => op.call(args, context),
            BuiltinItem::Constant(_) | BuiltinItem::Module(_) => {
                context.catch(EvalErrorKind::SymbolCannotBeCalled {
                    path: context.current_symbol_name().unwrap_or_default(),
                    src_ref: context.current_symbol_src_ref(),
                })
            }
        }
    }
}
