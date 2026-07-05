struct Camera {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;

struct Object {
    model: mat4x4<f32>,
};
@group(2) @binding(0) var<uniform> object: Object;

@group(1) @binding(0) var albedo_texture: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;

struct MaterialUniform {
    tint: vec4<f32>,
};
@group(1) @binding(2) var<uniform> material: MaterialUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = object.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(albedo_texture, albedo_sampler, in.uv);
    return sampled * material.tint;
}