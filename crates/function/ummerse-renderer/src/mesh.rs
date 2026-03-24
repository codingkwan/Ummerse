//! GPU 网格资源 - 顶点、索引缓冲区 + 几何体生成器

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use std::f32::consts::PI;

// ── 顶点格式 ──────────────────────────────────────────────────────────────────

/// 3D 顶点格式（位置 + 法线 + UV + 切线）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex3d {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// 切线（xyz = 切线方向，w = 副切线手性 ±1）
    pub tangent: [f32; 4],
}

impl Vertex3d {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
            tangent: [1.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn with_tangent(mut self, tangent: [f32; 4]) -> Self {
        self.tangent = tangent;
        self
    }
}

/// 2D 精灵顶点格式（位置 + UV + 顶点颜色）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex2d {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2d {
    pub fn new(position: Vec2, uv: Vec2, color: [f32; 4]) -> Self {
        Self {
            position: position.to_array(),
            uv: uv.to_array(),
            color,
        }
    }
}

// ── GPU 网格 ──────────────────────────────────────────────────────────────────

/// GPU 网格（已上传至 GPU 的顶点/索引缓冲区）
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub vertex_count: u32,
    /// AABB 包围盒（用于 CPU 剔除，世界空间）
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
}

impl std::fmt::Debug for GpuMesh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuMesh")
            .field("index_count", &self.index_count)
            .field("vertex_count", &self.vertex_count)
            .field("aabb_min", &self.aabb_min)
            .field("aabb_max", &self.aabb_max)
            .finish_non_exhaustive()
    }
}

