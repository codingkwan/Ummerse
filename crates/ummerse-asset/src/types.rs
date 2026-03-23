//! 内置资产类型定义

use serde::{Deserialize, Serialize};

/// 资产路径（强类型包装）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetPath(String);

impl AssetPath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 获取文件扩展名（小写）
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .extension()
            .and_then(|e| e.to_str())
    }

    /// 获取文件名（不含路径）
    pub fn file_name(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .file_name()
            .and_then(|n| n.to_str())
    }

    /// 获取不含扩展名的文件名
    pub fn stem(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .file_stem()
            .and_then(|s| s.to_str())
    }
}

impl From<&str> for AssetPath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for AssetPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for AssetPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 图像资产 ──────────────────────────────────────────────────────────────────

/// 图像像素格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8,
    Rgb8,
    Rgba16F,
    R8,
    Rg8,
}

/// 图像资产
#[derive(Debug, Clone)]
pub struct ImageAsset {
    /// 原始 RGBA8 像素数据
    pub data: Vec<u8>,
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
    /// 像素格式
    pub format: PixelFormat,
    /// 是否已预乘 Alpha
    pub premultiplied_alpha: bool,
}

impl ImageAsset {
    /// 创建纯色图像
    pub fn solid_color(width: u32, height: u32, color: [u8; 4]) -> Self {
        let pixel_count = (width * height) as usize;
        let mut data = Vec::with_capacity(pixel_count * 4);
        for _ in 0..pixel_count {
            data.extend_from_slice(&color);
        }
        Self {
            data,
            width,
            height,
            format: PixelFormat::Rgba8,
            premultiplied_alpha: false,
        }
    }

    /// 1x1 白色像素（默认占位纹理）
    pub fn white_pixel() -> Self {
        Self::solid_color(1, 1, [255, 255, 255, 255])
    }

    /// 1x1 黑色像素
    pub fn black_pixel() -> Self {
        Self::solid_color(1, 1, [0, 0, 0, 255])
    }

    /// 像素数量
    pub fn pixel_count(&self) -> u32 {
        self.width * self.height
    }

    /// 每像素字节数
    pub fn bytes_per_pixel(&self) -> u32 {
        match self.format {
            PixelFormat::Rgba8 => 4,
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba16F => 8,
            PixelFormat::R8 => 1,
            PixelFormat::Rg8 => 2,
        }
    }
}

// ── 网格资产 ──────────────────────────────────────────────────────────────────

/// 网格资产（CPU 侧）
#[derive(Debug, Clone)]
pub struct MeshAsset {
    /// 顶点位置 (x,y,z)
    pub positions: Vec<[f32; 3]>,
    /// 顶点法线
    pub normals: Vec<[f32; 3]>,
    /// UV 坐标
    pub uvs: Vec<[f32; 2]>,
    /// 切线
    pub tangents: Vec<[f32; 4]>,
    /// 顶点颜色（可选）
    pub colors: Vec<[f32; 4]>,
    /// 索引列表（三角形）
    pub indices: Vec<u32>,
    /// 子网格信息（起始索引 + 数量）
    pub sub_meshes: Vec<SubMesh>,
}

/// 子网格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubMesh {
    pub index_start: u32,
    pub index_count: u32,
    pub material_index: u32,
}

impl MeshAsset {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            tangents: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            sub_meshes: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for MeshAsset {
    fn default() -> Self {
        Self::new()
    }
}

// ── 音频资产 ──────────────────────────────────────────────────────────────────

/// 音频资产（已解码 PCM 数据）
#[derive(Debug, Clone)]
pub struct AudioAsset {
    /// 采样率（Hz）
    pub sample_rate: u32,
    /// 声道数（1 = 单声道，2 = 立体声）
    pub channels: u32,
    /// PCM 采样数据（f32，交错格式）
    pub samples: Vec<f32>,
    /// 时长（秒）
    pub duration_secs: f32,
}

impl AudioAsset {
    pub fn new(sample_rate: u32, channels: u32, samples: Vec<f32>) -> Self {
        let duration_secs = if sample_rate > 0 && channels > 0 {
            samples.len() as f32 / (sample_rate * channels) as f32
        } else {
            0.0
        };
        Self {
            sample_rate,
            channels,
            samples,
            duration_secs,
        }
    }

    /// 静默音频（用于占位）
    pub fn silence(duration_secs: f32, sample_rate: u32, channels: u32) -> Self {
        let sample_count = (duration_secs * sample_rate as f32 * channels as f32) as usize;
        Self::new(sample_rate, channels, vec![0.0f32; sample_count])
    }
}

// ── 着色器资产 ────────────────────────────────────────────────────────────────

/// 着色器语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
    Hlsl,
    SpirV,
}

/// 着色器资产
#[derive(Debug, Clone)]
pub struct ShaderAsset {
    /// 着色器源码（文本格式）或字节码（SpirV）
    pub source: ShaderSource,
    /// 着色器语言
    pub language: ShaderLanguage,
    /// 入口点（顶点）
    pub vertex_entry: String,
    /// 入口点（片元）
    pub fragment_entry: String,
}

/// 着色器源码
#[derive(Debug, Clone)]
pub enum ShaderSource {
    /// 文本源码（WGSL/GLSL/HLSL）
    Text(String),
    /// 二进制字节码（SpirV）
    Binary(Vec<u8>),
}

impl ShaderAsset {
    pub fn from_wgsl(source: impl Into<String>) -> Self {
        Self {
            source: ShaderSource::Text(source.into()),
            language: ShaderLanguage::Wgsl,
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
        }
    }

    pub fn source_text(&self) -> Option<&str> {
        match &self.source {
            ShaderSource::Text(s) => Some(s.as_str()),
            ShaderSource::Binary(_) => None,
        }
    }
}

// ── 文本资产 ──────────────────────────────────────────────────────────────────

/// 文本文件资产（JSON/TOML/Markdown 等）
#[derive(Debug, Clone)]
pub struct TextAsset {
    pub content: String,
    pub encoding: String,
}

impl TextAsset {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            encoding: "utf-8".to_string(),
        }
    }
}
