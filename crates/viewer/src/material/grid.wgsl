// Copyright © 2025-2026 The µcad authors <info@mucad.org>
// SPDX-License-Identifier: AGPL-3.0-or-later

#import bevy_pbr::forward_io::{Vertex, VertexOutput};
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_position_local_to_world, mesh_position_world_to_clip}

struct GridConfig {
    radius: f32,
    zoom_level: f32,
    view_angle: vec3<f32>,
    grid_color: vec3<f32>,
    x_axis_color: vec3<f32>,
    y_axis_color: vec3<f32>,
}

@group(3) @binding(0) var<uniform> grid_config: GridConfig;

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    var output: VertexOutput;

    let matrix = get_world_from_local(input.instance_index);
    // Translate vertex position -0.05 in Z
    let world_pos = vec4<f32>(input.position + vec3<f32>(0.0, 0.0, -0.05 / grid_config.zoom_level), 1.0);

    output.position = mesh_position_local_to_clip(matrix, world_pos);
    output.world_position = vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    return output;
}


fn grid_opacity(pos: vec2<f32>, zoom: f32) -> vec4<f32> {
    let angle_z = pow(abs(grid_config.view_angle.z), 1.0 / 3.0);

    let logz = log2(zoom / angle_z) / log2(10.0); // log10(zoom)
    let spacing = pow(10.0, floor(logz));
    let next_spacing = spacing * 10.0;
    let blend = fract(logz); // fade factor between levels

    // World-space UVs
    let uv_a = pos / spacing;
    let uv_b = pos / next_spacing;

    // Distance to nearest grid line (centered)
    let d_a = min(
        abs(fract(uv_a.x - 0.5) - 0.5),
        abs(fract(uv_a.y - 0.5) - 0.5),
    );
    let d_b = min(
        abs(fract(uv_b.x - 0.5) - 0.5),
        abs(fract(uv_b.y - 0.5) - 0.5),
    );

    // Screen-space derivatives (anti-aliasing width)
    let fw_a = length(fwidth(uv_a)) * angle_z;
    let fw_b = length(fwidth(uv_b)) * angle_z;

    // Anti-aliased line intensities
    let line_a = 1.0 - smoothstep(0.0, fw_a, d_a);
    let line_b = 1.0 - smoothstep(0.0, fw_b, d_b);

    // Weighting to ensure brightness doesn't vary
    let w_a = 1.0 - sqrt(blend);
    let w_b = sqrt(blend);

    let grid = (line_a * w_a + line_b * w_b) / (w_a + w_b); // Always adds to 1.0 total brightness

    var color = grid_config.grid_color;
    if fw_a < abs(uv_a.x) && fw_a > abs(uv_a.y)  {
        color = grid_config.x_axis_color;
    }
    if fw_a < abs(uv_a.y) && fw_a > abs(uv_a.x) {
        color = grid_config.y_axis_color;
    }

    let fade_z = (max(angle_z / 0.8 - 0.2, 0.0));

    return vec4<f32>(color, grid * fade_z);
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let p = mesh.world_position.xy;
    let grid = grid_opacity(p, 8.0 / grid_config.zoom_level);
    let fade = min(2.0 - length(p) / grid_config.radius, 1.0);
    return vec4(grid.xyz, grid.w * fade);
}
