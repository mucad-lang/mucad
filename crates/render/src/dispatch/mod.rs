// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_builtin::{BuiltinConstruct, BuiltinId, mu};

use crate::{GeometryOutputs, GeometryTree, Render, RenderContext, RenderError};
use microcad_hash::HashMap;

mod ops;
mod primitives;

/// Built-in execution function signature
pub type RenderFn = fn(&GeometryTree, &mut RenderContext) -> Result<GeometryOutputs, RenderError>;

#[derive(Debug, Default)]
pub struct RenderHooks {
    hooks: HashMap<BuiltinId, RenderFn>,
}

/// Builder methods.
impl RenderHooks {
    pub fn new() -> Self {
        let mut hooks = RenderHooks::default();
        hooks.insert::<mu::geo2d::Circle>();
        hooks.insert::<mu::geo2d::Rect>();
        hooks.insert::<mu::geo3d::Sphere>();
        hooks.insert::<mu::ops::Extrude>();
        hooks.insert::<mu::ops::Buffer>();
        hooks.insert::<mu::ops::Center>();
        hooks.insert::<mu::ops::Hull>();
        hooks.insert::<mu::ops::Revolve>();
        hooks.insert::<mu::ops::Subtract>();
        hooks.insert::<mu::ops::Intersect>();
        hooks.insert::<mu::ops::Union>();
        hooks.insert::<mu::ops::Align>();
        hooks.insert::<mu::ops::DistributeGrid>();
        hooks.insert::<mu::ops::Mirror>();
        hooks.insert::<mu::ops::Reflect>();
        hooks.insert::<mu::ops::Spiralize>();
        hooks.insert::<mu::ops::Multiply>();

        hooks
    }
}

impl RenderHooks {
    pub fn insert<C: BuiltinConstruct + Render>(&mut self) {
        self.hooks.insert(C::ITEM.id(), |tree, ctx| {
            C::from_model(ctx.model(tree).get())?.render(tree, ctx)
        });
    }

    pub fn get(&self, id: BuiltinId) -> Option<RenderFn> {
        self.hooks.get(&id).cloned()
    }
}
