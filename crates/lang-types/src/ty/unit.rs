// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad unit syntax element.

use serde::{Deserialize, Serialize};

use crate::{QuantityType, Scalar, Ty, Type, TypeError};

/// Definition of type & scale of numbers.
#[derive(
    Default, Clone, Debug, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum Unit {
    // Scalar
    /// No unit was given.
    #[default]
    None,
    /// Percents
    Percent,

    // Length
    /// Meters
    Meter,
    /// Centimeters
    Centimeter,
    /// Millimeters
    Millimeter,
    /// Micrometers
    Micrometer,
    /// Inches
    Inch,
    /// Feet
    Foot,
    /// Yards
    Yard,

    // Angle
    /// Degree
    Deg,
    /// Degree
    DegS,
    /// Gradient
    Grad,
    /// Turns
    Turns,
    /// Radians
    Rad,

    // Weight
    /// Grams
    Gram,
    /// Kilograms
    Kilogram,
    /// Pounds
    Pound,
    /// Ounces
    Ounce,

    // Areas
    /// Square Meters
    Meter2,
    /// Square Centimeters
    Centimeter2,
    /// Square Millimeters
    Millimeter2,
    /// Square Micrometers
    Micrometer2,
    /// Square Inches
    Inch2,
    /// Square Foot
    Foot2,
    /// Square Yard
    Yard2,

    // Volumes
    /// Cubic Meters
    Meter3,
    /// Cubic Centimeters
    Centimeter3,
    /// Cubic Millimeters
    Millimeter3,
    /// Cubic Micrometers
    Micrometer3,
    /// Cubic Inches
    Inch3,
    /// Cubic Foot
    Foot3,
    /// Cubic Yard
    Yard3,
    /// Liters
    Liter,
    /// Centiliter
    Centiliter,
    ///Milliliter
    Milliliter,
    /// Microliter
    Microliter,

    /// Density
    GramPerMeter3,
    /// Density
    GramPerMillimeter3,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Scalars
            Self::None => write!(f, ""),
            Self::Percent => write!(f, "%"),

            // Lengths
            Self::Meter => write!(f, "m"),
            Self::Centimeter => write!(f, "cm"),
            Self::Millimeter => write!(f, "mm"),
            Self::Micrometer => write!(f, "µm"),
            Self::Inch => write!(f, "in"),
            Self::Foot => write!(f, "ft"),
            Self::Yard => write!(f, "yd"),

            // Angles
            Self::Deg => write!(f, "deg"),
            Self::DegS => write!(f, "°"),
            Self::Grad => write!(f, "grad"),
            Self::Turns => write!(f, "turns"),
            Self::Rad => write!(f, "rad"),

            // Weights
            Self::Gram => write!(f, "g"),
            Self::Kilogram => write!(f, "kg"),
            Self::Pound => write!(f, "lb"),
            Self::Ounce => write!(f, "oz"),

            // Areas
            Self::Meter2 => write!(f, "m³"),
            Self::Centimeter2 => write!(f, "cm²"),
            Self::Millimeter2 => write!(f, "mm²"),
            Self::Micrometer2 => write!(f, "µm²"),
            Self::Inch2 => write!(f, "in²"),
            Self::Foot2 => write!(f, "ft²"),
            Self::Yard2 => write!(f, "yd²"),

            // Volumes
            Self::Meter3 => write!(f, "m³"),
            Self::Centimeter3 => write!(f, "cm³"),
            Self::Millimeter3 => write!(f, "mm³"),
            Self::Micrometer3 => write!(f, "µm³"),
            Self::Inch3 => write!(f, "in³"),
            Self::Foot3 => write!(f, "ft³"),
            Self::Yard3 => write!(f, "yd³"),

            Self::Milliliter => write!(f, "ml"),
            Self::Centiliter => write!(f, "cl"),
            Self::Liter => write!(f, "l"),
            Self::Microliter => write!(f, "µl"),

            // Density
            Self::GramPerMeter3 => write!(f, "g/m³"),
            Self::GramPerMillimeter3 => write!(f, "g/mm³"),
        }
    }
}

impl Ty for Unit {
    /// Return type to use with this unit.
    fn ty(&self) -> Type {
        Type::Quantity(self.quantity_type())
    }
}

impl Unit {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn quantity_type(&self) -> QuantityType {
        match self {
            Self::None | Self::Percent => QuantityType::Scalar,
            Self::Meter
            | Self::Centimeter
            | Self::Millimeter
            | Self::Micrometer
            | Self::Inch
            | Self::Foot
            | Self::Yard => QuantityType::Length,
            Self::Deg | Self::DegS | Self::Grad | Self::Turns | Self::Rad => QuantityType::Angle,
            Self::Gram | Self::Kilogram | Self::Pound | Self::Ounce => QuantityType::Weight,
            Self::Meter2
            | Self::Centimeter2
            | Self::Millimeter2
            | Self::Micrometer2
            | Self::Inch2
            | Self::Foot2
            | Self::Yard2 => QuantityType::Area,
            Self::Meter3
            | Self::Centimeter3
            | Self::Millimeter3
            | Self::Micrometer3
            | Self::Inch3
            | Self::Foot3
            | Self::Yard3
            | Self::Liter
            | Self::Centiliter
            | Self::Milliliter
            | Self::Microliter => QuantityType::Volume,
            Self::GramPerMeter3 | Self::GramPerMillimeter3 => QuantityType::Density,
        }
    }

