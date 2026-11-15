// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resolve locals.

use crate::{
    ResolveErrorKind, ResolveLibraryContext, ResolveWarning, SymbolNodeRef,
    library::{LookUp, Symbol, SymbolArena, SymbolNodeId, symbol, visitor},
    resolve::context::ExternalResolver,
    resolve::stack::{self, LocalTable, ResolveStack, ResolveStackFrame, ScopeAccess, SymbolFrame},
};
use mucad_lang_base::{
    Identifier, Name, PushIssue, SingleIdentifier, SrcReferrer, Stability, SymbolId,
    ToCompactString,
};
use mucad_lang_lower::ir::UnresolvedPath;

/// A symbol a path stands for, with the stability of the symbol or of an alias on the way to
/// it, if one of them is deprecated, see [`crate::Library::deprecation`].
type Found = (SymbolId, Option<Stability>);

pub struct ResolveVisitor<'a, 'lib, 'ctx> {
    stack: ResolveStack,
    lib_ctx: &'a mut ResolveLibraryContext<'lib, 'ctx>,
}

impl<'a, 'lib, 'ctx> ResolveVisitor<'a, 'lib, 'ctx> {
    /// Declare the property `name` of a source or workbench, e.g. `a = 5;`.
    ///
    /// The properties of a model have unique names, so a second assignment of a name in the same
    /// scope is an error. Functions have local variables instead, which can be assigned again.
    ///
    /// Only private properties are reported, if they are unused, see [`Self::declare`].
    fn declare_property(&mut self, name: Identifier, report_unused: bool) {
        if self
            .stack
            .top()
            .local_table()
            .is_some_and(|locals| locals.exists(&name))
        {
            self.lib_ctx
                .push_err(ResolveErrorKind::PropertyAlreadyDefined {
                    name: name.to_string(),
                    src_ref: name.src_ref(),
                });
        }
        self.declare(name, report_unused);
    }

    /// Declare the local `id` in the current frame, which is reported, if it is unused and
    /// `report_unused`, e.g. not for parameters of a workbench, which are properties of its model.
    fn declare(&mut self, id: Identifier, report_unused: bool) {
        let replaced = self
            .stack
            .top_mut()
            .local_table_mut()
            .and_then(|locals| locals.declare(id, report_unused));
        // A local of a function, which is assigned again before its value is used.
        if let Some(id) = replaced {
            self.warn_unused(id);
        }
    }

    fn warn_unused(&mut self, id: Identifier) {
        self.lib_ctx.push_warn(ResolveWarning::UnusedLocal {
            src_ref: id.src_ref(),
            name: id,
        });
    }

    pub fn new(lib_ctx: &'a mut ResolveLibraryContext<'lib, 'ctx>) -> Self {
        Self {
            stack: ResolveStack::new(),
            lib_ctx,
        }
    }

    pub fn transform<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut ResolveVisitor<'a, 'lib, 'ctx>, SymbolNodeId, &mut Symbol),
    {
        let lib = &mut *self.lib_ctx.lib_mut();
        let node_ids: Vec<SymbolNodeId> = lib.root.descendants(&lib.arena).collect();
        let arena_ptr = &mut lib.arena as *mut SymbolArena;
        node_ids.into_iter().for_each(|id| {
            // 1. Mutable reference to payload
            let payload = unsafe { (&mut *arena_ptr)[id].get_mut() };

            // 2. Both are passed together safely
            f(self, id, payload);
        });
    }

    pub fn visit(&mut self) {
        use crate::library::visitor::VisitorMut;

        // Imports first, because looking up other paths follows aliases
        // and searches the modules imported by wildcards.
        let is_import = |symbol: &Symbol| {
            matches!(
                symbol.def,
                symbol::SymbolDef::Alias(_) | symbol::SymbolDef::Wildcard(_)
            )
        };
        for imports in [true, false] {
            self.transform(|visitor, id, symbol| {
                if is_import(symbol) == imports {
                    visitor.scope(SymbolFrame::new(id), |visitor| {
                        visitor.visit_meta(&mut symbol.meta);
                        visitor.visit_def(&mut symbol.def);
                    })
                }
            });
        }
    }

