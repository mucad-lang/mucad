// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Import values from TOML

use mucad_lang_types::*;

/// Import TOML files into a tuple.
pub struct TomlImporter;

impl TomlImporter {
    /// Convert a TOML value into a µcad value.
    #[allow(unused)]
    fn toml_to_value(toml: toml::Value) -> Value {
        match toml {
            toml::Value::String(s) => s.into(),
            toml::Value::Integer(i) => i.into(),
            toml::Value::Float(f) => f.into(),
            toml::Value::Boolean(b) => b.into(),
            toml::Value::Datetime(_) => todo!(),
            toml::Value::Array(values) => {
                List::from_iter(values.into_iter().map(Self::toml_to_value)).into()
            }
            toml::Value::Table(map) => Tuple::from_iter(
                map.into_iter()
                    .map(|(k, v)| (k.into(), Self::toml_to_value(v))),
            )
            .into(),
        }
    }
}
