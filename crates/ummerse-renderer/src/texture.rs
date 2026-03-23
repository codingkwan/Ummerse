//! GPU 纹理管理

use bytemuck::{Pod, Zeroable};
use wgpu;

/// GPU 纹理（包含 wgpu Texture + View + Sampler）
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,
    pub label: String,
}

impl GpuTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// 从 RGBA8 原始字节创建 2D 纹理
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> anyhow::Result<Self> {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{}_sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            size,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            label: label.to_string(),
        })
    }

    /// 使用 image crate 加载图片文件
    pub fn from_image_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img_bytes: &[u8],
        label: &str,
    ) -> anyhow::Result<Self> {
        let img = image::load_from_memory(img_bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Self::from_bytes(device, queue, &rgba, width, height, label)
    }

    /// 创建深度纹理
    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{}_sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            size,
            format: Self::DEPTH_FORMAT,
            label: label.to_string(),
        }
    }

    /// 创建 HDR 渲染目标纹理（用于离屏渲染 + 后处理）
    pub fn create_hdr_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        sample_count: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let format = wgpu::TextureFormat::Rgba16Float;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{}_sampler", label)),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self { texture, view, sampler, size, format, label: label.to_string() }
    }

    /// 创建 1x1 纯色纹理（用作默认占位纹理）
    pub fn create_solid_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 4],
        label: &str,
    ) -> Self {
        Self::from_bytes(device, queue, &color, 1, 1, label)
            .expect("Failed to create solid color texture")
    }

    /// 宽度
    #[inline]
    pub fn width(&self) -> u32 {
        self.size.width
    }

    /// 高度
    #[inline]
    pub fn height(&self) -> u32 {
        self.size.height
    }
}

// ── 纹理集合（Texture Atlas）─────────────────────────────────────────────────

/// 纹理集合中的子纹理区域（UV 坐标）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct AtlasRegion {
    /// 左上角 UV
    pub uv_min: [f32; 2],
    /// 右下角 UV
    pub uv_max: [f32; 2],
}

impl AtlasRegion {
    pub fn new(uv_min: [f32; 2], uv_max: [f32; 2]) -> Self {
        Self { uv_min, uv_max }
    }

    /// 计算给定 UV 在 Atlas 中的实际 UV
    pub fn transform_uv(&self, uv: [f32; 2]) -> [f32; 2] {
        [
            self.uv_min[0] + uv[0] * (self.uv_max[0] - self.uv_min[0]),
            self.uv_min[1] + uv[1] * (self.uv_max[1] - self.uv_min[1]),
        ]
    }
}

/// 纹理采样模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplerMode {
    /// 线性过滤（平滑）
    Linear,
    /// 最近邻过滤（像素风）
    Nearest,
}

/// 创建标准采样器
pub fn create_sampler(
    device: &wgpu::Device,
    mode: SamplerMode,
    address_mode: wgpu::AddressMode,
    label: &str,
) -> wgpu::Sampler {
    let filter = match mode {
        SamplerMode::Linear => wgpu::FilterMode::Linear,
        SamplerMode::Nearest => wgpu::FilterMode::Nearest,
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
