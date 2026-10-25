// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: MIT

//! 5 Torus rings produced via list expansion.

std::geo3d::Torus([1..5] * 10mm, minor_radius = 3mm);
