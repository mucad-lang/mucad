// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

#import bevy_pbr::forward_io::{Vertex, VertexOutput};
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world, mesh_position_world_to_clip}

@group(2) @binding(0) var<uniform> zoom_level: f32;

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    var output: VertexOutput;

    let matrix = get_world_from_local(input.instance_index);
    // Translate vertex position +0.1 in Z
    let world_pos = vec4<f32>(input.position + vec3<f32>(0.0, 0.0, -0.01 / zoom_level), 1.0);

    output.position = mesh_position_local_to_clip(matrix, world_pos);
    output.world_position = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    return output;
}

fn ruler_opacity(pos: vec2<f32>, zoom: f32) -> vec4<f32> {
    // Compute current scale level
    let logz = log2(zoom) / log2(10.0); // log10(zoom)
    let spacing = pow(10.0, floor(logz));       // Current tick spacing
    let next_spacing = spacing * 10.0;          // Next level spacing
    let blend = fract(logz);                    // Blend factor between scales

    // Compute tick position (world space)
    let uv_a = pos.x / spacing;
    let uv_b = pos.x / next_spacing;

    // Distance to tick lines (just in X direction)
    let d_a = abs(fract(uv_a - 0.5) - 0.5);
    let d_b = abs(fract(uv_b - 0.5) - 0.5);

    // Anti-aliasing width (only X matters)
    let fw_a = fwidth(uv_a);
    let fw_b = fwidth(uv_b);

    // Anti-aliased tick intensities
    let line_a = 1.0 - smoothstep(0.0, fw_a, d_a);
    let line_b = 1.0 - smoothstep(0.0, fw_b, d_b);

    // Blend between spacing levels
    let w_a = 1.0 - sqrt(blend);
    let w_b = sqrt(blend);
    let tick = (line_a * w_a + line_b * w_b) / (w_a + w_b);

    // Optional: tick color could change per level
    let color = vec3<f32>(1.0); // Light gray

    return vec4<f32>(color, max(tick, 0.05));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = mesh.uv;
    let ruler = ruler_opacity(p, 1.0 / 100.0);
    return ruler * vec4(1.0, 1.0, 0.0, 1.0);
}
