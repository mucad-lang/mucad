// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Library visitors

use mucad_lang_lower::ir::Meta;
pub use mucad_lang_lower::ir::visitor::{
    ConstantVisitor, ConstantVisitorMut, FnVisitor, FnVisitorMut, LeafVisitor, LeafVisitorMut,
    SourceVisitor, SourceVisitorMut, WorkbenchExpressionVisitor, WorkbenchExpressionVisitorMut,
    WorkbenchVisitor, WorkbenchVisitorMut,
};

use crate::{
    Library, SymbolDef, SymbolNodeRef,
    library::{Symbol, symbol},
};

/// Visitor for a Library tree.
pub trait Visitor: SourceVisitor + FnVisitor + WorkbenchVisitor + ConstantVisitor {
    fn visit(&mut self, library: &Library) {
        library
            .root()
            .descendants()
            .for_each(|node| self.visit_node(node));
    }

    fn visit_node<'a>(&mut self, node: SymbolNodeRef<'a>) {
        self.visit_symbol(node.get());
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.visit_meta(&symbol.meta);
        self.visit_def(&symbol.def);
    }

    fn visit_file_path(&mut self, _path: &std::path::Path) {}

    fn visit_meta(&mut self, _meta: &Meta) {}
    fn visit_inline_module(&mut self, _inline_module: &symbol::InlineModule) {}
    fn visit_source_file(&mut self, source_file: &symbol::SourceFile) {
        match source_file {
            symbol::SourceFile::NotLoaded => {}
            symbol::SourceFile::Loaded { path, source } => {
                self.visit_file_path(path);
                self.visit_source(source);
            }
            symbol::SourceFile::Failed { path } => self.visit_file_path(path),
        }
    }

    fn visit_alias(&mut self, alias: &symbol::Alias) {
        self.visit_path(&alias.path);
    }
    fn visit_wildcard(&mut self, wildcard: &symbol::Wildcard) {
        self.visit_path(&wildcard.path);
    }

    fn visit_def(&mut self, def: &SymbolDef) {
        use SymbolDef::*;
        match def {
            // The sources of a library are its children.
            Root => {}
            InlineModule(inline_module) => self.visit_inline_module(inline_module),
            SourceFile(file_module) => self.visit_source_file(file_module),
            Workbench(workbench) => self.visit_workbench(workbench),
            Function(function) => self.visit_fn(function),
            Constant(constant) => self.visit_constant(constant),
            Alias(alias) => self.visit_alias(alias),
            Wildcard(wildcard) => self.visit_wildcard(wildcard),
        }
    }
}

/// Mutable Visitor for a Library tree.
pub trait VisitorMut:
    SourceVisitorMut + FnVisitorMut + WorkbenchVisitorMut + ConstantVisitorMut
{
    fn visit(&mut self, library: &mut Library) {
        library
            .root_mut()
            .transform(|node, symbol| self.visit_symbol(node, symbol));
    }

    fn visit_symbol<'a>(&mut self, _node: SymbolNodeRef<'a>, symbol: &mut Symbol) {
        self.visit_meta(&mut symbol.meta);
        self.visit_def(&mut symbol.def);
    }

    fn visit_file_path(&mut self, _path: &mut std::path::Path) {}
    fn visit_meta(&mut self, _meta: &mut Meta) {}

    fn visit_inline_module(&mut self, _inline_module: &mut symbol::InlineModule) {}
    fn visit_source_file(&mut self, source_file: &mut symbol::SourceFile) {
        match source_file {
            symbol::SourceFile::NotLoaded => {}
            symbol::SourceFile::Loaded { path, source } => {
                self.visit_file_path(path);
                self.visit_source(source);
            }
            symbol::SourceFile::Failed { path } => self.visit_file_path(path),
        }
    }

    fn visit_alias(&mut self, alias: &mut symbol::Alias) {
        self.visit_path(&mut alias.path);
    }
    fn visit_wildcard(&mut self, wildcard: &mut symbol::Wildcard) {
        self.visit_path(&mut wildcard.path);
    }

    fn visit_def(&mut self, def: &mut SymbolDef) {
        use SymbolDef::*;
        match def {
            // The sources of a library are its children.
            Root => {}
            InlineModule(inline_module) => self.visit_inline_module(inline_module),
            SourceFile(file_module) => self.visit_source_file(file_module),
            Workbench(workbench) => self.visit_workbench(workbench),
            Function(function) => self.visit_fn(function),
            Constant(constant) => self.visit_constant(constant),
            Alias(alias) => self.visit_alias(alias),
            Wildcard(wildcard) => self.visit_wildcard(wildcard),
        }
    }
}
