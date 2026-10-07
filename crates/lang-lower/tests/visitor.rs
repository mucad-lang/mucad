// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;

use microcad_lang_base::SymbolId;
use microcad_lang_lower::ir;
use microcad_lang_lower::ir::visitor::{
    ConstantVisitor, FnVisitor, LeafVisitor, SourceVisitor, Visitor, WorkbenchExpressionVisitor,
    WorkbenchVisitor,
};
use microcad_macros::__mu;
use std::collections::HashSet;

/// A visitor that collects references to all `ir::Path` instances found in the IR tree.
#[derive(Debug, Default)]
pub struct PathCollector {
    pub paths: Vec<ir::Path>,
}

impl PathCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience helper to collect all unique paths as a HashSet.
    pub fn collect_unique(mut self, tree: &ir::Tree) -> HashSet<ir::Path> {
        self.visit_tree(tree);
        self.paths.into_iter().collect()
    }
}

// Core leaf visitor overriding path collection
impl LeafVisitor for PathCollector {
    fn visit_path(&mut self, path: &ir::Path) {
        self.paths.push(path.clone());
    }
}

// Sub-trait implementations (inherit default traversal behavior)
impl ConstantVisitor for PathCollector {}
impl WorkbenchExpressionVisitor for PathCollector {}
impl WorkbenchVisitor for PathCollector {}
impl FnVisitor for PathCollector {}
impl SourceVisitor for PathCollector {}
impl Visitor for PathCollector {}

#[test]
fn path_collector() {
    let source = common::source_from_test_file("circle");
    let (ir, _) = common::ir_from_source(&source).expect("No errors");

    // Collect all paths from a full IR tree
    let mut collector = PathCollector::new();
    collector.visit_tree(&ir.tree);

    let paths = collector.collect_unique(&ir.tree);
    println!("Found {} paths", paths.len());
    for path in &paths {
        println!("{path}");
    }

    let symbol_ids: Vec<_> = paths
        .iter()
        .filter_map(|path| path.symbol_id())
        .cloned()
        .collect();

    assert!(symbol_ids.contains(&SymbolId::Builtin(__mu!(geo2d::Circle))));
    assert!(symbol_ids.contains(&SymbolId::Builtin(__mu!(core::member_access))));
}
