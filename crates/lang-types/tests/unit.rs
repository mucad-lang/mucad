// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unit tests

use microcad_lang_types::{Scalar, Unit};

/// A value is normalized to the base unit before it is rounded, e.g. `0.04m` is exactly `40mm`.
#[test]
fn normalize_f64() {
    assert_eq!(Unit::Meter.normalize_f64(0.04), Scalar::from_num(40));
    assert_eq!(Unit::Meter.normalize_f64(2.5), Scalar::from_num(2500));
    assert_eq!(Unit::Centimeter.normalize_f64(4.0), Scalar::from_num(40));
    assert_eq!(Unit::Inch.normalize_f64(1.0), Scalar::from_num(25.4));
}

/// Area, volume and density units are consistent with their length units.
#[test]
fn unit_factors() {
    let factor = |unit: Unit| unit.factor().to_num::<f64>();
    let micrometer = factor(Unit::Micrometer);

    assert!((factor(Unit::Micrometer2) - micrometer.powi(2)).abs() < 1e-9);
    assert!((factor(Unit::Micrometer3) - micrometer.powi(3)).abs() < 1e-9);
    // 1µl = 1mm³
    assert_eq!(factor(Unit::Microliter), factor(Unit::Millimeter3));
    // 1g/m³ = 1g / 1_000_000_000mm³
    assert!(factor(Unit::GramPerMeter3) < 1e-8);
    assert_eq!(factor(Unit::GramPerMillimeter3), 1.0);
}
