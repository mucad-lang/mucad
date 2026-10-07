// Copyright © 2024-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod stack;

use microcad_lang_resolve::{
    Library, LibraryCache,
    library::symbol::{SymbolNodeId, SymbolNodeRef},
};
pub use stack::*;

use microcad_builtin::BuiltinRegistry;
use microcad_lang_base::{
    IssueList, LibraryId, LookUpName, Name, PushIssue, Shared, SrcRef, SrcReferrer,
};
use microcad_lang_types::{Value, model::ModelTreeBuilderMut};

use crate::{Eval, EvalError, EvalInfo, EvalIssue, EvalResult, EvalWarning};

#[derive(Debug, Default)]
pub struct EvalContext {
    stack: Stack,

    issues: IssueList<EvalIssue>,

    pub builtins: BuiltinRegistry,

    pub lib: Option<Shared<Library>>,
    pub lib_cache: Option<Shared<LibraryCache>>,
}

/// Builder functions
impl EvalContext {
    pub fn new() -> Self {
        let builtins = BuiltinRegistry::new();

        Self {
            builtins,
            ..Default::default()
        }
    }

    /// Add a library to the context.
    ///
    /// This is needed when you want to evaluate other symbols too.
    pub fn with_lib(mut self, lib: Shared<Library>) -> Self {
        self.lib = Some(lib);
        self
    }

    pub fn with_lib_cache(mut self, lib_cache: Shared<LibraryCache>) -> Self {
        self.lib_cache = Some(lib_cache);
        self
    }
}

/// Library access functions
impl EvalContext {
    /// The library currently being evaluated (`SymbolId::Item`).
    pub fn library(&self) -> Option<Shared<Library>> {
        self.lib.clone()
    }

    /// An external library (`SymbolId::External`).
    pub fn external_library(&self, lib_id: &LibraryId) -> Option<Shared<Library>> {
        self.lib_cache.as_ref()?.read_unwrap().get(lib_id)
    }

    /// Evaluate the symbol `id` in library `lib`.
    ///
    /// The library is passed as owned [`Shared`], so its read lock does not borrow `self`.
    pub(crate) fn eval_symbol_node(
        &mut self,
        lib: Option<Shared<Library>>,
        id: SymbolNodeId,
    ) -> EvalResult {
        let Some(lib) = lib else {
            todo!("Error handling: Library not loaded")
        };
        let lib = lib.read_unwrap();
        SymbolNodeRef::new(id, &lib.arena).get().def.eval(self)
    }
}

/// Scope access functions
impl EvalContext {
    pub fn scope<T>(&mut self, frame: impl Into<StackFrame>, f: impl FnOnce(&mut Self) -> T) -> T {
        // 1. Temporarily swap out the stack to avoid self-borrow issues
        let mut stack = std::mem::take(&mut self.stack);
        stack.push(frame);
        self.stack = stack;

        // 2. Define a guard that pops the frame on Drop
        struct PopGuard<'a>(&'a mut EvalContext);

        impl<'a> Drop for PopGuard<'a> {
            fn drop(&mut self) {
                self.0.stack.pop();
            }
        }

        // 3. Instantiate the guard
        let _guard = PopGuard(self);

        // 4. Run the closure. When `_guard` goes out of scope right after this,
        // it will execute `self.stack.pop()` even if `f` panics or short-circuits.
        f(_guard.0)
    }

    pub fn top(&self) -> &StackFrame {
        self.stack.top()
    }

    pub fn top_mut(&mut self) -> &mut StackFrame {
        self.stack.top_mut()
    }

    pub fn issues(self) -> IssueList<EvalIssue> {
        self.issues
    }
}

impl LookUpName for EvalContext {
    fn look_up_built_in_name(&self, builtin_id: &microcad_builtin::BuiltinId) -> Option<Name> {
        self.builtins.look_up_built_in_name(builtin_id)
    }
}

impl ContextScope for EvalContext {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.stack.look_up_local(name)
    }

    fn current_symbol_name(&self) -> Option<String> {
        self.stack.current_symbol_name()
    }
}

impl StackRead for EvalContext {
    type Frame = StackFrame;

    fn get_local(&self, name: &Name) -> Option<&Value> {
        self.stack.get_local(name)
    }

    fn top(&self) -> &Self::Frame {
        self.stack.top()
    }
}

impl StackWrite for EvalContext {
    fn pop(&mut self) -> Self::Frame {
        self.stack.pop()
    }

    fn push(&mut self, frame: impl Into<Self::Frame>) {
        self.stack.push(frame);
    }

    fn top_mut(&mut self) -> &mut Self::Frame {
        self.stack.top_mut()
    }
}

impl ModelTreeBuilderMut for EvalContext {
    fn model_tree_builder_mut(&mut self) -> &mut microcad_lang_types::model::ModelTreeBuilder {
        self.top_mut().model_tree_builder_mut()
    }
}

impl PushIssue<EvalIssue> for EvalContext {
    fn push_issue(&mut self, issue: impl Into<EvalIssue>) {
        self.issues.push_issue(issue);
    }

    fn push_err(&mut self, err: impl Into<EvalError>) {
        self.issues.push_err(err)
    }

    fn push_warn(&mut self, warn: impl Into<EvalWarning>) {
        self.issues.push_warn(warn)
    }

    fn push_info(&mut self, info: impl Into<EvalInfo>) {
        self.issues.push_info(info)
    }
}

impl SrcReferrer for EvalIssue {
    fn src_ref(&self) -> SrcRef {
        match self {
            Self::Err(err) => err.src_ref(),
            _ => SrcRef::none(),
        }
    }
}