impl GpuMesh {
    /// 从 MeshData 上传到 GPU
    pub fn from_mesh_data(device: &wgpu::Device, data: &MeshData, label: &str) -> Self {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_vb", label)),
            contents: bytemuck::cast_slice(&data.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_ib", label)),
            contents: bytemuck::cast_slice(&data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let (aabb_min, aabb_max) = data.compute_aabb();

        Self {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
            vertex_count: data.vertices.len() as u32,
            aabb_min: aabb_min.to_array(),
            aabb_max: aabb_max.to_array(),
        }
    }

    /// 从 2D 顶点数据上传到 GPU
    pub fn from_2d_data(device: &wgpu::Device, data: &MeshData2d, label: &str) -> GpuMesh2d {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_vb", label)),
            contents: bytemuck::cast_slice(&data.vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{}_ib", label)),
            contents: bytemuck::cast_slice(&data.indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        GpuMesh2d {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
            vertex_count: data.vertices.len() as u32,
        }
    }
}

/// 2D GPU 网格
pub struct GpuMesh2d {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub vertex_count: u32,
}

impl std::fmt::Debug for GpuMesh2d {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuMesh2d")
            .field("index_count", &self.index_count)
            .field("vertex_count", &self.vertex_count)
            .finish_non_exhaustive()
    }
}

// ── 网格 CPU 数据 ─────────────────────────────────────────────────────────────

/// CPU 侧 3D 网格数据（用于生成和修改网格）
#[derive(Debug)]
pub struct MeshData {
    pub vertices: Vec<Vertex3d>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, position: Vec3, normal: Vec3, uv: Vec2) -> u32 {
        let idx = self.vertices.len() as u32;
        self.vertices.push(Vertex3d::new(position, normal, uv));
        idx
    }

    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    /// 计算 AABB 包围盒
    pub fn compute_aabb(&self) -> (Vec3, Vec3) {
        if self.vertices.is_empty() {
            return (Vec3::ZERO, Vec3::ZERO);
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for v in &self.vertices {
            let p = Vec3::from(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        (min, max)
    }

    /// 计算顶点切线（用于法线贴图）
    pub fn compute_tangents(&mut self) {
        for i in (0..self.indices.len()).step_by(3) {
            let i0 = self.indices[i] as usize;
            let i1 = self.indices[i + 1] as usize;
            let i2 = self.indices[i + 2] as usize;

            let v0 = Vec3::from(self.vertices[i0].position);
            let v1 = Vec3::from(self.vertices[i1].position);
            let v2 = Vec3::from(self.vertices[i2].position);

            let uv0 = Vec2::from(self.vertices[i0].uv);
            let uv1 = Vec2::from(self.vertices[i1].uv);
            let uv2 = Vec2::from(self.vertices[i2].uv);

            let delta_pos1 = v1 - v0;
            let delta_pos2 = v2 - v0;
            let delta_uv1 = uv1 - uv0;
            let delta_uv2 = uv2 - uv0;

            let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x + 1e-8);
            let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

            for &idx in &[i0, i1, i2] {
                self.vertices[idx].tangent = [tangent.x, tangent.y, tangent.z, 1.0];
            }
        }
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU 侧 2D 网格数据
#[derive(Debug)]
pub struct MeshData2d {
    pub vertices: Vec<Vertex2d>,
    pub indices: Vec<u32>,
}

impl MeshData2d {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl Default for MeshData2d {
    fn default() -> Self {
        Self::new()
    }
}

// ── 几何体构建器 ──────────────────────────────────────────────────────────────

/// 网格构建器（Builder 模式，生成标准几何体）
#[derive(Debug)]
pub struct MeshBuilder {
    data: MeshData,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            data: MeshData::new(),
        }
    }

    pub fn add_vertex(&mut self, position: Vec3, normal: Vec3, uv: Vec2) -> &mut Self {
        self.data.add_vertex(position, normal, uv);
        self
    }

    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) -> &mut Self {
        self.data.add_triangle(a, b, c);
        self
    }

    pub fn vertex_count(&self) -> u32 {
        self.data.vertices.len() as u32
    }

    pub fn index_count(&self) -> u32 {
        self.data.indices.len() as u32
    }

    pub fn vertex_data(&self) -> &[Vertex3d] {
        &self.data.vertices
    }

    pub fn index_data(&self) -> &[u32] {
        &self.data.indices
    }

    /// 生成最终 MeshData（自动计算切线）
    pub fn build(mut self) -> MeshData {
        self.data.compute_tangents();
        self.data
    }

    // ── 标准几何体 ─────────────────────────────────────────────────────────

    /// 四边形平面（中心在原点，XY 平面）
    pub fn quad(size: f32) -> Self {
        let h = size * 0.5;
        let mut b = Self::new();
        // 4 个顶点
        b.data
            .add_vertex(Vec3::new(-h, -h, 0.0), Vec3::Z, Vec2::new(0.0, 1.0));
        b.data
            .add_vertex(Vec3::new(h, -h, 0.0), Vec3::Z, Vec2::new(1.0, 1.0));
        b.data
            .add_vertex(Vec3::new(h, h, 0.0), Vec3::Z, Vec2::new(1.0, 0.0));
        b.data
            .add_vertex(Vec3::new(-h, h, 0.0), Vec3::Z, Vec2::new(0.0, 0.0));
        b.data.add_triangle(0, 1, 2);
        b.data.add_triangle(0, 2, 3);
        b
    }

    /// 完整立方体（6 面，共 24 顶点，36 索引）
    pub fn cube(size: f32) -> Self {
        let h = size * 0.5;
        let mut b = Self::new();

        // 每个面的法线和顶点顺序
        let faces: &[(Vec3, [Vec3; 4])] = &[
            // 前面 (+Z)
            (
                Vec3::Z,
                [
                    Vec3::new(-h, -h, h),
                    Vec3::new(h, -h, h),
                    Vec3::new(h, h, h),
                    Vec3::new(-h, h, h),
                ],
            ),
            // 后面 (-Z)
            (
                Vec3::NEG_Z,
                [
                    Vec3::new(h, -h, -h),
                    Vec3::new(-h, -h, -h),
                    Vec3::new(-h, h, -h),
                    Vec3::new(h, h, -h),
                ],
            ),
            // 上面 (+Y)
            (
                Vec3::Y,
                [
                    Vec3::new(-h, h, h),
                    Vec3::new(h, h, h),
                    Vec3::new(h, h, -h),
                    Vec3::new(-h, h, -h),
                ],
            ),
            // 下面 (-Y)
            (
                Vec3::NEG_Y,
                [
                    Vec3::new(-h, -h, -h),
                    Vec3::new(h, -h, -h),
                    Vec3::new(h, -h, h),
                    Vec3::new(-h, -h, h),
                ],
            ),
            // 右面 (+X)
            (
                Vec3::X,
                [
                    Vec3::new(h, -h, h),
                    Vec3::new(h, -h, -h),
                    Vec3::new(h, h, -h),
                    Vec3::new(h, h, h),
                ],
            ),
            // 左面 (-X)
            (
                Vec3::NEG_X,
                [
                    Vec3::new(-h, -h, -h),
                    Vec3::new(-h, -h, h),
                    Vec3::new(-h, h, h),
                    Vec3::new(-h, h, -h),
                ],
            ),
        ];

        let uvs = [
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];

        for (normal, verts) in faces {
            let base = b.data.vertices.len() as u32;
            for (i, &pos) in verts.iter().enumerate() {
                b.data.add_vertex(pos, *normal, uvs[i]);
            }
            b.data.add_triangle(base, base + 1, base + 2);
            b.data.add_triangle(base, base + 2, base + 3);
        }
        b
    }

    /// 球体（UV 球，longitude × latitude 细分）
    pub fn sphere(radius: f32, longitude_segments: u32, latitude_segments: u32) -> Self {
        let mut b = Self::new();

        for lat in 0..=latitude_segments {
            let theta = lat as f32 * PI / latitude_segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=longitude_segments {
                let phi = lon as f32 * 2.0 * PI / longitude_segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = cos_phi * sin_theta;
                let y = cos_theta;
                let z = sin_phi * sin_theta;

                let pos = Vec3::new(x, y, z) * radius;
                let normal = Vec3::new(x, y, z);
                let uv = Vec2::new(
                    lon as f32 / longitude_segments as f32,
                    lat as f32 / latitude_segments as f32,
                );
                b.data.add_vertex(pos, normal, uv);
            }
        }

        for lat in 0..latitude_segments {
            for lon in 0..longitude_segments {
                let first = lat * (longitude_segments + 1) + lon;
                let second = first + longitude_segments + 1;
                b.data.add_triangle(first, second, first + 1);
                b.data.add_triangle(second, second + 1, first + 1);
            }
        }
        b
    }

    /// 胶囊体（2D 圆形 + 顶/底半球）
    pub fn capsule(radius: f32, height: f32, segments: u32) -> Self {
        // 简化版：使用圆柱体近似
        Self::cylinder(radius, height, segments)
    }

    /// 圆柱体
    pub fn cylinder(radius: f32, height: f32, segments: u32) -> Self {
        let mut b = Self::new();
        let h = height * 0.5;
        let step = 2.0 * PI / segments as f32;

        // 上下顶面中心
        let top_center = b
            .data
            .add_vertex(Vec3::new(0.0, h, 0.0), Vec3::Y, Vec2::new(0.5, 0.5));
        let bot_center =
            b.data
                .add_vertex(Vec3::new(0.0, -h, 0.0), Vec3::NEG_Y, Vec2::new(0.5, 0.5));

        // 侧面 + 顶/底边缘
        for i in 0..segments {
            let angle = i as f32 * step;
            let nx = angle.cos();
            let nz = angle.sin();

            let top_uv = Vec2::new(i as f32 / segments as f32, 0.0);
            let bot_uv = Vec2::new(i as f32 / segments as f32, 1.0);

            b.data
                .add_vertex(Vec3::new(nx * radius, h, nz * radius), Vec3::Y, top_uv);
            b.data
                .add_vertex(Vec3::new(nx * radius, -h, nz * radius), Vec3::NEG_Y, bot_uv);
            // 侧面顶点（带侧向法线）
            b.data.add_vertex(
                Vec3::new(nx * radius, h, nz * radius),
                Vec3::new(nx, 0.0, nz),
                top_uv,
            );
            b.data.add_vertex(
                Vec3::new(nx * radius, -h, nz * radius),
                Vec3::new(nx, 0.0, nz),
                bot_uv,
            );
        }

        let base = 2u32; // top_center=0, bot_center=1
        for i in 0..segments {
            let next = (i + 1) % segments;
            let stride = 4u32;
            let cur_top = base + i * stride;
            let cur_bot = base + i * stride + 1;
            let next_top = base + next * stride;
            let next_bot = base + next * stride + 1;
            let cur_side_top = base + i * stride + 2;
            let cur_side_bot = base + i * stride + 3;
            let next_side_top = base + next * stride + 2;
            let next_side_bot = base + next * stride + 3;

            // 顶盖
            b.data.add_triangle(top_center, cur_top, next_top);
            // 底盖
            b.data.add_triangle(bot_center, next_bot, cur_bot);
            // 侧面（2 个三角形）
            b.data
                .add_triangle(cur_side_top, next_side_top, cur_side_bot);
            b.data
                .add_triangle(next_side_top, next_side_bot, cur_side_bot);
        }
        b
    }

    /// 圆锥体
    pub fn cone(radius: f32, height: f32, segments: u32) -> Self {
        let mut b = Self::new();
        let tip = b
            .data
            .add_vertex(Vec3::new(0.0, height, 0.0), Vec3::Y, Vec2::new(0.5, 0.5));
        let center = b
            .data
            .add_vertex(Vec3::ZERO, Vec3::NEG_Y, Vec2::new(0.5, 0.5));
        let step = 2.0 * PI / segments as f32;

        for i in 0..segments {
            let a0 = i as f32 * step;
            let a1 = (i + 1) as f32 * step;
            let x0 = a0.cos() * radius;
            let z0 = a0.sin() * radius;
            let x1 = a1.cos() * radius;
            let z1 = a1.sin() * radius;
            let uv0 = Vec2::new(a0 / (2.0 * PI), 1.0);
            let uv1 = Vec2::new(a1 / (2.0 * PI), 1.0);
            let side_n0 = Vec3::new(x0, radius / height, z0).normalize_or_zero();
            let side_n1 = Vec3::new(x1, radius / height, z1).normalize_or_zero();

            let v0 = b.data.add_vertex(Vec3::new(x0, 0.0, z0), side_n0, uv0);
            let v1 = b.data.add_vertex(Vec3::new(x1, 0.0, z1), side_n1, uv1);
            let v0b = b.data.add_vertex(Vec3::new(x0, 0.0, z0), Vec3::NEG_Y, uv0);
            let v1b = b.data.add_vertex(Vec3::new(x1, 0.0, z1), Vec3::NEG_Y, uv1);

            // 侧面
            b.data.add_triangle(tip, v0, v1);
            // 底盖
            b.data.add_triangle(center, v1b, v0b);
        }
        b
    }

    /// 平面（XZ 平面，Y 向上，细分网格）
    pub fn plane(width: f32, depth: f32, divisions_x: u32, divisions_z: u32) -> Self {
        let mut b = Self::new();
        let hw = width * 0.5;
        let hd = depth * 0.5;
        let cols = divisions_x + 1;
        let rows = divisions_z + 1;

        for z in 0..rows {
            for x in 0..cols {
                let tx = x as f32 / divisions_x as f32;
                let tz = z as f32 / divisions_z as f32;
                let px = -hw + tx * width;
                let pz = -hd + tz * depth;
                b.data
                    .add_vertex(Vec3::new(px, 0.0, pz), Vec3::Y, Vec2::new(tx, tz));
            }
        }

        for z in 0..divisions_z {
            for x in 0..divisions_x {
                let tl = z * cols + x;
                let tr = tl + 1;
                let bl = tl + cols;
                let br = bl + 1;
                b.data.add_triangle(tl, bl, tr);
                b.data.add_triangle(tr, bl, br);
            }
        }
        b
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}
