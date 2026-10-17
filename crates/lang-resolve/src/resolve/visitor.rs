// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resolve locals.

use std::cell::RefCell;

use crate::{
    Library, LookUp, ResolveErrorKind, ResolveLibraryContext, SymbolNodeRef,
    library::{Symbol, SymbolArena, SymbolNodeId, symbol, visitor},
    resolve::context::LibraryCache,
    resolve::stack::{self, LocalTable, ResolveStack, ResolveStackFrame, ScopeAccess, SymbolFrame},
};
use microcad_lang_base::{
    ExternalId, HashSet, Identifier, LibraryId, PushIssue, SharedMut, SingleIdentifier,
    SrcReferrer, SymbolId, ToCompactString,
};

/// The symbol of a look up result in `lib`, see [`Library::symbol_id`].
fn found_symbol(lib: &Library, found: LookUp, lib_id: Option<LibraryId>) -> SymbolId {
    match found {
        LookUp::Local(node_id) => lib.symbol_id(node_id, lib_id),
        LookUp::External(symbol_id) => symbol_id,
    }
}

/// Looks up symbols inside modules of other libraries in the library cache.
///
/// Glob imports of these modules are followed into further libraries, e.g. a module of
/// library `a` which re-exports a module of library `b` with `pub use b::geo::*;`.
struct ExternalResolver {
    lib_cache: SharedMut<LibraryCache>,
    /// The modules being searched, to stop at libraries which import each other.
    searching: RefCell<HashSet<ExternalId>>,
}

impl ExternalResolver {
    fn new(lib_cache: SharedMut<LibraryCache>) -> Self {
        Self {
            lib_cache,
            searching: Default::default(),
        }
    }

    fn look_up(&self, module: &ExternalId, parts: &[Identifier]) -> Option<SymbolId> {
        if !self.searching.borrow_mut().insert(module.clone()) {
            return None;
        }

        let lib = self.lib_cache.read_unwrap().get(&module.lib_id);
        let found = lib.and_then(|lib| {
            lib.look_up_in_module(module.id, parts, &|module, parts| {
                self.look_up(module, parts)
            })
            .map(|found| found_symbol(&lib, found, Some(module.lib_id)))
        });

        self.searching.borrow_mut().remove(module);
        found
    }
}

pub struct ResolveVisitor<'a, 'lib, 'ctx> {
    stack: ResolveStack,
    lib_ctx: &'a mut ResolveLibraryContext<'lib, 'ctx>,
}

impl<'a, 'lib, 'ctx> ResolveVisitor<'a, 'lib, 'ctx> {
    /// Declare the property `name` of a source or workbench, e.g. `a = 5;`.
    ///
    /// The properties of a model have unique names, so a second assignment of a name in the same
    /// scope is an error. Functions have local variables instead, which can be assigned again.
    fn declare_property(&mut self, name: Identifier) {
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
        self.declare_local(name);
    }

    pub fn new(lib_ctx: &'a mut ResolveLibraryContext<'lib, 'ctx>) -> Self {
        Self {
            stack: ResolveStack::default(),
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
                self.0.stack.pop();
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
        if let Some(locals) = self.stack.top_mut().local_table_mut() {
            locals.declare_local(id);
        }
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

    fn unused_locals(&self) -> impl Iterator<Item = &Identifier> {
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
                *path = symbol::Path::Resolved(SymbolId::Local(id.to_compact_string()));
                true
            }
            _ => false,
        }
    }

    /// Resolve `path` to a symbol of this library, of an external library or of the prelude.
    fn resolve_symbol(&mut self, path: &mut symbol::Path) {
        let symbol::Path::Unresolved(unresolved_path) = path else {
            return;
        };

        // Known built-ins are resolved while lowering, unknown ones have been reported there.
        if unresolved_path.builtin_id().is_some() {
            return;
        }

        let mut paths = Vec::new();

        let externals = ExternalResolver::new(self.lib_ctx.lib_cache());
        let external = |module: &ExternalId, parts: &[Identifier]| externals.look_up(module, parts);

        // Look up in current library being resolved, including glob imports of other libraries.
        if let Some(symbol_node_id) = self.stack.symbol_node_id() {
            let lib = self.lib_ctx.lib();
            paths.extend(
                lib.look_up_symbol(symbol_node_id, unresolved_path, &external)
                    .map(|found| symbol::Path::Resolved(found_symbol(lib, found, None))),
            );
        }

        // Look up external libraries.
        if !unresolved_path.is_absolute
            && let Some(external_name) = unresolved_path.external_name()
            && let Some(dep) = self.lib_ctx.lib().dependencies.get(&external_name)
        {
            let lib = self.lib_ctx.lib_cache().read_unwrap().get(dep);
            match lib {
                Some(lib) => {
                    // The library appears in its own tree under its name, so only `mu` is removed.
                    let path = unresolved_path.without_mu_prefix();
                    paths.extend(lib.look_up_symbol(lib.root, &path, &external).map(|found| {
                        symbol::Path::Resolved(found_symbol(&lib, found, Some(lib.id())))
                    }));
                }
                None => self.lib_ctx.push_err(ResolveErrorKind::LibraryNotLoaded {
                    name: external_name.to_string(),
                }),
            }
        }

        // The prelude has the lowest priority, e.g. `Circle` for `std::geo2d::Circle`.
        if paths.is_empty()
            && let Some(id) = unresolved_path.single_identifier()
            && let Some(symbol_id) = self.lib_ctx.prelude_symbol(id)
        {
            paths.push(symbol::Path::Resolved(symbol_id));
        }

        // Unresolved and ambiguous paths stay unresolved, evaluation reports them again.
        match paths.len() {
            0 => self.lib_ctx.push_err(ResolveErrorKind::UnresolvedPath {
                path: unresolved_path.to_string(),
                src_ref: unresolved_path.src_ref,
            }),
            1 => *path = paths.remove(0),
            _ => self.lib_ctx.push_err(ResolveErrorKind::AmbiguousPath {
                path: unresolved_path.to_string(),
                candidates: paths.iter().map(|path| path.to_string()).collect(),
                src_ref: unresolved_path.src_ref,
            }),
        }
    }
}

