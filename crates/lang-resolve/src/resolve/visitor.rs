// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resolve locals.

use crate::{
    ResolveLibraryContext, SymbolNodeRef,
    library::{Symbol, SymbolArena, SymbolNodeId, symbol, visitor},
    resolve::stack::{self, LocalTable, ResolveStack, ResolveStackFrame, ScopeAccess, SymbolFrame},
};
use microcad_lang_base::{ExternalId, Identifier, SingleIdentifier, SymbolId, ToCompactString};

pub struct ResolveVisitor<'a, 'lib, 'ctx> {
    stack: ResolveStack,
    lib_ctx: &'a mut ResolveLibraryContext<'lib, 'ctx>,
}

impl<'a, 'lib, 'ctx> ResolveVisitor<'a, 'lib, 'ctx> {
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
        let lib = &mut *self.lib_ctx.lib;
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
        self.transform(|visitor, id, symbol| {
            visitor.scope(SymbolFrame::new(id), |visitor| {
                visitor.visit_meta(&mut symbol.meta);
                visitor.visit_def(&mut symbol.def);
            })
        });
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
    fn visit_path(&mut self, path: &mut symbol::Path) {
        if let symbol::Path::Unresolved(unresolved_path) = path {
            let mut paths = Vec::new();

            // Look up in current library being resolved.
            if let Some(symbol_node_id) = self.stack.symbol_node_id()
                && let Some(node_id) = self.lib_ctx.lib.look_up(symbol_node_id, unresolved_path)
            {
                paths.push(symbol::Path::Resolved(SymbolId::Item(node_id)));
            }

            // Look up external libraries.
            if !unresolved_path.is_absolute
                && let Some(external_name) = unresolved_path.external_name()
                && let Some(dep) = self.lib_ctx.lib.dependencies.get(&external_name)
            {
                let lib = self
                    .lib_ctx
                    .ctx()
                    .lib_cache
                    .read_unwrap()
                    .get(dep)
                    .expect("Library not loaded.");
                let lib = lib.read_unwrap();

                let resolved = lib.look_up(lib.root, unresolved_path).map(|id| {
                    symbol::Path::Resolved(SymbolId::External(ExternalId {
                        lib_id: lib.id(),
                        id,
                    }))
                });

                paths.extend(resolved);
            }

            if let Some(id) = unresolved_path.single_identifier() {
                if self.local_table().unwrap().exists(id) {
                    paths.push(symbol::Path::Resolved(SymbolId::Local(
                        id.to_compact_string(),
                    )));
                } else {
                    todo!("Error handling: Local not in scope {id}")
                }
            }

            match paths.len() {
                0 => todo!("Could not resolve path {unresolved_path}"),
                1 => {
                    *path = paths.first().cloned().unwrap();
                }
                n => {
                    todo!("Ambiguous symbols, found {n} paths");
                }
            }
        }
    }
}

impl<'a, 'lib, 'ctx> visitor::ConstantVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {}
impl<'a, 'lib, 'ctx> visitor::WorkbenchVisitorMut for ResolveVisitor<'a, 'lib, 'ctx> {
    fn visit_workbench(&mut self, workbench: &mut microcad_lang_lower::ir::workbench::Workbench) {
        use microcad_lang_lower::ir::visitor::WorkbenchExpressionVisitorMut;
        self.visit_workbench_signature(&mut workbench.signature);

        workbench
            .statements
            .iter_mut()
            .for_each(|stmt| self.visit_workbench_statement(stmt))
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
        init.parameters
            .iter()
            .for_each(|param| self.declare_local(param.id.clone()));

        init.statements.iter_mut().for_each(|init_statement| {
            self.visit_name(&mut init_statement.name);
            self.visit_workbench_expr(&mut init_statement.expression);
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
            self.declare_local(name.clone());
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
            self.declare_local(name.clone());
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
