// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use microcad_driver::prelude as mu;

/// Configuration of the LSP serve
#[derive(Default, Debug)]
pub struct ServiceConfig {
    pub use_viewer: bool,
}

/// µcad LSP config with driver config and service config.
#[derive(Default, Debug)]
pub struct Config {
    /// µcad Driver configuration
    pub driver: mu::DriverConfig,

    /// LSP service
    pub service: ServiceConfig,
}
