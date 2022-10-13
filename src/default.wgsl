struct System {
    time: f32,
    resolution: vec2<u32>,
};
@group(0) @binding(0)
var<uniform> system: System;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv: vec2<f32> = in.uv;
    let time: f32 = system.time;
    let color: vec3<f32> = 0.5 + 0.5 * cos(vec3<f32>(time) + vec3<f32>(uv.x, uv.y, uv.x) + vec3<f32>(0.0, 2.0, 4.0));
    return vec4<f32>(color, 1.0);
}