    pub fn scope<T>(
        &mut self,
        frame: impl Into<ResolveStackFrame>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        // 1. Temporarily swap out the stack to avoid self-borrow issues
        let mut stack = std::mem::take(&mut self.stack);
        stack.push(frame);
        self.stack = stack;

        // 2. Define a guard that pops the frame on Drop
        struct PopGuard<'b, 'a, 'ctx, 'lib>(&'b mut ResolveVisitor<'a, 'lib, 'ctx>);

        impl<'b, 'a, 'ctx, 'lib> Drop for PopGuard<'b, 'a, 'ctx, 'lib> {
            fn drop(&mut self) {
                // The locals of the frame are out of scope, so the unused ones are reported.
                let frame = self.0.stack.pop();
                let unused: Vec<_> = frame.unused_locals().into_iter().cloned().collect();
                unused.into_iter().for_each(|id| self.0.warn_unused(id));
            }
        }

        // 3. Instantiate the guard
        let _guard = PopGuard(self);

        // 4. Run the closure. When `_guard` goes out of scope right after this,
        // it will execute `self.stack.pop()` even if `f` panics or short-circuits.
        f(_guard.0)
    }
}

impl<'a, 'lib, 'ctx> ScopeAccess for ResolveVisitor<'a, 'lib, 'ctx> {
    fn local_table(&self) -> Option<&LocalTable> {
        self.stack.local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.stack.local_table_mut()
    }

    fn declare_local(&mut self, id: Identifier) {
        self.declare(id, true);
    }

    fn use_local(&mut self, id: &Identifier) -> bool {
        let found = self.stack.traversal_mut(|frame| {
            if let Some(locals) = frame.local_table_mut()
                && locals.use_local(id)
            {
                // We found and recorded the usage, stop traversing!
                return std::ops::ControlFlow::Break(());
            }
            // Not in this frame, keep looking downwards
            std::ops::ControlFlow::Continue(())
        });

        // If traversal_mut returned Some(()), it means we broke early (found it).
        found.is_some()
    }

    fn unused_locals(&self) -> Vec<&Identifier> {
        self.stack.unused_locals()
    }

    fn symbol_node_id(&self) -> Option<SymbolNodeId> {
        self.stack.symbol_node_id()
    }
}

impl<'a, 'lib, 'ctx> visitor::VisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_symbol<'b>(&mut self, node: SymbolNodeRef<'b>, symbol: &mut Symbol) {
        self.scope(SymbolFrame::new(node.id), |visitor| {
            visitor.visit_meta(&mut symbol.meta);
            visitor.visit_def(&mut symbol.def);
        })
    }
}

impl<'a, 'lib, 'ctx> visitor::LeafVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    /// Locals shadow symbols.
    fn visit_path(&mut self, path: &mut symbol::Path) {
        if !self.resolve_local(path) {
            self.resolve_symbol(path);
        }
    }

    /// Locals are values, which cannot be called, so they do not shadow a called symbol,
    /// e.g. the operation `center` in `c.center()` with a local `center`.
    fn visit_callee(&mut self, path: &mut symbol::Path) {
        self.resolve_symbol(path);
    }
}

impl<'a, 'lib, 'ctx> ResolveVisitor<'a, 'lib, 'ctx> {
    /// Resolve `path` to a local, e.g. `x` in `x + 1`, returns `true` if it is a local.
    fn resolve_local(&mut self, path: &mut symbol::Path) -> bool {
        let symbol::Path::Unresolved(unresolved_path) = path else {
            return false;
        };
        match unresolved_path.single_identifier() {
            Some(id) if self.stack.is_local(id) => {
                self.use_local(id);
                *path = symbol::Path::Resolved(SymbolId::Local(id.to_compact_string()));
                true
            }
            _ => false,
        }
    }

