// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A visitor to make all paths in an IR human readable

use mucad_lang_base::LookUpName;

use crate::ir;

/// A visitor to make all paths in an IR human readable
#[derive(Debug)]
pub struct MakeHumanReadable<'a, Ctx: LookUpName> {
    pub ctx: &'a Ctx,
}

impl<'a, Ctx: LookUpName> MakeHumanReadable<'a, Ctx> {
    pub fn new(ctx: &'a Ctx) -> Self {
        Self { ctx }
    }
}

impl<'a, Ctx: LookUpName> ir::visitor::LeafVisitorMut for MakeHumanReadable<'a, Ctx> {
    fn visit_path(&mut self, path: &mut ir::Path) {
        if let ir::Path::Resolved(symbol_id) = path
            && let Some(name) = self.ctx.look_up_symbol_name(symbol_id)
        {
            *path = ir::Path::HumanReadable {
                name,
                id: symbol_id.clone(),
            };
        }
    }
}

// Sub-trait implementations (inherit default traversal behavior)
impl<'a, Ctx: LookUpName> ir::visitor::ConstantVisitorMut for MakeHumanReadable<'a, Ctx> {}
impl<'a, Ctx: LookUpName> ir::visitor::WorkbenchExpressionVisitorMut
    for MakeHumanReadable<'a, Ctx>
{
}
impl<'a, Ctx: LookUpName> ir::visitor::WorkbenchVisitorMut for MakeHumanReadable<'a, Ctx> {}
impl<'a, Ctx: LookUpName> ir::visitor::FnVisitorMut for MakeHumanReadable<'a, Ctx> {}
impl<'a, Ctx: LookUpName> ir::visitor::SourceVisitorMut for MakeHumanReadable<'a, Ctx> {}
impl<'a, Ctx: LookUpName> ir::visitor::VisitorMut for MakeHumanReadable<'a, Ctx> {}
