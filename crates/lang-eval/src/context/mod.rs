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
    IssueList, LibraryId, LookUpName, Name, PushIssue, SharedMut, SrcRef, SrcReferrer,
};
use microcad_lang_types::{ArgumentValueList, Value, model::ModelTreeBuilderMut};

use crate::{
    Callable, Eval, EvalError, EvalErrorKind, EvalInfo, EvalIssue, EvalResult, EvalWarning,
};

#[derive(Debug, Default)]
pub struct EvalContext {
    stack: Stack,

    issues: IssueList<EvalIssue>,

    pub builtins: BuiltinRegistry,

    /// ID of the library being evaluated, which is stored in `lib_cache`.
    pub lib_id: Option<LibraryId>,

    /// All libraries, including the one being evaluated.
    pub lib_cache: SharedMut<LibraryCache>,
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

    /// Set the library to be evaluated.
    ///
    /// The library must be stored in the library cache.
    pub fn with_lib_id(mut self, lib_id: LibraryId) -> Self {
        self.lib_id = Some(lib_id);
        self
    }

    pub fn with_lib_cache(mut self, lib_cache: SharedMut<LibraryCache>) -> Self {
        self.lib_cache = lib_cache;
        self
    }
}

/// Library access functions
impl EvalContext {
    /// A library by its ID.
    pub fn library(&self, lib_id: &LibraryId) -> Option<SharedMut<Library>> {
        self.lib_cache.read_unwrap().get(lib_id)
    }

    /// Evaluate the root symbol of the library being evaluated.
    pub fn eval_library(&mut self) -> EvalResult {
        let Some(lib_id) = self.lib_id else {
            return self.catch(EvalErrorKind::NoLibraryToEvaluate);
        };
        let Some(lib) = self.library(&lib_id) else {
            return self.catch(EvalErrorKind::LibraryNotLoaded { lib_id });
        };
        let lib = lib.read_unwrap();
        lib.root().get().eval(self)
    }

    /// Call the symbol `id` in library `lib_id` (e.g. a workbench).
    ///
    /// The library's read lock is held during the call.
    pub(crate) fn call_symbol(
        &mut self,
        lib_id: LibraryId,
        id: SymbolNodeId,
        args: &ArgumentValueList,
    ) -> EvalResult {
        let Some(lib) = self.library(&lib_id) else {
            return self.catch(EvalErrorKind::LibraryNotLoaded { lib_id });
        };
        let lib = lib.read_unwrap();
        SymbolNodeRef::new(id, &lib.arena)
            .get()
            .def
            .call(args, self)
    }

    /// Evaluate the symbol `id` in library `lib_id`.
    ///
    /// The library's read lock is held during the evaluation.
    pub(crate) fn eval_symbol(&mut self, lib_id: LibraryId, id: SymbolNodeId) -> EvalResult {
        let Some(lib) = self.library(&lib_id) else {
            return self.catch(EvalErrorKind::LibraryNotLoaded { lib_id });
        };
        let lib = lib.read_unwrap();
        SymbolNodeRef::new(id, &lib.arena).get().eval(self)
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

        impl Drop for PopGuard<'_> {
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