    /// Resolve `path` to a symbol of this library, of an external library or of the prelude.
    ///
    /// The use of a deprecated symbol is reported, also of a deprecated module in the path.
    fn resolve_symbol(&mut self, path: &mut symbol::Path) {
        let symbol::Path::Unresolved(unresolved_path) = path else {
            return;
        };

        // Known built-ins are resolved while lowering, unknown ones have been reported there.
        if unresolved_path.builtin_id().is_some() {
            return;
        }

        let (mut found, missing_library) = self.look_up(unresolved_path);
        if let Some(name) = missing_library {
            self.lib_ctx.push_err(ResolveErrorKind::LibraryNotLoaded {
                name: name.to_string(),
            });
        }

        // Unresolved and ambiguous paths stay unresolved, evaluation reports them again.
        match found.len() {
            0 => self.lib_ctx.push_err(ResolveErrorKind::UnresolvedPath {
                path: unresolved_path.to_string(),
                src_ref: unresolved_path.src_ref,
            }),
            1 => {
                let unresolved_path = unresolved_path.clone();
                let (symbol_id, deprecation) = found.remove(0);
                *path = symbol::Path::Resolved(symbol_id);
                self.warn_deprecated(&unresolved_path, deprecation);
            }
            _ => self.lib_ctx.push_err(ResolveErrorKind::AmbiguousPath {
                path: unresolved_path.to_string(),
                candidates: found
                    .into_iter()
                    .map(|(id, _)| symbol::Path::Resolved(id).to_string())
                    .collect(),
                src_ref: unresolved_path.src_ref,
            }),
        }
    }

    /// The symbols `path` stands for: of this library, of an external library or, without
    /// both, of the prelude, and the name of an external library of the path, which is not
    /// loaded.
    fn look_up(&self, path: &UnresolvedPath) -> (Vec<Found>, Option<Name>) {
        let mut found = Vec::new();
        let mut missing_library = None;

        let external = ExternalResolver::new(self.lib_ctx.lib_cache());

        // Look up in current library being resolved, including glob imports of other libraries.
        if let Some(symbol_node_id) = self.stack.symbol_node_id() {
            let lib = self.lib_ctx.lib();
            if let Some(look_up) = lib.look_up_symbol(symbol_node_id, path, &external) {
                found.push(self.found(lib, look_up, None));
            }
        }

        // Look up external libraries.
        if !path.is_absolute
            && let Some(external_name) = path.external_name()
            && let Some(dep) = self.lib_ctx.lib().dependencies.get(&external_name)
        {
            let lib = self.lib_ctx.lib_cache().read_unwrap().get(dep);
            match lib {
                Some(lib) => {
                    // The library appears in its own tree under its name, so only `mu` is removed.
                    let path = path.without_mu_prefix();
                    if let Some(look_up) = lib.look_up_symbol(lib.root, &path, &external) {
                        found.push(self.found(&lib, look_up, Some(lib.id())));
                    }
                }
                None => missing_library = Some(external_name),
            }
        }

        // The prelude has the lowest priority, e.g. `Circle` for `std::geo2d::Circle`.
        if found.is_empty()
            && let Some(id) = path.single_identifier()
            && let Some(symbol_id) = self.lib_ctx.prelude_symbol(id)
        {
            let deprecation = self.deprecation(&symbol_id);
            found.push((symbol_id, deprecation));
        }

        (found, missing_library)
    }

    /// The symbol found in `lib` with its deprecation, also of an alias on the way to it.
    ///
    /// `lib_id` is the ID of `lib`, if it is another library than the one being resolved.
    fn found(
        &self,
        lib: &crate::Library,
        look_up: LookUp,
        lib_id: Option<mucad_lang_base::LibraryId>,
    ) -> Found {
        let alias_deprecation = match &look_up {
            LookUp::Local(node_id) => lib.deprecation(*node_id).cloned(),
            LookUp::External(_) => None,
        };
        let symbol_id = look_up.symbol_id(lib, lib_id);
        let deprecation = alias_deprecation.or_else(|| self.deprecation(&symbol_id));
        (symbol_id, deprecation)
    }

    /// The stability of the symbol `id` of this or another library, if it is deprecated.
    fn deprecation(&self, id: &SymbolId) -> Option<Stability> {
        match id {
            SymbolId::Item(node_id) => self.lib_ctx.lib().deprecation(*node_id).cloned(),
            SymbolId::External(external_id) => {
                let lib = self
                    .lib_ctx
                    .lib_cache()
                    .read_unwrap()
                    .get(&external_id.lib_id)?;
                lib.deprecation(external_id.id).cloned()
            }
            SymbolId::Builtin(_) | SymbolId::Local(_) => None,
        }
    }

