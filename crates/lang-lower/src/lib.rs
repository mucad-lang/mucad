// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Lowering compile step

pub mod ir;

mod diag;
pub use diag::{LowerError, LowerInfo, LowerIssue, LowerResult, LowerWarning};

mod desugar;
mod fold;
mod scaffold;

use microcad_builtin::BuiltinLibrary;
use microcad_lang_base::{
    IssueResult, HashId, IssueList, LookUpName, PushIssue, Shared, Source, Span,
    SpanToSrcRef, SrcRef, SymbolId, hash_id,
};

pub use ir::CastInto;

/// Intermediate representation
use microcad_lang_parse::Ast;
use microcad_macros::Artifact;
use serde::{Deserialize, Serialize};

use crate::scaffold::Scaffold;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Artifact)]
pub struct Ir {
    pub input_hash: HashId,
    pub output_hash: HashId,
    pub tree: ir::Tree,
}

pub trait Unresolver {
    fn unresolve(&self, id: impl Into<SymbolId>) -> String;
}

impl Desugar<Ast> for ir::desugared::Source {
    fn desugar(node: &Ast, context: &mut LowerContext) -> LowerResult<Self> {
        ir::desugared::Source::desugar(node.tree(), context)
    }
}

pub struct LowerContext<'source> {
    source: &'source Source,
    arena: ir::Arena,
    node_id_stack: Vec<ir::NodeId>,
    builtins: Shared<BuiltinLibrary>,
    issues: IssueList<LowerIssue>,
}

impl<'source> LookUpName for LowerContext<'source> {
    fn look_up_built_in_name(
        &self,
        builtin_id: &microcad_builtin::BuiltinId,
    ) -> Option<microcad_lang_base::Name> {
        self.builtins.look_up_built_in_name(builtin_id)
    }
}

/// Builder methods
impl<'source> LowerContext<'source> {
    /// Create a context with default settings and a fresh `BuiltinRegistry`.
    pub fn new(source: &'source Source) -> Self {
        LowerContextBuilder::new(source).build()
    }

    /// Create a builder for customized lowering setup (e.g., passing shared builtins).
    pub fn builder(source: &'source Source) -> LowerContextBuilder<'source> {
        LowerContextBuilder::new(source)
    }
}

/// Accessors
impl<'source> LowerContext<'source> {
    /// Access built-in registry.
    pub fn builtins(&self) -> &BuiltinLibrary {
        &self.builtins
    }

    pub fn top_node(&self) -> &ir::NodeId {
        self.node_id_stack.last().unwrap()
    }

    pub fn scaffold_item(&mut self, node: impl Into<ir::Item>) -> ir::NodeId {
        self.arena.new_node(node.into())
    }

    pub fn scaffold_item_with_children(
        &mut self,
        node: impl Into<ir::Item>,
        children: impl Scaffold,
    ) -> ir::NodeId {
        let node_id = self.scaffold_item(node);
        self.node_id_stack.push(node_id);
        children.scaffold(self);
        self.node_id_stack.pop();
        node_id
    }
}

impl<'source> PushIssue<LowerIssue> for LowerContext<'source> {
    fn push_issue(&mut self, issue: impl Into<LowerIssue>) {
        self.issues.push_issue(issue);
    }

    fn push_err(&mut self, err: impl Into<LowerError>) {
        self.issues.push_err(err);
    }

    fn push_warn(&mut self, warn: impl Into<LowerWarning>) {
        self.issues.push_warn(warn);
    }

    fn push_info(&mut self, info: impl Into<LowerInfo>) {
        self.issues.push_info(info);
    }
}

impl<'source> From<&'source Source> for LowerContext<'source> {
    fn from(source: &'source Source) -> Self {
        Self::new(source)
    }
}

impl<'source> SpanToSrcRef for LowerContext<'source> {
    fn span_to_src_ref(&self, span: &Span) -> SrcRef {
        self.source.span_to_src_ref(span)
    }
}

pub trait Desugar<AstNode>: Sized {
    fn desugar(node: &AstNode, context: &mut LowerContext) -> LowerResult<Self>;
}

impl Unresolver for BuiltinLibrary {
    fn unresolve(&self, id: impl Into<SymbolId>) -> String {
        let id = id.into();
        match id {
            SymbolId::Builtin(builtin_id) => {
                if let Some(builtin) = self.get(builtin_id) {
                    String::from(builtin.name())
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }
}

pub struct LowerContextBuilder<'source> {
    source: &'source Source,
    builtins: Option<Shared<BuiltinLibrary>>,
}

impl<'source> LowerContextBuilder<'source> {
    pub fn new(source: &'source Source) -> Self {
        Self {
            source,
            builtins: None,
        }
    }

    /// Provide a custom or shared `BuiltinRegistry`.
    pub fn with_builtins(mut self, builtins: Shared<BuiltinLibrary>) -> Self {
        self.builtins = Some(builtins);
        self
    }

    /// Build and initialize the lowering context.
    pub fn build(self) -> LowerContext<'source> {
        LowerContext {
            source: self.source,
            arena: ir::Arena::default(),
            node_id_stack: Vec::new(),
            builtins: self
                .builtins
                .unwrap_or_else(|| Shared::new(BuiltinLibrary::new())),
            issues: IssueList::default(),
        }
    }
}

pub fn lower<'source>(
    context: &mut LowerContext<'source>,
    ast: &Ast,
) -> IssueResult<Ir, LowerIssue> {
    // Short-circuit on fatal errors
    let ir = match ir::desugared::Source::desugar(ast.tree(), context) {
        Ok(ir) => ir,
        Err(fatal_error) => {
            // Ensure the fatal error is logged in the diagnostics
            context.push_err(fatal_error);
            let issues = std::mem::take(&mut context.issues);
            return Err(issues);
        }
    };

    let root = ir.scaffold(context);
    let arena = std::mem::take(&mut context.arena);
    let mut tree = ir::Tree::from((root, arena));

    {
        use crate::ir::visitor::VisitorMut;
        fold::Fold::new(context).visit_tree(&mut tree);
    }

    let ir = Ir {
        input_hash: ast.output_hash(),
        output_hash: hash_id!(tree),
        tree,
    };

    let issues = std::mem::take(&mut context.issues);

    if issues.has_errors() {
        Err(issues)
    } else {
        Ok((ir, issues))
    }
}
