//! 内置资产类型定义

use serde::{Deserialize, Serialize};

// ── 资产路径 ──────────────────────────────────────────────────────────────────

/// 资产路径（相对于项目资产根目录）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetPath(String);

impl AssetPath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .extension()
            .and_then(|e| e.to_str())
    }

    /// 推断资产类型标签（返回字符串）
    pub fn extension_lower(&self) -> String {
        self.extension().unwrap_or("").to_lowercase()
    }
}

impl std::fmt::Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for AssetPath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for AssetPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&std::path::Path> for AssetPath {
    fn from(p: &std::path::Path) -> Self {
        Self(p.to_string_lossy().into_owned())
    }
}

// ── 图像资产 ──────────────────────────────────────────────────────────────────

/// 图像资产
#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub data: Vec<u8>,
    pub mip_levels: u32,
}

impl ImageAsset {
    pub fn new(width: u32, height: u32, format: ImageFormat, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            format,
            data,
            mip_levels: 1,
        }
    }

    /// 像素数量
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// 字节数
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }
}

/// 图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    Depth32Float,
    Bc7RgbaUnorm,
}

impl ImageFormat {
    /// 每像素字节数
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            Self::Rgba8Unorm | Self::Rgba8UnormSrgb => 4,
            Self::Rgba16Float => 8,
            Self::Rgba32Float => 16,
            Self::Depth32Float => 4,
            Self::Bc7RgbaUnorm => 1, // 压缩格式，近似值
        }
    }
}

// ── 网格资产 ──────────────────────────────────────────────────────────────────

/// 网格顶点
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl MeshVertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
            tangent: [1.0, 0.0, 0.0, 1.0],
        }
    }
}

/// 网格资产
#[derive(Debug, Clone)]
pub struct MeshAsset {
    pub name: String,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
    pub aabb: ummerse_math::aabb::Aabb3d,
}

impl MeshAsset {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            indices: Vec::new(),
            submeshes: Vec::new(),
            aabb: ummerse_math::aabb::Aabb3d::default(),
        }
    }

    /// 顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 三角面数量
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// 子网格（对应一个材质槽）
#[derive(Debug, Clone)]
pub struct SubMesh {
    pub name: String,
    pub index_offset: u32,
    pub index_count: u32,
    pub material_index: u32,
}

// ── 音频资产 ──────────────────────────────────────────────────────────────────

/// 音频资产（PCM 浮点采样）
#[derive(Debug, Clone)]
pub struct AudioAsset {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub duration_secs: f64,
}

impl AudioAsset {
    pub fn new(sample_rate: u32, channels: u16, samples: Vec<f32>) -> Self {
        let duration_secs = samples.len() as f64 / (sample_rate as f64 * channels as f64);
        Self {
            sample_rate,
            channels,
            samples,
            duration_secs,
        }
    }
}

// ── 着色器资产 ────────────────────────────────────────────────────────────────

/// 着色器资产
#[derive(Debug, Clone)]
pub struct ShaderAsset {
    pub name: String,
    pub stage: ShaderStage,
    pub source: ShaderSource,
}

impl ShaderAsset {
    pub fn wgsl(name: impl Into<String>, stage: ShaderStage, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stage,
            source: ShaderSource::Wgsl(source.into()),
        }
    }
}

/// 着色器阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

/// 着色器源码
#[derive(Debug, Clone)]
pub enum ShaderSource {
    Wgsl(String),
    SpirV(Vec<u32>),
}

// ── 文本资产 ──────────────────────────────────────────────────────────────────

/// 文本资产（UTF-8 字符串）
#[derive(Debug, Clone)]
pub struct TextAsset {
    pub content: String,
    pub encoding: TextEncoding,
}

impl TextAsset {
    pub fn utf8(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            encoding: TextEncoding::Utf8,
        }
    }
}

/// 文本编码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    Utf8,
    Utf16,
    Latin1,
}
