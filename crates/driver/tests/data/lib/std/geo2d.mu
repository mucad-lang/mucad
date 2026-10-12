// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

/// A circle with a radius.
pub sketch Circle(radius: Length) {
    __mu::geo2d::Circle(radius = radius);
}