    pub fn factor(&self) -> Scalar {
        Scalar::from_num(match &self {
            // Scalar
            Self::None => 1.0,
            Self::Percent => 0.01_f64,

            // Lengths
            Self::Meter => 1_000_f64,
            Self::Centimeter => 10_f64,
            Self::Millimeter => 1.0,
            Self::Micrometer => 1.0 / 1_000_f64,
            Self::Inch => 25.4_f64,
            Self::Foot => 304.8_f64,
            Self::Yard => 914.4_f64,

            // Angles
            Self::Deg | Self::DegS => 1.0 / 180. * std::f64::consts::PI,
            Self::Grad => 1.0 / 200. * std::f64::consts::PI,
            Self::Turns => 2.0 * std::f64::consts::PI,
            Self::Rad => 1.0,

            // Weights
            Self::Gram => 1.0,
            Self::Kilogram => 1_000_f64,
            Self::Pound => 453.59237_f64,
            Self::Ounce => 28.349_523_125_f64,

            // Areas
            Self::Meter2 => 1_000_000_f64,
            Self::Centimeter2 => 100_f64,
            Self::Millimeter2 => 1.0,
            Self::Micrometer2 => 0.000_000_1,
            Self::Inch2 => 645.16_f64,
            Self::Foot2 => 92_903_043.04_f64,
            Self::Yard2 => 836_127.36_f64,

            // Volumes
            Self::Meter3 => 1_000_000_000_f64,
            Self::Centimeter3 => 1_000_f64,
            Self::Millimeter3 => 1.0,
            Self::Micrometer3 => 0.000_000_000_1,
            Self::Inch3 => 16_387.06_f64,
            Self::Foot3 => 28_316_846.592_f64,
            Self::Yard3 => 764_554_857.984_f64,
            Self::Liter => 1_000_000_f64,
            Self::Centiliter => 10_000_f64,
            Self::Milliliter => 1_000_f64,
            Self::Microliter => 1_000_000.0_f64,

            // Densities
            Self::GramPerMeter3 => 1_000_000_000_f64,
            Self::GramPerMillimeter3 => 1_f64,
        })
    }

    /// Normalize value to base unit.
    pub fn normalize(self, x: Scalar) -> Scalar {
        x * self.factor()
    }

    /// Denormalize value to unit
    pub fn denormalize(self, x: Scalar) -> Scalar {
        x / self.factor()
    }
}

impl std::str::FromStr for Unit {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Scalars
            "" => Ok(Self::None),
            "%" => Ok(Self::Percent),

            // Lengths
            "m" => Ok(Self::Meter),
            "cm" => Ok(Self::Centimeter),
            "mm" => Ok(Self::Millimeter),
            "µm" => Ok(Self::Micrometer),
            "in" => Ok(Self::Inch),
            "\"" => Ok(Self::Inch),
            "ft" => Ok(Self::Foot),
            "\'" => Ok(Self::Foot),
            "yd" => Ok(Self::Yard),

            // Angles
            "deg" => Ok(Self::Deg),
            "°" => Ok(Self::DegS),
            "grad" => Ok(Self::Grad),
            "turns" => Ok(Self::Turns),
            "rad" => Ok(Self::Rad),

            // Weights
            "g" => Ok(Self::Gram),
            "kg" => Ok(Self::Kilogram),
            "lb" => Ok(Self::Pound),
            "oz" => Ok(Self::Ounce),

            // Areas
            "m²" | "m2" => Ok(Self::Meter2),
            "cm²" | "cm2" => Ok(Self::Centimeter2),
            "mm²" | "mm2" => Ok(Self::Millimeter2),
            "µm²" | "µm2" => Ok(Self::Micrometer2),
            "in²" | "in2" => Ok(Self::Inch2),
            "ft²" | "ft2" => Ok(Self::Foot2),
            "yd²" | "yd2" => Ok(Self::Yard2),

            // Volumes
            "m³" | "m3" => Ok(Self::Meter3),
            "cm³" | "cm3" => Ok(Self::Centimeter3),
            "mm³" | "mm3" => Ok(Self::Millimeter3),
            "µm³" | "µm3" => Ok(Self::Micrometer3),
            "in³" | "in3" => Ok(Self::Inch3),
            "ft³" | "ft3" => Ok(Self::Foot3),
            "yd³" | "yd3" => Ok(Self::Yard3),
            "ml" => Ok(Self::Milliliter),
            "cl" => Ok(Self::Centiliter),
            "l" => Ok(Self::Liter),
            "µl" => Ok(Self::Microliter),

            "g/mm³" => Ok(Self::GramPerMillimeter3),
            "g/m³" => Ok(Self::GramPerMeter3),

            // Unknown
            _ => Err(TypeError::UnknownUnit(s.into())),
        }
    }
}