    /// Report the use of a deprecated symbol by `path` with its `deprecation`, and of a
    /// deprecated module in it, e.g. of `old` in `old::A`.
    ///
    /// Like in Rust, deprecated symbols used inside a deprecated symbol are not reported.
    fn warn_deprecated(&mut self, path: &UnresolvedPath, deprecation: Option<Stability>) {
        if self.in_deprecated_symbol() {
            return;
        }
        // The modules in the path, e.g. `old` of `old::A`, reported at their name.
        for len in 1..path.parts.len() {
            let module = UnresolvedPath {
                is_absolute: path.is_absolute,
                parts: path.parts[..len].into(),
                src_ref: path.parts[len - 1].src_ref(),
            };
            if let (found, _) = self.look_up(&module)
                && let [(_, Some(stability))] = found.as_slice()
            {
                self.push_deprecated(&module, stability);
            }
        }
        if let Some(stability) = deprecation {
            self.push_deprecated(path, &stability);
        }
    }

    /// Report the use of the deprecated symbol `path`.
    ///
    /// A path visited again, e.g. a default value of a parameter, which is also one of the
    /// implicit init, is reported once, see [`mucad_lang_base::IssueList`].
    fn push_deprecated(&mut self, path: &UnresolvedPath, stability: &Stability) {
        self.lib_ctx.push_warn(ResolveWarning::Deprecated {
            path: path.to_string(),
            deprecation: stability.deprecation().unwrap_or_default(),
            note: stability.deprecation_note().map(str::to_string),
            src_ref: path.src_ref,
        });
    }

    /// Whether the symbol being resolved or a module around it is deprecated.
    fn in_deprecated_symbol(&self) -> bool {
        let Some(node_id) = self.stack.symbol_node_id() else {
            return false;
        };
        let lib = self.lib_ctx.lib();
        let node = SymbolNodeRef::new(node_id, &lib.arena);
        std::iter::once(node)
            .chain(node.ancestors())
            .any(|node| node.get().ver.stability.is_deprecated())
    }
}

impl<'a, 'lib, 'ctx> visitor::ConstantVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {}
impl<'a, 'lib, 'ctx> visitor::WorkbenchVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_workbench(&mut self, workbench: &mut mucad_lang_lower::ir::workbench::Workbench) {
        use mucad_lang_lower::ir::visitor::WorkbenchExpressionVisitorMut;

        // Workbench parameters are locals of the workbench body.
        self.scope(stack::WorkbenchFrame::default(), |visitor| {
            visitor.visit_workbench_signature(&mut workbench.signature);

            workbench
                .statements
                .iter_mut()
                .for_each(|stmt| visitor.visit_workbench_statement(stmt))
        })
    }

    fn visit_workbench_signature(
        &mut self,
        signature: &mut mucad_lang_lower::ir::workbench::WorkbenchSignature,
    ) {
        use mucad_lang_lower::ir::visitor::ConstantVisitorMut;

        self.visit_parameter_list(&mut signature.parameters);
        // Workbench parameters are properties of its model, which can be used from outside.
        signature
            .parameters
            .iter()
            .for_each(|param| self.declare(param.id.clone(), false));

        signature
            .inits
            .iter_mut()
            .for_each(|init| self.visit_workbench_init(init));
    }

    fn visit_workbench_init(&mut self, init: &mut mucad_lang_lower::ir::workbench::Init) {
        use mucad_lang_lower::ir::visitor::{
            ConstantVisitorMut, LeafVisitorMut, WorkbenchExpressionVisitorMut,
        };

        self.visit_workbench_init_attr(&mut init.attr);
        self.visit_parameter_list(&mut init.parameters);

        // Init parameters with the name of a workbench parameter set it,
        // e.g. `center` in `sketch Circle(radius: Length, center: Vec2) { init(diameter: Length, center: Vec2) { … } }`.
        let sets_parameter: Vec<_> = init
            .parameters
            .iter()
            .map(|param| self.stack.is_local(&param.id))
            .collect();

        // Init parameters are only visible inside the init.
        self.scope(stack::WorkbenchGroupFrame::default(), |visitor| {
            init.parameters
                .iter()
                .zip(sets_parameter)
                .for_each(|(param, sets_parameter)| {
                    visitor.declare(param.id.clone(), !sets_parameter)
                });

            init.statements.iter_mut().for_each(|init_statement| {
                visitor.visit_name(&mut init_statement.name);
                visitor.visit_workbench_expr(&mut init_statement.expression);
            });
        });
    }

    fn visit_workbench_init_attr(
        &mut self,
        _attr: &mut mucad_lang_lower::ir::workbench::InitAttributes,
    ) {
    }
}
impl<'a, 'lib, 'ctx> visitor::WorkbenchExpressionVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_workbench_statement(
        &mut self,
        statement: &mut mucad_lang_lower::ir::workbench::WorkbenchStatement,
    ) {
        use mucad_lang_lower::ir::visitor::LeafVisitorMut;

        use mucad_lang_lower::ir::WorkbenchStatement;

        self.visit_model_attributes(statement.attr_mut());
        match statement {
            WorkbenchStatement::Expression(_) => {}
            WorkbenchStatement::Prop(prop) => self.visit_name(&mut prop.name),
            WorkbenchStatement::Assignment(assignment) => self.visit_name(&mut assignment.name),
        }
        self.visit_workbench_expr(statement.expression_mut());
        // Properties (`prop`) can be used from outside, assignments are hidden.
        match statement {
            WorkbenchStatement::Expression(_) => {}
            WorkbenchStatement::Prop(prop) => self.declare_property(prop.name.clone(), false),
            WorkbenchStatement::Assignment(assignment) => {
                self.declare_property(assignment.name.clone(), true)
            }
        }
    }

    fn visit_workbench_group(&mut self, group: &mut mucad_lang_lower::ir::workbench::Group) {
        self.visit_model_attributes(&mut group.attr);
        self.scope(stack::WorkbenchGroupFrame::default(), |visitor| {
            group
                .statements
                .iter_mut()
                .for_each(|statement| visitor.visit_workbench_statement(statement));
        })
    }
}

