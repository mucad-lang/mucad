// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later


#import bevy_pbr::forward_io::{Vertex, VertexOutput};
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world, mesh_position_world_to_clip}

@group(2) @binding(0) var<uniform> arrow_start_enabled: u32;
@group(2) @binding(1) var<uniform> arrow_end_enabled: u32;
@group(2) @binding(2) var<uniform> arrow_head_size: f32;

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    var output: VertexOutput;

    let matrix = get_world_from_local(input.instance_index);
    let world_pos = vec4<f32>(input.position, 1.0);

    output.position = mesh_position_local_to_clip(matrix, world_pos);
    output.world_position = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    return output;
}


// Simple SDF for a triangle (used for arrow heads)
fn triangle_sdf(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> f32 {
    let ba = b - a;
    let pa = p - a;
    let cb = c - b;
    let pb = p - b;
    let ac = a - c;
    let pc = p - c;

    let s = sign(ba.x * pa.y - ba.y * pa.x)
          + sign(cb.x * pb.y - cb.y * pb.x)
          + sign(ac.x * pc.y - ac.y * pc.x);

    let inside = s >= 2.0;

    // Distance to triangle edges
    let d0 = dot(pa - ba * clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0), pa - ba * clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0));
    let d1 = dot(pb - cb * clamp(dot(pb, cb) / dot(cb, cb), 0.0, 1.0), pb - cb * clamp(dot(pb, cb) / dot(cb, cb), 0.0, 1.0));
    let d2 = dot(pc - ac * clamp(dot(pc, ac) / dot(ac, ac), 0.0, 1.0), pc - ac * clamp(dot(pc, ac) / dot(ac, ac), 0.0, 1.0));

    let dist = sqrt(min(min(d0, d1), d2));
    return if inside { -dist } else { dist };
}

// Main SDF for full arrow (shaft + optional heads)
fn arrow_sdf(uv: vec2<f32>) -> f32 {
    let p = (uv - vec2<f32>(0.5)) * 2.0;

    let head_size = clamp(arrow_head_size, 0.0, 0.49);
    let shaft_length = 1.0 - 2.0 * head_size;
    let shaft_width = 0.1;

    // Shaft rectangle
    let shaft_center = vec2<f32>(0.0, 0.0);
    let half_shaft = vec2<f32>(shaft_length * 0.5, shaft_width);
    let d_shaft = max(abs(p - shaft_center) - half_shaft, vec2<f32>(0.0));
    let dist_shaft = length(d_shaft);

    var dist = dist_shaft;

    // Start arrow head (left)
    if arrow_start_enabled != 0u {
        let tip = vec2<f32>(-1.0 + head_size, 0.0);
        let left = tip + vec2<f32>(head_size,  head_size);
        let right = tip + vec2<f32>(head_size, -head_size);
        let d_start = triangle_sdf(p, tip, left, right);
        dist = min(dist, d_start);
    }

    // End arrow head (right)
    if arrow_end_enabled != 0u {
        let tip = vec2<f32>(1.0 - head_size, 0.0);
        let left = tip - vec2<f32>(head_size,  head_size);
        let right = tip - vec2<f32>(head_size, -head_size);
        let d_end = triangle_sdf(p, tip, left, right);
        dist = min(dist, d_end);
    }

    return dist;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let dist = arrow_sdf(mesh.uv);
    let aa = fwidth(dist);
    let alpha = 1.0 - smoothstep(0.0, aa, dist);
    let color = vec3<f32>(0.1, 0.1, 0.1); // Dark arrow

    return vec4<f32>(color, alpha);
}
