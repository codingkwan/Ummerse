//! # Ummerse Renderer
//!
//! 2D/3D 渲染管线，基于 wgpu：
//! - 2D 精灵批渲染（正交相机 + Alpha 混合）
//! - 3D PBR 渲染管线（Cook-Torrance BRDF）
//! - 后处理（ACES 色调映射 + Gamma 校正）
//! - 渲染图（声明式 Pass 编排）
//! - 几何体生成器（Cube/Sphere/Cylinder 等）

pub mod camera;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod render_graph;
pub mod texture;

pub use camera::{Camera2d, Camera3d, CameraProjection, CameraUniform2d, CameraUniform3d};
pub use material::{
    AlphaMode, DirectionalLightUniform, Material, MaterialId, PbrMaterial, PbrMaterialUniform,
    UnlitMaterial,
};
pub use mesh::{GpuMesh, GpuMesh2d, MeshBuilder, MeshData, MeshData2d, Vertex2d, Vertex3d};
pub use pipeline::{
    PBR_SHADER_WGSL, POST_PROCESS_SHADER_WGSL, PostProcessPipeline, RenderPipeline2d,
    RenderPipeline3d, SPRITE_SHADER_WGSL,
};
pub use render_graph::{FrameContext, PassKind, RenderGraph, RenderNode, ResourceId};
pub use texture::{GpuTexture, SamplerMode};

/// 渲染器配置
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// MSAA 采样数（1/2/4/8）
    pub sample_count: u32,
    /// 是否启用 HDR
    pub hdr: bool,
    /// 阴影贴图尺寸（512/1024/2048/4096）
    pub shadow_map_size: u32,
    /// 最大动态光源数
    pub max_lights: u32,
    /// 曝光值（后处理）
    pub exposure: f32,
    /// Gamma 值（后处理，通常为 2.2）
    pub gamma: f32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            sample_count: 4,
            hdr: true,
            shadow_map_size: 2048,
            max_lights: 256,
            exposure: 1.0,
            gamma: 2.2,
        }
    }
}

/// GPU 渲染上下文（持有 wgpu Device/Queue/Surface）
#[derive(Debug)]
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    /// 渲染器配置
    pub renderer_config: RendererConfig,
    /// 深度缓冲
    pub depth_texture: GpuTexture,
    /// HDR 渲染目标（若启用 HDR）
    pub hdr_texture: Option<GpuTexture>,
}

impl RenderContext {
    /// 创建 wgpu 渲染上下文
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
        renderer_config: RendererConfig,
    ) -> anyhow::Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No suitable GPU adapter found"))?;

        tracing::info!(
            "GPU Adapter: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Ummerse GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_texture = GpuTexture::create_depth_texture(
            &device,
            width.max(1),
            height.max(1),
            renderer_config.sample_count,
            "depth_texture",
        );

        let hdr_texture = if renderer_config.hdr {
            Some(GpuTexture::create_hdr_target(
                &device,
                width.max(1),
                height.max(1),
                renderer_config.sample_count,
                "hdr_target",
            ))
        } else {
            None
        };

        Ok(Self {
            device,
            queue,
            config,
            surface,
            renderer_config,
            depth_texture,
            hdr_texture,
        })
    }

    /// 调整窗口大小（重新创建深度/HDR 纹理）
    pub fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);

        // 重建深度缓冲
        self.depth_texture = GpuTexture::create_depth_texture(
            &self.device,
            w,
            h,
            self.renderer_config.sample_count,
            "depth_texture",
        );

        // 重建 HDR 目标
        if self.renderer_config.hdr {
            self.hdr_texture = Some(GpuTexture::create_hdr_target(
                &self.device,
                w,
                h,
                self.renderer_config.sample_count,
                "hdr_target",
            ));
        }
        tracing::debug!("RenderContext resized to {}x{}", w, h);
    }

    /// 当前表面宽度
    #[inline]
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// 当前表面高度
    #[inline]
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// 当前宽高比
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height.max(1) as f32
    }

    /// 表面格式
    #[inline]
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// 获取下一帧的 Surface Texture 用于渲染
    pub fn current_frame(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    /// 创建命令编码器
    pub fn create_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    /// 提交命令缓冲
    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

/// 精灵批渲染器（2D 高效批处理）
#[derive(Debug)]
pub struct SpriteBatch {
    vertices: Vec<Vertex2d>,
    indices: Vec<u32>,
    current_texture: Option<u64>, // texture ID
    draw_calls: u32,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(4 * 1024),
            indices: Vec::with_capacity(6 * 1024),
            current_texture: None,
            draw_calls: 0,
        }
    }

    /// 添加精灵到批次（position_px = 屏幕像素坐标，size_px = 像素大小）
    pub fn push_sprite(
        &mut self,
        position: glam::Vec2,
        size: glam::Vec2,
        uv_min: glam::Vec2,
        uv_max: glam::Vec2,
        color: [f32; 4],
        rotation: f32,
    ) {
        let base = self.vertices.len() as u32;
        let half = size * 0.5;

        // 4 个角的局部坐标
        let corners = [
            glam::Vec2::new(-half.x, -half.y),
            glam::Vec2::new(half.x, -half.y),
            glam::Vec2::new(half.x, half.y),
            glam::Vec2::new(-half.x, half.y),
        ];

        let uvs = [
            glam::Vec2::new(uv_min.x, uv_max.y),
            glam::Vec2::new(uv_max.x, uv_max.y),
            glam::Vec2::new(uv_max.x, uv_min.y),
            glam::Vec2::new(uv_min.x, uv_min.y),
        ];

        let cos_r = rotation.cos();
        let sin_r = rotation.sin();

        for (i, &corner) in corners.iter().enumerate() {
            // 旋转
            let rx = corner.x * cos_r - corner.y * sin_r;
            let ry = corner.x * sin_r + corner.y * cos_r;
            let world_pos = glam::Vec2::new(position.x + rx, position.y + ry);
            self.vertices.push(Vertex2d::new(world_pos, uvs[i], color));
        }

        // 2 个三角形
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    /// 清空批次（每帧开始时调用）
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.current_texture = None;
        self.draw_calls = 0;
    }

    /// 当前顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 精灵数量（每 4 个顶点 = 1 个精灵）
    pub fn sprite_count(&self) -> usize {
        self.vertices.len() / 4
    }
}

impl Default for SpriteBatch {
    fn default() -> Self {
        Self::new()
    }
}