impl<'a, 'lib, 'ctx> visitor::SourceVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_source(&mut self, source: &mut mucad_lang_lower::ir::Source) {
        self.scope(stack::SourceFrame::default(), |visitor| {
            source
                .statements
                .iter_mut()
                .for_each(|stmt| visitor.visit_source_statement(stmt));
        })
    }

    fn visit_source_statement(&mut self, statement: &mut symbol::SourceStatement) {
        use mucad_lang_lower::ir::visitor::{LeafVisitorMut, WorkbenchExpressionVisitorMut};
        if let Some(name) = &mut statement.name {
            self.visit_name(name);
        }
        self.visit_model_attributes(&mut statement.attr);
        self.visit_export_attributes(&mut statement.exports);
        self.visit_workbench_expr(&mut statement.expression);
        // Named statements of a source are properties of its model, which can be exported.
        if let Some(name) = &mut statement.name {
            self.declare_property(name.clone(), false);
        }
    }
}

impl<'a, 'lib, 'ctx> visitor::FnVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_fn(&mut self, function: &mut symbol::Function) {
        self.scope(stack::FunctionFrame::default(), |visitor| {
            visitor.visit_fn_signature(&mut function.signature);
            function
                .statements
                .iter_mut()
                .for_each(|stmt| visitor.visit_fn_statement(stmt));
        })
    }

    fn visit_fn_signature(
        &mut self,
        signature: &mut mucad_lang_lower::ir::function::FunctionSignature,
    ) {
        use mucad_lang_lower::ir::visitor::ConstantVisitorMut;
        self.visit_parameter_list(&mut signature.parameters);
        signature
            .parameters
            .iter()
            .for_each(|param| self.declare_local(param.id.clone()));
    }

    fn visit_fn_scope(&mut self, scope: &mut mucad_lang_lower::ir::function::Scope) {
        self.scope(stack::FunctionScopeFrame::default(), |ctx| {
            scope
                .statements
                .iter_mut()
                .for_each(|stmt| ctx.visit_fn_statement(stmt))
        })
    }

    fn visit_fn_local_assignment(
        &mut self,
        local_assignment: &mut mucad_lang_lower::ir::function::FunctionLocalAssignment,
    ) {
        use mucad_lang_lower::ir::visitor::LeafVisitorMut;

        self.visit_name(&mut local_assignment.id);
        self.visit_fn_expr(&mut local_assignment.expression);
        self.declare_local(local_assignment.id.clone());
    }
}
