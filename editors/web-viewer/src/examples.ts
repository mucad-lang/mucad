// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

import csgCube from "../../../examples/csg_cube.mu?raw";
import drillPlate from "../../../examples/drill_plate.mu?raw";

/** An example from the repository's `examples` folder. */
export interface Example {
  name: string;
  code: string;
}

/** The examples, which can be selected. Extend this list for more examples. */
export const examples: Example[] = [
  { name: "CsgCube (3D)", code: csgCube },
  { name: "DrillPlate (2D)", code: drillPlate },
];
