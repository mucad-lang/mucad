// Copyright © 2026 The µcad authors <info@microcad.xyz>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Render output type.

use std::sync::Arc;

use cgmath::SquareMatrix;

use microcad_core::{
    self as core, Bounds2D, Bounds3D, CalcBounds3D, Geometry3D, GeometryType, Mat4, Transformed3D,
    traits::TransformAffine,
};

use microcad_hash::{HashId, ToHash, hash_id};
use microcad_lang_types::{
    ModelNodeRef, ModelTree,
    math::IntoFloat,
    model::{ModelNodeId, ModelType, NodeExt},
};

use crate::{RenderAttributes, RenderResolution};

/// Geometry output to be stored in the render cache.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct GeometryOutput {
    pub id: Option<String>,
    pub attr: RenderAttributes,
    pub geometry: core::Geometry,
    pub bounds: core::Bounds3D,
}

/// Builder methods.
impl GeometryOutput {
    pub fn new(geometry: impl Into<core::Geometry>) -> Self {
        let geometry = geometry.into();
        let bounds = geometry.calc_bounds_3d();
        Self {
            id: None,
            attr: Default::default(),
            geometry,
            bounds,
        }
    }

    pub fn geo2d(geo2d: impl Into<core::Geometry2D>) -> Self {
        Self::new(geo2d.into())
    }

    pub fn geo3d(geo3d: impl Into<core::Geometry3D>) -> Self {
        Self::new(geo3d.into())
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_attr(mut self, attr: RenderAttributes) -> Self {
        self.attr = attr;
        self
    }
}

/// Accessors.
impl GeometryOutput {
    /// Return GeometryType.
    pub fn ty(&self) -> GeometryType {
        self.geometry.ty()
    }

    /// The radius of a centered circle that wraps the output geometries bounds on the ground.
    pub fn ground_radius(&self) -> core::Length {
        let mut bounds = Bounds2D::new(self.bounds.min.truncate(), self.bounds.max.truncate());
        bounds.extend_by_point(core::Vec2::new(0.0, 0.0));
        core::Length::mm(bounds.radius())
    }

    /// The radius of a centered sphere, that wrap the geometries bounds.
    pub fn scene_radius(&self) -> core::Length {
        let mut bounds = self.bounds.clone();
        bounds.extend_by_point(core::Vec3::new(0.0, 0.0, 0.0));
        core::Length::mm(bounds.radius())
    }
}

impl From<core::Geometry> for GeometryOutput {
    fn from(geometry: core::Geometry) -> Self {
        let bounds = geometry.calc_bounds_3d();
        Self {
            id: None,
            attr: Default::default(),
            geometry,
            bounds,
        }
    }
}

impl TransformAffine for GeometryOutput {
    fn transform_affine(&mut self, m: &Mat4) {
        self.geometry.transform_affine(m);
        self.bounds = self.geometry.calc_bounds_3d();
    }
}

#[derive(Debug, Default, Clone)]
pub struct GeometryOutputs {
    outputs: Vec<Arc<GeometryOutput>>,
    geometry_type: GeometryType,
    bounds: Bounds3D,
}

impl GeometryOutputs {
    /// Returns an iterator over references to the output items.
    pub fn iter(&self) -> std::slice::Iter<'_, Arc<GeometryOutput>> {
        self.outputs.iter()
    }

    /// Returns the number of geometry outputs.
    #[inline]
    pub fn len(&self) -> usize {
        self.outputs.len()
    }

    /// Returns `true` if there are no geometry outputs.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.outputs.is_empty()
    }

    /// Return all outputs with no name.
    pub fn primary(self) -> Self {
        Self::from_iter(self.into_iter().filter(|geo| geo.id.is_none()))
    }

    /// Extract 2D geometries from outputs
    pub fn to_2d(&self) -> core::Geometries2D {
        // TODO geometry type check

        core::Geometries2D::new(
            self.outputs
                .iter()
                .filter_map(|output| match &output.geometry {
                    core::Geometry::Geometry2D(geo2d) => Some(geo2d.clone()),
                    core::Geometry::Geometry3D(_) => None,
                })
                .collect(),
        )
    }

    /// Extract 3D geometries from outputs
    pub fn to_3d(&self) -> core::Geometries3D {
        // TODO geometry type check

        core::Geometries3D::new(
            self.outputs
                .iter()
                .filter_map(|output| match &output.geometry {
                    core::Geometry::Geometry2D(_) => None,
                    core::Geometry::Geometry3D(geo3d) => Some(geo3d.clone()),
                })
                .collect(),
        )
    }

    /// Return geometry type.
    pub fn ty(&self) -> core::GeometryType {
        self.geometry_type
    }

    /// Bounds
    pub fn bounds(&self) -> &Bounds3D {
        &self.bounds
    }
}

