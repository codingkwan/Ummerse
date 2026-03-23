//! # Ummerse Renderer
//!
//! 2D/3D 渲染管线，基于 wgpu + Bevy 渲染器：
//! - 2D 精灵批渲染
//! - 3D PBR 渲染管线
//! - 后处理效果
//! - 自定义着色器支持（WGSL）

pub mod camera;
pub mod material;
pub mod mesh;
pub mod pipeline;
pub mod render_graph;
pub mod texture;

pub use camera::{Camera2d, Camera3d, CameraProjection};
pub use material::{Material, PbrMaterial, UnlitMaterial};
pub use mesh::{GpuMesh, MeshBuilder};
pub use pipeline::{RenderPipeline2d, RenderPipeline3d};
pub use texture::GpuTexture;

use wgpu;

/// 渲染器配置
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub sample_count: u32,
    pub hdr: bool,
    pub shadow_map_size: u32,
    pub max_lights: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            sample_count: 4,
            hdr: true,
            shadow_map_size: 2048,
            max_lights: 256,
        }
    }
}

/// GPU 渲染上下文
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
}

impl RenderContext {
    /// 创建 wgpu 渲染上下文
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No suitable GPU adapter found"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Ummerse GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
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
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self { device, queue, config, surface })
    }

    /// 调整窗口大小
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
