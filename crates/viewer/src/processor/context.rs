// Copyright © 2025-2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The context of geometry processor.

use microcad_driver::prelude as mu;

use crate::{config, processor::registry::InstanceRegistry};

/// The current state of the processor.
#[derive(Debug, Clone, Default)]
pub enum ProcessingState {
    #[default]
    /// The processor does currently nothing.
    Idle,
    /// The processor is busy (with progress between 0..100.0)
    Busy(f32),
    /// The processor is in an error state.
    Error,
}

/// The context of the processor.
///
/// It caches the relevant data to assure that only necessary models will be rerendered.
pub struct ProcessorContext {
    pub(super) state: ProcessingState,

    /// Flag to tell whether to initialize.
    pub(super) initialized: bool,

    /// The compiler, which is created with the search paths during initialization.
    pub(super) compiler: mu::Compiler,

    /// The renderer with the render resolution and the render cache,
    /// which keeps the geometry of unchanged models between compilations.
    pub(super) renderer: mu::render::Renderer,

    pub(super) theme: config::Theme,

    pub(super) line_number: Option<u32>,

    /// Keeps a UUID register of all model instances that have been rendered.
    pub(super) instance_registry: InstanceRegistry,
}

impl Default for ProcessorContext {
    fn default() -> Self {
        Self {
            state: Default::default(),
            initialized: false,
            compiler: mu::Compiler::new(),
            renderer: mu::render::Renderer::new()
                .with_cache(mu::SharedMut::new(mu::render::RenderCache::new())),
            theme: Default::default(),
            line_number: None,
            instance_registry: Default::default(),
        }
    }
}
