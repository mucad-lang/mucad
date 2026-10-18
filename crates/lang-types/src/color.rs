// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad color syntax element

use derive_more::Display;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Copy, Display, Hash, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[display("{:.2}", *_0 as f32 / u16::MAX as f32)]
pub struct ColorChannel(u16);

impl From<f32> for ColorChannel {
    fn from(value: f32) -> Self {
        // Clamp NaN/out-of-bound values strictly to [0.0, 1.0]
        let clamped = value.clamp(0.0, 1.0);

        // Scale to standard 16-bit integer range with proper rounding
        let scaled = (clamped * u16::MAX as f32).round() as u16;

        ColorChannel(scaled)
    }
}

impl From<ColorChannel> for f32 {
    fn from(color_channel: ColorChannel) -> Self {
        color_channel.0 as f32
    }
}

impl From<ColorChannel> for Value {
    fn from(color_channel: ColorChannel) -> Self {
        Self::from(f32::from(color_channel))
    }
}

impl From<ColorChannel> for u8 {
    fn from(color_channel: ColorChannel) -> Self {
        (color_channel.0 / 256) as u8
    }
}

/// A color with RGBA channels
#[derive(Copy, Display, Hash, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[display("rgba({r}, {g}, {b}, {a})")]
pub struct Color {
    /// red value
    pub r: ColorChannel,
    /// green value
    pub g: ColorChannel,
    /// blue value
    pub b: ColorChannel,
    /// alpha value
    pub a: ColorChannel,
}

impl Color {
    /// Create new color.
    pub fn new(
        r: impl Into<ColorChannel>,
        g: impl Into<ColorChannel>,
        b: impl Into<ColorChannel>,
        a: impl Into<ColorChannel>,
    ) -> Self {
        Self {
            r: r.into(),
            g: g.into(),
            b: b.into(),
            a: a.into(),
        }
    }

    /// Create new color from RGBA values.
    pub fn rgba(
        r: impl Into<ColorChannel>,
        g: impl Into<ColorChannel>,
        b: impl Into<ColorChannel>,
        a: impl Into<ColorChannel>,
    ) -> Self {
        Self::new(r, g, b, a)
    }

    /// Create new color from RGB values. Alpha is 1.0.
    pub fn rgb(
        r: impl Into<ColorChannel>,
        g: impl Into<ColorChannel>,
        b: impl Into<ColorChannel>,
    ) -> Self {
        Self::rgba(r, g, b, 1.0_f32)
    }

    /// Make color transparent.
    pub fn make_transparent(self, a: impl Into<ColorChannel>) -> Self {
        let mut new = self;
        new.a = a.into();
        new
    }

    /// Construct a color from a hex string like `#FFCCAA`.
    pub fn from_hex_str(hex: &str) -> Result<Self, ParseColorError> {
        if !hex.starts_with("#") {
            return Err(ParseColorError::ParseColorFromHex(hex.into()));
        }
        let hex = &hex[1..];

        let hex4bit = |pos| u8::from_str_radix(&hex[pos..pos + 1], 16).map(|v| v as f32 / 15.0);
        let hex8bit = |pos| u8::from_str_radix(&hex[pos..pos + 2], 16).map(|v| v as f32 / 255.0);

        match hex.len() {
            // #RGB or #RGBA single digit hex
            3 | 4 => Ok(Color::rgba(
                hex4bit(0)?,
                hex4bit(1)?,
                hex4bit(2)?,
                if hex.len() == 4 { hex4bit(3)? } else { 1.0 },
            )),
            // #RRGGBB or #RRGGBBAA double digit hex
            6 | 8 => Ok(Color::rgba(
                hex8bit(0)?,
                hex8bit(2)?,
                hex8bit(4)?,
                if hex.len() == 8 { hex8bit(6)? } else { 1.0 },
            )),
            _ => Err(ParseColorError::ParseColorFromHex(hex.into())),
        }
    }

    /// Return color as SVG color.
    pub fn to_svg_color(&self) -> String {
        let r: u8 = self.r.into();
        let g: u8 = self.g.into();
        let b: u8 = self.b.into();
        let a: f32 = self.a.into();

        if a < 1.0 {
            format!("rgba({r}, {g}, {b}, {a:.2})")
        } else {
            format!("#{r:02X}{g:02X}{b:02X}")
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgba(0.5, 0.5, 0.5, 1.0)
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blue" => Ok(Self::rgb(0.0, 0.0, 1.0)),
            "red" => Ok(Self::rgb(1.0, 0.0, 0.0)),
            "green" => Ok(Self::rgb(0.0, 1.0, 0.0)),
            "yellow" => Ok(Self::rgb(1.0, 1.0, 0.0)),
            "cyan" => Ok(Self::rgb(0.0, 1.0, 1.0)),
            "magenta" => Ok(Self::rgb(1.0, 0.0, 1.0)),
            "black" => Ok(Self::rgb(0.0, 0.0, 0.0)),
            "white" => Ok(Self::rgb(1.0, 1.0, 1.0)),
            "gray" => Ok(Self::rgb(0.5, 0.5, 0.5)),
            "orange" => Ok(Self::rgb(1.0, 0.5, 0.0)),
            "purple" => Ok(Self::rgb(0.5, 0.0, 0.5)),
            "pink" => Ok(Self::rgb(1.0, 0.75, 0.8)),
            "brown" => Ok(Self::rgb(0.6, 0.3, 0.1)),
            "lime" => Ok(Self::rgb(0.75, 1.0, 0.0)),
            "teal" => Ok(Self::rgb(0.0, 0.5, 0.5)),
            "navy" => Ok(Self::rgb(0.0, 0.0, 0.5)),
            "transparent" => Ok(Self::rgba(0.0, 0.0, 0.0, 0.0)),
            s => {
                if s.starts_with("#") {
                    Self::from_hex_str(s)
                } else {
                    Err(ParseColorError::UnknownColorName(s.to_string()))
                }
            }
        }
    }
}

use thiserror::Error;

use crate::Value;

/// An error when parsing a color from a string
#[derive(Error, Debug, Diagnostic)]
pub enum ParseColorError {
    /// Unknown color name.
    #[error("Unknown color name: {0}")]
    UnknownColorName(String),

    /// Unknown color name.
    #[error("Could not parse color from hex string: {0}")]
    ParseColorFromHex(String),

    /// Error parsing integer.
    #[error("Error parsing integer literal: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}
