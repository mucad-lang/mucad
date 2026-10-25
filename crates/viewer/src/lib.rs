// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad viewer library.

pub mod config;
pub mod material;
pub mod plugin;
pub mod processor;
pub mod scene;
pub mod screenshot;
pub mod stdin;
pub mod to_bevy;
pub mod view_model;

pub use crate::config::Config;
pub use crate::to_bevy::ToBevy;

pub use crate::view_model::ViewModel;

pub use crate::plugin::MucadPlugin;
