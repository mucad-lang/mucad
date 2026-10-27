// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! WASM API of shared links with a preamble.

use mucad_driver::prelude as mu;
use wasm_bindgen::prelude::wasm_bindgen;

/// The code and the preamble of a shared link, see `Mucad.decodeLinkWithPreamble`.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct SharedLink {
    code: String,
    preamble: String,
}

#[wasm_bindgen]
impl SharedLink {
    /// The formatted µcad code, code with errors is unformatted.
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }

    /// The preamble as JSON object, e.g. `{"name":"brick.mu"}`, `{}` without a preamble.
    #[wasm_bindgen(getter)]
    pub fn preamble(&self) -> String {
        self.preamble.clone()
    }
}

/// The link with the µcad `code` behind the `preamble` as JSON object, e.g.
/// `{"name":"brick.mu"}`, which is written as TOML.
///
/// Fails, if the preamble is no JSON object or has values, which TOML has not, e.g. `null`.
pub fn encode(code: &str, preamble: &str, strip: bool) -> Result<String, String> {
    let preamble: toml::Table = serde_json::from_str(preamble)
        .map_err(|err| format!("the preamble is no JSON object for TOML: {err}"))?;
    mu::share::link_with_preamble(code, &preamble, strip).map_err(|err| err.to_string())
}

/// The code and the preamble as JSON object of a shared link.
pub fn decode(link: &str) -> Result<SharedLink, String> {
    let shared = mu::share::decode_with_preamble(link).map_err(|err| err.to_string())?;
    Ok(SharedLink {
        code: shared.code,
        preamble: serde_json::to_string(&shared.preamble).map_err(|err| err.to_string())?,
    })
}
