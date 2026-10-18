// Copyright © 2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Tests for the conversion of geometries into bevy meshes.

use mucad_driver::prelude as mu;
use mucad_viewer::to_bevy::ToBevyMesh;

/// A geometry collection results in one mesh per geometry.
#[test]
fn collection_to_meshes() {
    let line_string = mu::core::LineString::from(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
    let polygon = mu::core::Polygon::new(line_string.clone(), vec![]);
    let collection = mu::core::Geometry2D::Collection(mu::core::Geometries2D::new(vec![
        mu::core::Geometry2D::LineString(line_string),
        mu::core::Geometry2D::Polygon(polygon),
    ]));

    assert_eq!(collection.to_bevy_mesh_default().len(), 2);
}
