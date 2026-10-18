// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod stack;

use microcad_lang_resolve::{
    Library, LibraryCache,
    library::symbol::{self, SymbolNodeId, SymbolNodeRef},
};
pub use stack::*;

use microcad_builtin::{BuiltinEvalContext, BuiltinLibrary};
use microcad_lang_base::{
    ExternalId, IssueList, LibraryId, LookUpName, Name, PushIssue, Shared, SharedMut, SrcRef,
    SymbolId, ToCompactString,
};
use microcad_lang_types::{ArgumentValueList, Value, model::ModelTreeBuilderMut};

use crate::{
    Callable, Eval, EvalAdvice, EvalError, EvalErrorKind, EvalIssue, EvalResult, EvalWarning,
};

#[derive(Debug)]
pub struct EvalContext {
    stack: Stack,

    issues: IssueList<EvalIssue>,

    builtins: Shared<BuiltinLibrary>,

    /// ID of the library being evaluated, which is stored in `lib_cache`.
    lib_id: Option<LibraryId>,

    /// All libraries, including the one being evaluated.
    lib_cache: SharedMut<LibraryCache>,
}

/// Builder functions
impl EvalContext {
    /// Create an `EvalContext` with default settings and standard built-ins (`BuiltinRegistry::new()`).
    pub fn new() -> Self {
        EvalContextBuilder::new().build()
    }

    /// Create a builder for customized evaluation context setup.
    pub fn builder() -> EvalContextBuilder {
        EvalContextBuilder::new()
    }
}

/// Accessors
impl EvalContext {
    pub fn builtins(&self) -> &BuiltinLibrary {
        &self.builtins
    }

    pub fn lib_id(&self) -> Option<&LibraryId> {
        self.lib_id.as_ref()
    }

    /// A library by its ID.
    pub fn library(&self, lib_id: &LibraryId) -> Option<Shared<Library>> {
        self.lib_cache.with_read(|lib_cache| lib_cache.get(lib_id))
    }
}

/// Eval methods
impl EvalContext {
    /// Evaluate the main source of the library being evaluated.
    pub fn eval_library(&mut self) -> EvalResult {
        let Some(lib_id) = self.lib_id else {
            return self.catch(EvalErrorKind::NoLibraryToEvaluate);
        };
        let Some(lib) = self.library(&lib_id) else {
            return self.catch(EvalErrorKind::LibraryNotLoaded { lib_id });
        };
        let root = lib.root();

        // A single source file is loaded as child of an empty library root.
        let main = match &root.get().def {
            symbol::SymbolDef::Root(None) => root
                .children()
                .find(|child| matches!(child.get().def, symbol::SymbolDef::SourceFile(_)))
                .unwrap_or(root),
            _ => root,
        };
        main.get().eval(self)
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
        self.with_library(lib_id, |context| {
            SymbolNodeRef::new(id, &lib.arena)
                .get()
                .def
                .call(args, context)
        })
    }

    /// Run `f` with `lib_id` as current library.
    ///
    /// Symbols of a library (e.g. `SymbolId::Item`) refer to that library's tree,
    /// so it becomes the current library while one of its symbols is evaluated.
    fn with_library<T>(&mut self, lib_id: LibraryId, f: impl FnOnce(&mut Self) -> T) -> T {
        let previous = self.lib_id.replace(lib_id);
        let result = f(self);
        self.lib_id = previous;
        result
    }

    /// Call the workbench or function `id` in library `lib_id`.
    pub(crate) fn call_symbol_workbench(
        &mut self,
        call: &symbol::workbench::WorkbenchCall,
        lib_id: LibraryId,
        id: SymbolNodeId,
    ) -> EvalResult {
        let args = call.args.eval(self)?;
        self.scope(
            SymbolCallFrame {
                path: call.path.clone(),
                src_ref: call.src_ref,
            },
            |context| context.call_symbol(lib_id, id, &args),
        )
    }

