//! GPU 网格资源

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};

/// GPU 顶点格式
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex3d {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

/// 2D 精灵顶点
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex2d {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// GPU 网格（已上传至 GPU）
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub vertex_count: u32,
}

/// 网格构建器（Builder 模式）
pub struct MeshBuilder {
    vertices: Vec<Vertex3d>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, position: Vec3, normal: Vec3, uv: Vec2) -> &mut Self {
        self.vertices.push(Vertex3d {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        self
    }

    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) -> &mut Self {
        self.indices.extend_from_slice(&[a, b, c]);
        self
    }

    /// 创建平面网格
    pub fn quad(size: f32) -> Self {
        let h = size * 0.5;
        let mut builder = Self::new();
        builder.add_vertex(Vec3::new(-h, -h, 0.0), Vec3::Z, Vec2::new(0.0, 1.0));
        builder.add_vertex(Vec3::new( h, -h, 0.0), Vec3::Z, Vec2::new(1.0, 1.0));
        builder.add_vertex(Vec3::new( h,  h, 0.0), Vec3::Z, Vec2::new(1.0, 0.0));
        builder.add_vertex(Vec3::new(-h,  h, 0.0), Vec3::Z, Vec2::new(0.0, 0.0));
        builder.add_triangle(0, 1, 2);
        builder.add_triangle(0, 2, 3);
        builder
    }

    /// 创建立方体网格
    pub fn cube(size: f32) -> Self {
        let h = size * 0.5;
        let mut builder = Self::new();
        // 简化：只有一个面（完整版需要6个面）
        // 前面 (+Z)
        builder.add_vertex(Vec3::new(-h, -h,  h), Vec3::Z, Vec2::new(0.0, 1.0));
        builder.add_vertex(Vec3::new( h, -h,  h), Vec3::Z, Vec2::new(1.0, 1.0));
        builder.add_vertex(Vec3::new( h,  h,  h), Vec3::Z, Vec2::new(1.0, 0.0));
        builder.add_vertex(Vec3::new(-h,  h,  h), Vec3::Z, Vec2::new(0.0, 0.0));
        builder.add_triangle(0, 1, 2);
        builder.add_triangle(0, 2, 3);
        builder
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn vertex_data(&self) -> &[Vertex3d] {
        &self.vertices
    }

    pub fn index_data(&self) -> &[u32] {
        &self.indices
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}
