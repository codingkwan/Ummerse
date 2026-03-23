//! # Ummerse Asset
//!
//! 资产管理系统：
//! - 同步/懒加载资产句柄
//! - 资产热重载（非 Wasm 平台）
//! - 纹理、网格、音频、场景等资产类型
//! - 强类型资产路径解析

pub mod asset_server;
pub mod handle;
pub mod loader;
pub mod types;
pub mod watcher;

pub use asset_server::AssetServer;
pub use handle::{AssetHandle, AssetId, AssetState, WeakAssetHandle};
pub use loader::{AssetLoader, EmbeddedLoader, FileSystemLoader, ImageLoader, LoadContext};
pub use types::{AudioAsset, ImageAsset, MeshAsset, ShaderAsset, TextAsset};

// AssetPath 从 lib.rs 直接导出，供其他模块使用
pub use types::AssetPath;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 资产类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Image,
    Mesh,
    Audio,
    Shader,
    Scene,
    Script,
    Font,
    Text,
    Binary,
    Unknown,
}

impl AssetType {
    /// 从文件扩展名推断资产类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "bmp" | "webp" | "hdr" | "exr" | "ktx2" => Self::Image,
            "gltf" | "glb" | "obj" | "fbx" | "dae" => Self::Mesh,
            "wav" | "mp3" | "ogg" | "flac" => Self::Audio,
            "wgsl" | "glsl" | "hlsl" | "spv" => Self::Shader,
            "uscn" | "scn" | "tscn" => Self::Scene,
            "wasm" => Self::Script,
            "ttf" | "otf" | "woff" | "woff2" => Self::Font,
            "txt" | "md" | "toml" | "json" | "ron" => Self::Text,
            _ => Self::Unknown,
        }
    }
}

// ── 错误类型 ──────────────────────────────────────────────────────────────────

/// 资产系统错误
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Asset not found: {path}")]
    NotFound { path: String },

    #[error("IO error for '{path}': {message}")]
    IoError { path: String, message: String },

    #[error("Parse error for '{path}': {message}")]
    ParseError { path: String, message: String },

    #[error("Unsupported asset format: .{extension}")]
    UnsupportedFormat { extension: String },

    #[error("Asset load failed: {0}")]
    LoadFailed(String),
}

/// 资产系统 Result 类型别名
pub type Result<T> = std::result::Result<T, AssetError>;