    /// Evaluate the symbol `id` in library `lib_id`.
    ///
    /// The library's read lock is held during the evaluation.
    pub(crate) fn eval_symbol(&mut self, lib_id: LibraryId, id: SymbolNodeId) -> EvalResult {
        let Some(lib) = self.library(&lib_id) else {
            return self.catch(EvalErrorKind::LibraryNotLoaded { lib_id });
        };
        self.with_library(lib_id, |context| {
            SymbolNodeRef::new(id, &lib.arena).get().eval(context)
        })
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

    fn look_up_item_name(&self, item_id: &SymbolNodeId) -> Option<Name> {
        self.symbol_name(&SymbolId::Item(*item_id))
    }

    fn look_up_external_item_name(&self, external_id: &ExternalId) -> Option<Name> {
        self.symbol_name(&SymbolId::External(external_id.clone()))
    }
}

/// Symbol names
impl EvalContext {
    /// The full name of a symbol, e.g. `std::geo2d::Circle` or `__mu::debug::assert`.
    ///
    /// Items refer to the current library.
    pub fn symbol_name(&self, id: &SymbolId) -> Option<Name> {
        use microcad_lang_resolve::SymbolNodeExt;

        let (lib_id, node_id) = match id {
            SymbolId::Builtin(builtin_id) => {
                return self.builtins.look_up_built_in_name(builtin_id);
            }
            SymbolId::Local(name) => return Some(name.clone()),
            SymbolId::Item(node_id) => (*self.lib_id.as_ref()?, *node_id),
            SymbolId::External(external_id) => (external_id.lib_id, external_id.id),
        };
        let lib = self.library(&lib_id)?;
        let path = SymbolNodeRef::new(node_id, &lib.arena).abs_path();
        Some(path.to_compact_string())
    }
}

impl ContextScope for EvalContext {
    fn look_up_local(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.stack.look_up_local(name)
    }

    /// The full name of the called symbol, e.g. `std::geo2d::Circle`.
    fn current_symbol_name(&self) -> Option<String> {
        self.stack
            .current_call_scope()
            .find_map(|frame| match frame {
                StackFrame::Call(call) => Some(
                    call.path
                        .symbol_id()
                        .and_then(|id| self.symbol_name(id))
                        .map(|name| name.to_string())
                        .unwrap_or_else(|| call.path.to_string()),
                ),
                frame => frame.current_symbol_name(),
            })
    }

    fn current_symbol_src_ref(&self) -> SrcRef {
        self.stack.current_symbol_src_ref()
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

/// Built-in calls
impl EvalContext {
    /// Run `f` with a context for a built-in called at `src_ref`
    /// and keep the issues the built-in reports.
    pub(crate) fn with_builtin_context<T>(
        &mut self,
        src_ref: SrcRef,
        f: impl FnOnce(&mut BuiltinEvalContext) -> T,
    ) -> T {
        let mut builtin_context = BuiltinEvalContext::from(src_ref);
        let result = f(&mut builtin_context);
        self.append_issues(builtin_context.issues);
        result
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

    fn push_advice(&mut self, advice: impl Into<EvalAdvice>) {
        self.issues.push_advice(advice)
    }
}

/// Builder for constructing an [`EvalContext`].
#[derive(Debug, Default)]
pub struct EvalContextBuilder {
    builtins: Option<Shared<BuiltinLibrary>>,
    lib_id: Option<LibraryId>,
    lib_cache: Option<SharedMut<LibraryCache>>,
}

impl EvalContextBuilder {
    /// Create a new `EvalContextBuilder` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a pre-existing or shared `BuiltinRegistry`.
    pub fn with_builtins(mut self, builtins: Shared<BuiltinLibrary>) -> Self {
        self.builtins = Some(builtins);
        self
    }

    /// Set the current active library ID being evaluated.
    pub fn with_lib_id(mut self, lib_id: LibraryId) -> Self {
        self.lib_id = Some(lib_id);
        self
    }

    /// Attach a shared, mutable library cache.
    pub fn with_lib_cache(mut self, lib_cache: SharedMut<LibraryCache>) -> Self {
        self.lib_cache = Some(lib_cache);
        self
    }

    /// Build and initialize the [`EvalContext`].
    pub fn build(self) -> EvalContext {
        EvalContext {
            stack: Stack::default(),
            issues: IssueList::default(),
            builtins: self
                .builtins
                .unwrap_or_else(|| Shared::new(BuiltinLibrary::new())),
            lib_id: self.lib_id,
            lib_cache: self.lib_cache.unwrap_or_default(),
        }
    }
}