impl From<GeometryOutput> for GeometryOutputs {
    fn from(geometry: GeometryOutput) -> Self {
        let geometry_type = geometry.ty();
        let bounds = geometry.bounds.clone();
        Self {
            outputs: vec![Arc::new(geometry)],
            geometry_type,
            bounds,
        }
    }
}

impl From<core::Geometry> for GeometryOutputs {
    fn from(geo: core::Geometry) -> Self {
        let output = GeometryOutput::from(geo);
        Self::from(output)
    }
}

impl std::fmt::Display for GeometryOutputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if matches!(self.geometry_type, GeometryType::Empty) {
            write!(f, "[]")
        } else {
            write!(f, "{type}[{len}]", type = self.geometry_type, len = self.len())
        }
    }
}

// Implement IntoIterator for &GeometryOutputs so callers can write `for output in &outputs`
impl IntoIterator for GeometryOutputs {
    type Item = Arc<GeometryOutput>;
    type IntoIter = std::vec::IntoIter<Arc<GeometryOutput>>;

    fn into_iter(self) -> Self::IntoIter {
        self.outputs.into_iter()
    }
}

impl FromIterator<Arc<GeometryOutput>> for GeometryOutputs {
    fn from_iter<T: IntoIterator<Item = Arc<GeometryOutput>>>(iter: T) -> Self {
        let outputs: Vec<_> = iter.into_iter().collect();
        let geometry_type = outputs
            .iter()
            .fold(GeometryType::Empty, |acc, item| acc.merge(item.ty()));
        let bounds = outputs
            .iter()
            .fold(Bounds3D::default(), |bounds, geometry| {
                bounds.extend(geometry.bounds.clone())
            });

        Self {
            outputs,
            geometry_type,
            bounds,
        }
    }
}

impl<T> From<core::WithBounds3D<T>> for GeometryOutput
where
    T: Into<Geometry3D> + CalcBounds3D + Transformed3D,
{
    fn from(with_bounds: core::WithBounds3D<T>) -> Self {
        GeometryOutput {
            id: None,
            geometry: core::Geometry::from(with_bounds.inner.into()),
            bounds: with_bounds.bounds,
            attr: Default::default(),
        }
    }
}

impl TransformAffine for GeometryOutputs {
    fn transform_affine(&mut self, m: &Mat4) {
        let mut new_bounds = Bounds3D::default();
        for output_arc in &mut self.outputs {
            // Arc::make_mut clones the inner GeometryOutput ONLY if shared.
            // Otherwise, it provides a direct &mut reference.
            let output = Arc::make_mut(output_arc);
            output.transform_affine(m);

            // Accumulate updated bounds
            new_bounds = new_bounds.extend(output.bounds.clone());
        }
        self.bounds = new_bounds;
    }
}

#[derive(Debug, Clone)]
pub struct GeometryNodeData {
    /// The output (2D/3D) this render output is expected to produce.
    pub output_type: ModelType,
    /// Local transformation matrix.
    pub local_matrix: core::Mat4,
    /// World transformation matrix.
    pub world_matrix: core::Mat4,
    /// The render resolution, calculated from transformation matrix.
    pub resolution: Option<RenderResolution>,
    /// The output geometry.
    pub outputs: GeometryOutputs,
    /// Computed model hash.
    pub hash: HashId,

    pub model_node_id: ModelNodeId,
}

impl GeometryNodeData {
    /// Create new render output for model.
    pub fn new<'tree>(model: ModelNodeRef<'tree>) -> Self {
        // Groups and operations like `difference` take the output type of their children.
        let output_type = model.deduce_output_type();
        let hash = hash_id!(model);
        let local_matrix = model.local_matrix().into_float();

        GeometryNodeData {
            output_type,
            local_matrix,
            world_matrix: Mat4::identity(),
            resolution: None,
            outputs: Default::default(),
            hash,
            model_node_id: model.id,
        }
    }

    pub fn model<'a>(&self, tree: &'a ModelTree) -> ModelNodeRef<'a> {
        ModelNodeRef::new(self.model_node_id, &tree.arena)
    }

    /// Get render resolution.
    pub fn resolution(&self) -> &Option<RenderResolution> {
        &self.resolution
    }
}

impl ToHash for GeometryNodeData {
    fn to_hash(&self) -> HashId {
        self.hash
    }
}
