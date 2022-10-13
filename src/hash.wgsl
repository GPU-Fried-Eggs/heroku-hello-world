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

// Fragment shader

fn base_hash(p: vec2<u32>) -> u32 {
	let temp32: vec2<u32> = 1103515245u * ((p >> vec2<u32>(1u)) ^ p.yx);
	let hash32: u32 = 1103515245u * (temp32.x ^ (temp32.y >> 3u));
	return hash32 ^ (hash32 >> 16u);
}

fn hash1(seed: ptr<function, f32>) -> f32 {
    let x: f32 = (*seed) + 0.1; (*seed) = x + 0.1;
	let n: u32 = base_hash(bitcast<vec2<u32>>(vec2<f32>(x, (*seed))));
	return f32(n) / f32(0xffffffffu);
}

fn hash2(seed: ptr<function, f32>) -> vec2<f32> {
    let x: f32 = (*seed) + 0.1; (*seed) = x + 0.1;
	let n: u32 = base_hash(bitcast<vec2<u32>>(vec2<f32>(x, (*seed))));
	let rz: vec2<u32> = vec2<u32>(n, n * 48271u);
	return vec2<f32>(rz.xy & vec2<u32>(0x7fffffffu)) / f32(0x7fffffffu);
}

fn hash3(seed: ptr<function, f32>) -> vec3<f32> {
    let x: f32 = (*seed) + 0.1; (*seed) = x + 0.1;
    let n: u32 = base_hash(bitcast<vec2<u32>>(vec2<f32>(x, (*seed))));
	let rz: vec3<u32> = vec3<u32>(n, n * 16807u, n * 48271u);
	return vec3<f32>(rz & vec3<u32>(0x7fffffffu)) / f32(0x7fffffffu);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var seed: f32 = f32(base_hash(bitcast<vec2<u32>>(in.uv))) / f32(0xffffffffu) + f32(system.time);
    return vec4<f32>(hash3(&seed), 1.0);
}