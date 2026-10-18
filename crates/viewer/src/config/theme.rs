// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Color theme

use bevy::ecs::resource::Resource;
use mucad_driver::prelude as mu;

use serde::Deserialize;

/// Represents a color theme.
#[derive(Resource, Clone, Debug, PartialEq, Deserialize)]
pub struct Theme {
    /// Name of the theme.
    pub name: Option<String>,

    /// Primary color.
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub primary: mu::Color,

    /// Brighter color.
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub bright: mu::Color,

    /// Brighter color.
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub brighter: mu::Color,

    /// Darker color.
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub darker: mu::Color,

    /// Complementary color (e.g. operators, intersections).
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub complementary: mu::Color,

    /// Signal color, e.g. for selections.
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub signal: mu::Color,

    /// Color for guidelines (e.g. inference lines).
    #[serde(deserialize_with = "deserialize_color_from_hex")]
    pub guide: mu::Color,
}

fn deserialize_color_from_hex<'de, D>(deserializer: D) -> Result<mu::Color, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    mu::Color::from_hex_str(&s)
        .map_err(|err| serde::de::Error::custom(format!("invalid color '{}': {}", s, err)))
}

impl Default for Theme {
    fn default() -> Self {
        let hex = |s| mu::Color::from_hex_str(s).expect("Valid hex string");

        Self {
            name: Some("default".into()),
            primary: hex("#001728"),
            bright: hex("#DBF3FF"),
            brighter: hex("#7CA0B1"),
            darker: hex("#477991"),
            complementary: hex("#7D0303"),
            signal: hex("#FFA41C"),
            guide: hex("#009CDE"),
        }
    }
}
