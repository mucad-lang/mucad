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
        hooks.insert::<mu::ops::Difference>();
        hooks.insert::<mu::ops::Extrude>();
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
