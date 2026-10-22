// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resolve stack.

use derive_more::From;
use mucad_lang_base::Identifier;

use crate::library::SymbolNodeId;

mod stackframe;
pub use stackframe::{LocalTable, ScopeAccess};

#[derive(Debug, Default)]
pub(crate) struct FunctionFrame {
    //symbol: mir::SymbolHandle,
    locals: LocalTable,
}

impl ScopeAccess for FunctionFrame {
    fn local_table(&self) -> Option<&LocalTable> {
        self.locals.local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.locals.local_table_mut()
    }
}

#[derive(Debug, Default)]
pub(crate) struct FunctionScopeFrame(LocalTable);

impl ScopeAccess for FunctionScopeFrame {
    fn local_table(&self) -> Option<&LocalTable> {
        self.0.local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.0.local_table_mut()
    }
}

#[derive(Debug, Default)]
pub(crate) struct WorkbenchFrame {
    //symbol: mir::SymbolHandle,
    locals: LocalTable,
}

impl ScopeAccess for WorkbenchFrame {
    fn local_table(&self) -> Option<&LocalTable> {
        self.locals.local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.locals.local_table_mut()
    }
}

#[derive(Debug, Default)]
pub(crate) struct WorkbenchGroupFrame(LocalTable);

impl ScopeAccess for WorkbenchGroupFrame {
    fn local_table(&self) -> Option<&LocalTable> {
        self.0.local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.0.local_table_mut()
    }
}

#[derive(Debug, Default)]
pub(crate) struct SourceFrame {
    //symbol: mir::SymbolHandle,
    locals: LocalTable,
}

impl ScopeAccess for SourceFrame {
    fn local_table(&self) -> Option<&LocalTable> {
        self.locals.local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.locals.local_table_mut()
    }
}

#[derive(Debug)]
pub(crate) struct SymbolFrame {
    pub node_id: SymbolNodeId,
}

impl SymbolFrame {
    pub fn new(node_id: SymbolNodeId) -> Self {
        Self { node_id }
    }
}

#[derive(Debug, From)]
pub(crate) enum ResolveStackFrame {
    Source(SourceFrame),
    Function(FunctionFrame),
    FunctionScope(FunctionScopeFrame),
    Workbench(WorkbenchFrame),
    WorkbenchGroup(WorkbenchGroupFrame),
    Symbol(SymbolFrame),
}

impl ScopeAccess for ResolveStackFrame {
    fn local_table(&self) -> Option<&LocalTable> {
        match &self {
            ResolveStackFrame::Source(source_file_frame) => source_file_frame.local_table(),
            ResolveStackFrame::Function(function_frame) => function_frame.local_table(),
            ResolveStackFrame::FunctionScope(function_scope) => function_scope.local_table(),
            ResolveStackFrame::Workbench(workbench) => workbench.local_table(),
            ResolveStackFrame::WorkbenchGroup(workbench_group) => workbench_group.local_table(),
            _ => None,
        }
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        match self {
            ResolveStackFrame::Source(source_file_frame) => source_file_frame.local_table_mut(),
            ResolveStackFrame::Function(function_frame) => function_frame.local_table_mut(),
            ResolveStackFrame::FunctionScope(function_scope) => function_scope.local_table_mut(),
            ResolveStackFrame::Workbench(workbench) => workbench.local_table_mut(),
            ResolveStackFrame::WorkbenchGroup(workbench_group) => workbench_group.local_table_mut(),
            _ => None,
        }
    }

    fn symbol_node_id(&self) -> Option<SymbolNodeId> {
        match self {
            ResolveStackFrame::Symbol(SymbolFrame { node_id }) => Some(*node_id),
            _ => None,
        }
    }
}

/// A stack for resolving.
#[derive(Debug, Default)]
pub struct ResolveStack(Vec<ResolveStackFrame>);

impl ResolveStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, frame: impl Into<ResolveStackFrame>) {
        self.0.push(frame.into());
    }

    pub fn pop(&mut self) -> ResolveStackFrame {
        self.0.pop().expect("A stack frame")
    }

    pub fn top(&self) -> &ResolveStackFrame {
        self.0.last().expect("A stack frame") // Intentionally no error handling here
    }

    pub fn top_mut(&mut self) -> &mut ResolveStackFrame {
        self.0.last_mut().expect("A stack frame")
    }

    /// Returns true if `id` is a local in any frame of the current symbol.
    pub fn is_local(&self, id: &Identifier) -> bool {
        self.current_symbol_scope().any(|frame| {
            frame
                .local_table()
                .is_some_and(|local_table| local_table.exists(id))
        })
    }

    pub fn current_symbol_scope(&self) -> impl Iterator<Item = &ResolveStackFrame> {
        self.0.iter().rev().scan(false, |hit_symbol_frame, frame| {
            if *hit_symbol_frame {
                return None;
            }
            if matches!(frame, ResolveStackFrame::Symbol(_)) {
                *hit_symbol_frame = true;
            }
            Some(frame)
        })
    }

    pub fn traversal_mut<T>(
        &mut self,
        mut f: impl FnMut(&mut ResolveStackFrame) -> std::ops::ControlFlow<T>,
    ) -> Option<T> {
        for frame in self.0.iter_mut().rev() {
            if let std::ops::ControlFlow::Break(value) = f(frame) {
                return Some(value);
            }
        }
        None
    }
}

impl ScopeAccess for ResolveStack {
    fn local_table(&self) -> Option<&LocalTable> {
        self.top().local_table()
    }

    fn local_table_mut(&mut self) -> Option<&mut LocalTable> {
        self.top_mut().local_table_mut()
    }

    fn symbol_node_id(&self) -> Option<SymbolNodeId> {
        self.current_symbol_scope().find_map(|frame| match frame {
            ResolveStackFrame::Symbol(SymbolFrame { node_id }) => Some(*node_id),
            _ => None,
        })
    }
}
