// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

use mucad_driver::prelude as mu;
use mucad_viewer_ipc::ViewerCommand;

/// Configuration of the LSP serve
#[derive(Default, Debug)]
pub struct ServiceConfig {
    /// The viewer to show the active document, e.g. [`ViewerCommand::mucad_viewer`],
    /// or `None` for no viewer.
    pub viewer: Option<ViewerCommand>,
}

/// µcad LSP config with driver config and service config.
#[derive(Default, Debug)]
pub struct Config {
    /// µcad Driver configuration
    pub driver: mu::DriverConfig,

    /// LSP service
    pub service: ServiceConfig,
}