impl<'a, 'lib, 'ctx> visitor::ConstantVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {}
impl<'a, 'lib, 'ctx> visitor::WorkbenchVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_workbench(&mut self, workbench: &mut microcad_lang_lower::ir::workbench::Workbench) {
        use microcad_lang_lower::ir::visitor::WorkbenchExpressionVisitorMut;

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
        signature: &mut microcad_lang_lower::ir::workbench::WorkbenchSignature,
    ) {
        use microcad_lang_lower::ir::visitor::ConstantVisitorMut;

        self.visit_parameter_list(&mut signature.parameters);
        signature
            .parameters
            .iter()
            .for_each(|param| self.declare_local(param.id.clone()));

        signature
            .inits
            .iter_mut()
            .for_each(|init| self.visit_workbench_init(init));
    }

    fn visit_workbench_init(&mut self, init: &mut microcad_lang_lower::ir::workbench::Init) {
        use microcad_lang_lower::ir::visitor::{
            ConstantVisitorMut, LeafVisitorMut, WorkbenchExpressionVisitorMut,
        };

        self.visit_workbench_init_attr(&mut init.attr);
        self.visit_parameter_list(&mut init.parameters);

        // Init parameters are only visible inside the init.
        self.scope(stack::WorkbenchGroupFrame::default(), |visitor| {
            init.parameters
                .iter()
                .for_each(|param| visitor.declare_local(param.id.clone()));

            init.statements.iter_mut().for_each(|init_statement| {
                visitor.visit_name(&mut init_statement.name);
                visitor.visit_workbench_expr(&mut init_statement.expression);
            });
        });
    }

    fn visit_workbench_init_attr(
        &mut self,
        _attr: &mut microcad_lang_lower::ir::workbench::InitAttributes,
    ) {
    }
}
impl<'a, 'lib, 'ctx> visitor::WorkbenchExpressionVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_workbench_statement(
        &mut self,
        statement: &mut microcad_lang_lower::ir::workbench::WorkbenchStatement,
    ) {
        use microcad_lang_lower::ir::visitor::LeafVisitorMut;

        self.visit_model_attributes(&mut statement.attr);
        if let Some(name) = &mut statement.name {
            self.visit_name(name);
        }
        self.visit_workbench_expr(&mut statement.expression);
        if let Some(name) = &mut statement.name {
            self.declare_property(name.clone());
        }
    }

    fn visit_workbench_group(&mut self, group: &mut microcad_lang_lower::ir::workbench::Group) {
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
    fn visit_source(&mut self, source: &mut microcad_lang_lower::ir::Source) {
        self.scope(stack::SourceFrame::default(), |visitor| {
            source
                .statements
                .iter_mut()
                .for_each(|stmt| visitor.visit_source_statement(stmt));
            // TODO Output unused locals
        })
    }

    fn visit_source_statement(&mut self, statement: &mut symbol::SourceStatement) {
        use microcad_lang_lower::ir::visitor::{LeafVisitorMut, WorkbenchExpressionVisitorMut};
        if let Some(name) = &mut statement.name {
            self.visit_name(name);
        }
        self.visit_model_attributes(&mut statement.attr);
        self.visit_workbench_expr(&mut statement.expression);
        if let Some(name) = &mut statement.name {
            self.declare_property(name.clone());
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

            // TODO: Output unused locals here.
        })
    }

    fn visit_fn_signature(
        &mut self,
        signature: &mut microcad_lang_lower::ir::function::FunctionSignature,
    ) {
        use microcad_lang_lower::ir::visitor::ConstantVisitorMut;
        self.visit_parameter_list(&mut signature.parameters);
        signature
            .parameters
            .iter()
            .for_each(|param| self.declare_local(param.id.clone()));
    }

    fn visit_fn_scope(&mut self, scope: &mut microcad_lang_lower::ir::function::Scope) {
        self.scope(stack::FunctionScopeFrame::default(), |ctx| {
            scope
                .statements
                .iter_mut()
                .for_each(|stmt| ctx.visit_fn_statement(stmt))

            // TODO: Output unused locals here.
        })
    }

    fn visit_fn_local_assignment(
        &mut self,
        local_assignment: &mut microcad_lang_lower::ir::function::FunctionLocalAssignment,
    ) {
        use microcad_lang_lower::ir::visitor::LeafVisitorMut;

        self.visit_name(&mut local_assignment.id);
        self.visit_fn_expr(&mut local_assignment.expression);
        self.declare_local(local_assignment.id.clone());
    }
}
