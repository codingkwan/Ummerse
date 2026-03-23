//! 渲染管线定义

/// 2D 渲染管线（精灵批渲染）
pub struct RenderPipeline2d {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

/// 3D 渲染管线（PBR）
pub struct RenderPipeline3d {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

/// 内置 2D 着色器（WGSL）
pub const SPRITE_SHADER_WGSL: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.uv) * in.color;
}
"#;

/// 内置 3D PBR 着色器（WGSL，简化版）
pub const PBR_SHADER_WGSL: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _pad: f32,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var t_albedo: texture_2d<f32>;
@group(2) @binding(1) var s_albedo: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_pos = world_pos.xyz;
    out.normal = normalize((model.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(t_albedo, s_albedo, in.uv);
    let light_dir = normalize(vec3<f32>(1.0, 2.0, 3.0));
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let ambient = 0.1;
    return albedo * (ambient + diffuse * 0.9);
}
"#;
