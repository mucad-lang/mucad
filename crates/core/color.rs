// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad color syntax element

use miette::Diagnostic;
use std::str::FromStr;

/// A color with RGBA channels
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    /// red value
    pub r: f32,
    /// green value
    pub g: f32,
    /// blue value
    pub b: f32,
    /// alpha value
    pub a: f32,
}

impl Color {
    /// Built-in red color.
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    /// Built-in green color.
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    /// Built-in blue color.
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    /// Built-in yellow color.
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    /// Built-in cyan color.
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    /// Built-in magenta color.
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    /// Built-in black color.
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    /// Built-in white color.
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    /// Built-in gray color.
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    /// Built-in orange color.
    pub const ORANGE: Self = Self::rgb(1.0, 0.5, 0.0);
    /// Built-in purple color.
    pub const PURPLE: Self = Self::rgb(0.5, 0.0, 0.5);
    /// Built-in pink color.
    pub const PINK: Self = Self::rgb(1.0, 0.75, 0.8);
    /// Built-in brown color.
    pub const BROWN: Self = Self::rgb(0.6, 0.3, 0.1);
    /// Built-in lime color.
    pub const LIME: Self = Self::rgb(0.75, 1.0, 0.0);
    /// Built-in teal color.
    pub const TEAL: Self = Self::rgb(0.0, 0.5, 0.5);
    /// Built-in navy color.
    pub const NAVY: Self = Self::rgb(0.0, 0.0, 0.5);
    /// Built-in transparent color.
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Create new color.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create new color from RGBA values.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    /// Create new color from RGB values. Alpha is 1.0.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0_f32)
    }

    /// Make color transparent.
    pub fn make_transparent(self, a: f32) -> Self {
        let mut new = self;
        new.a = a;
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
        let r = (self.r * 255.0) as u8;
        let g = (self.g * 255.0) as u8;
        let b = (self.b * 255.0) as u8;
        let a = self.a;

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

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blue" => Ok(Self::BLUE),
            "red" => Ok(Self::RED),
            "green" => Ok(Self::GREEN),
            "yellow" => Ok(Self::YELLOW),
            "cyan" => Ok(Self::CYAN),
            "magenta" => Ok(Self::MAGENTA),
            "black" => Ok(Self::BLACK),
            "white" => Ok(Self::WHITE),
            "gray" => Ok(Self::GRAY),
            "orange" => Ok(Self::ORANGE),
            "purple" => Ok(Self::PURPLE),
            "pink" => Ok(Self::PINK),
            "brown" => Ok(Self::BROWN),
            "lime" => Ok(Self::LIME),
            "teal" => Ok(Self::TEAL),
            "navy" => Ok(Self::NAVY),
            "transparent" | "none" => Ok(Self::TRANSPARENT),
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
