// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later


#import bevy_pbr::forward_io::{Vertex, VertexOutput};
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world, mesh_position_world_to_clip}

@group(2) @binding(0) var<uniform> start_angle: f32; // radians
@group(2) @binding(1) var<uniform> end_angle: f32; // radians
@group(2) @binding(2) var<uniform> inner_radius: f32; // Default = 0.0
@group(2) @binding(3) var<uniform> outer_radius: f32; // Default = 1.0

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

// Adapted from iq's sdRing (https://iquilezles.org/articles/distfunctions2d/)
fn sd_ring_segment(p: vec2<f32>, n: vec2<f32>, r: f32, th: f32) -> f32 {
    var q = vec2<f32>(abs(p.x), p.y);

    // Rotate to align with direction vector `n`
    let rot = mat2x2<f32>(
        n.x,  n.y,
       -n.y,  n.x
    );
    q = rot * q;

    // Compute SDF
    let ring_dist = abs(length(q) - r) - th * 0.5;

    let clip_dist = length(vec2<f32>(
        q.x,
        max(0.0, abs(r - q.y) - th * 0.5)
    )) * sign(q.x);

    return max(ring_dist, clip_dist);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let pos = (uv - vec2<f32>(0.5)) * 2.0;

    let r = (inner_radius + outer_radius) * 0.5;
    let th = outer_radius - inner_radius;
    let center_angle = 0.5 * (start_angle + end_angle);
    let dir = vec2<f32>(cos(center_angle), sin(center_angle));

    let dist = sd_ring_segment(pos, dir, r, th);

    // Apply angular clipping separately (since original function is symmetric)
    var theta = atan2(pos.y, pos.x);
    let two_pi = 6.2831853;
    theta = (theta + two_pi) % two_pi;

    let a0 = (start_angle + two_pi) % two_pi;
    let a1 = (end_angle + two_pi) % two_pi;

    var in_sector = false;
    if (a0 <= a1) {
        in_sector = theta >= a0 && theta <= a1;
    } else {
        in_sector = theta >= a0 || theta <= a1;
    }

    let aa = fwidth(dist);
    let fill = 1.0 - smoothstep(0.0, aa, dist);
    let alpha = 0.5;

    if (in_sector) {
        let color = vec3<f32>(0.3, 0.7, 0.9);
        return vec4<f32>(color, alpha * fill);
    } else {
        return vec4<f32>(0.0);
    }
}
