// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unit tests

use microcad_lang_types::{Scalar, Unit, value::approx_eq};

/// Normalized literals are equal within the rounding errors, e.g. `0.04m` is `40mm`.
#[test]
fn normalize() {
    let literal = |s: &str| Scalar::from_str(s).expect("A number");

    assert!(approx_eq(
        Unit::Meter.normalize(literal("0.04")),
        literal("40")
    ));
    assert!(approx_eq(
        Unit::Meter.normalize(literal("2.5")),
        literal("2500")
    ));
    assert!(approx_eq(
        Unit::Centimeter.normalize(literal("4")),
        literal("40")
    ));
    assert!(approx_eq(
        Unit::Inch.normalize(literal("1")),
        literal("25.4")
    ));
    // `denormalize` is the inverse of `normalize`.
    let deg = Unit::Deg.normalize(literal("90"));
    assert!(approx_eq(Unit::Deg.denormalize(deg), literal("90")));
}

/// Values are equal within the rounding errors of fixed-point arithmetic.
#[test]
fn approx_equal() {
    let literal = |s: &str| Scalar::from_str(s).expect("A number");

    // `5.1` cannot be represented exactly.
    assert!(approx_eq(literal("5.1") * 4, literal("20.4")));
    assert!(approx_eq(literal("0.1") * 1000, literal("100")));
    assert!(!approx_eq(literal("20.4"), literal("20.41")));
    assert!(!approx_eq(literal("1"), literal("1.0001")));
}

/// Area, volume and density units are consistent with their length units.
#[test]
fn unit_factors() {
    let factor = |unit: Unit| unit.normalize(Scalar::ONE).to_num::<f64>();
    let micrometer = factor(Unit::Micrometer);

    assert!((factor(Unit::Micrometer2) - micrometer.powi(2)).abs() < 1e-9);
    assert!((factor(Unit::Micrometer3) - micrometer.powi(3)).abs() < 1e-9);
    // 1µl = 1mm³
    assert_eq!(factor(Unit::Microliter), factor(Unit::Millimeter3));
    // 1g/m³ = 1g / 1_000_000_000mm³
    assert!(factor(Unit::GramPerMeter3) < 1e-8);
    assert_eq!(factor(Unit::GramPerMillimeter3), 1.0);
}
