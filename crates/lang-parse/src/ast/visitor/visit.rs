// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Default visit implementation for an abstract syntax nodes.

use crate::ast;

use crate::ast::Visitor;
use std::ops::ControlFlow;

/// Default node visitor.
pub trait Visit {
    /// Default visitor method.
    fn visit<'ast, V>(&'ast self, _visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        ControlFlow::Continue(())
    }
}

impl<T: Visit> Visit for Vec<T> {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        for item in self {
            item.visit(visitor)?;
        }
        ControlFlow::Continue(())
    }
}

impl<T: Visit> Visit for Option<T> {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        if let Some(item) = &self {
            item.visit(visitor)?;
        }
        ControlFlow::Continue(())
    }
}

impl<T: Visit> Visit for Box<T> {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        self.as_ref().visit(visitor)
    }
}

impl<T: Visit> Visit for mucad_lang_base::Spanned<T> {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        self.value.visit(visitor)
    }
}

// Primitives that don't need visiting

impl Visit for ast::Span {}
impl Visit for ast::def::WorkbenchKind {}

impl Visit for ast::Comment {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        visitor.visit_comment(self)
    }
}

impl Visit for ast::Attributes {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        self.0.visit(visitor)
    }
}

impl Visit for ast::StatementList {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        self.extras.visit(visitor)?;
        for (statement, trailing) in &self.statements {
            statement.visit(visitor)?;
            trailing.visit(visitor)?;
        }
        self.tail.visit(visitor)
    }
}

impl Visit for ast::LeadingExtras {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        self.0.visit(visitor)
    }
}

impl Visit for ast::TrailingExtras {
    fn visit<'ast, V>(&'ast self, visitor: &mut V) -> ControlFlow<V::BreakTy>
    where
        V: Visitor<'ast> + ?Sized,
    {
        self.0.visit(visitor)
    }
}
