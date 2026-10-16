// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::geo3d::*;

part CylinderStack(heights: [Length], radii: [Length], offset: Length = 0mm) {
    assert(
        heights.count() == radii.count() - 1,
        "radii must be an array with one more elements that heights",
    );

    self = Cylinder(
        height = heights.head(),
        radius_bottom = radii.head(),
        radius_top = radii.tail().head(),
        offset,
    );
    if heights.count() > 1 {
        self | CylinderStack(
            heights = heights.tail(),
            radii = radii.tail(),
            offset = offset + heights.head(),
        )
    } else {
        self
    }
}

CylinderStack(heights = [5, 15, 5]mm, radii = [6, 4, 4, 6]mm);
