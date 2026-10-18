// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! µcad unit syntax element.

use serde::{Deserialize, Serialize};

use crate::{QuantityType, Scalar, Ty, Type, TypeError};

/// A fixed-point number with more fractional bits than a [`Scalar`] for the factors of units.
///
/// The integer part has 31 bits for factors up to `1_000_000_000` (`m³`).
type Factor = fixed::FixedI128<fixed::types::extra::U96>;

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

    /// The factor to the base unit, e.g. `1000` from `m` to `mm`.
    ///
    /// The factor has more fractional bits than a [`Scalar`], e.g. for `µm` or `°`.
    fn factor(&self) -> Factor {
        match &self {
            // Scalar
            Self::None => Factor::ONE,
            Self::Percent => Factor::lit("0.01"),

            // Lengths
            Self::Meter => Factor::lit("1000"),
            Self::Centimeter => Factor::lit("10"),
            Self::Millimeter => Factor::ONE,
            Self::Micrometer => Factor::lit("0.001"),
            Self::Inch => Factor::lit("25.4"),
            Self::Foot => Factor::lit("304.8"),
            Self::Yard => Factor::lit("914.4"),

            // Angles
            Self::Deg | Self::DegS => Factor::PI / 180,
            Self::Grad => Factor::PI / 200,
            Self::Turns => Factor::TAU,
            Self::Rad => Factor::ONE,

            // Weights
            Self::Gram => Factor::ONE,
            Self::Kilogram => Factor::lit("1000"),
            Self::Pound => Factor::lit("453.59237"),
            Self::Ounce => Factor::lit("28.349523125"),

            // Areas
            Self::Meter2 => Factor::lit("1000000"),
            Self::Centimeter2 => Factor::lit("100"),
            Self::Millimeter2 => Factor::ONE,
            Self::Micrometer2 => Factor::lit("0.000001"),
            Self::Inch2 => Factor::lit("645.16"),
            Self::Foot2 => Factor::lit("92903.04"),
            Self::Yard2 => Factor::lit("836127.36"),

            // Volumes
            Self::Meter3 => Factor::lit("1000000000"),
            Self::Centimeter3 => Factor::lit("1000"),
            Self::Millimeter3 => Factor::ONE,
            Self::Micrometer3 => Factor::lit("0.000000001"),
            Self::Inch3 => Factor::lit("16387.064"),
            Self::Foot3 => Factor::lit("28316846.592"),
            Self::Yard3 => Factor::lit("764554857.984"),
            Self::Liter => Factor::lit("1000000"),
            Self::Centiliter => Factor::lit("10000"),
            Self::Milliliter => Factor::lit("1000"),
            Self::Microliter => Factor::ONE,

            // Densities
            Self::GramPerMeter3 => Factor::lit("0.000000001"),
            Self::GramPerMillimeter3 => Factor::ONE,
        }
    }

    /// The number of decimals of a value in this unit within the resolution of a [`Scalar`],
    /// e.g. 9 for `mm` and 7 for `°`.
    pub fn decimals(&self) -> usize {
        // In `f64`, because the resolution is smaller than a `Scalar` for large factors.
        let resolution = Scalar::DELTA.to_num::<f64>() / self.factor().to_num::<f64>();
        (-resolution.log10()).floor().max(0.0) as usize
    }

    /// Normalize value to base unit.
    ///
    /// The product with the precise factor is rounded only once, e.g. `1000µm` is `1mm`.
    pub fn normalize(self, x: Scalar) -> Scalar {
        Scalar::ZERO.add_prod(x, self.factor())
    }

    /// Denormalize value to unit, the inverse of [`Unit::normalize`].
    pub fn denormalize(self, x: Scalar) -> Scalar {
        Scalar::ZERO.add_prod(x, Factor::ONE / self.factor())
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
