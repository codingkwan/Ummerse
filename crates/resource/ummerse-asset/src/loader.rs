//! 资产加载器接口与内置实现

use std::collections::HashMap;

use crate::{AssetError, AssetPath, Result};

// ── 加载上下文 ────────────────────────────────────────────────────────────────

/// 加载上下文 - 携带资产路径和原始字节数据
#[derive(Debug)]
pub struct LoadContext {
    /// 资产路径
    pub path: AssetPath,
    /// 原始字节
    pub bytes: Vec<u8>,
}

impl LoadContext {
    pub fn new(path: AssetPath, bytes: Vec<u8>) -> Self {
        Self { path, bytes }
    }

    /// 将字节解析为 UTF-8 文本
    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.bytes).map_err(|e| AssetError::ParseError {
            path: self.path.to_string(),
            message: format!("UTF-8 decode error: {}", e),
        })
    }

    /// 将字节解析为 JSON
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let text = self.as_str()?;
        serde_json::from_str(text).map_err(|e| AssetError::ParseError {
            path: self.path.to_string(),
            message: e.to_string(),
        })
    }

    /// 将字节解析为 TOML
    pub fn parse_toml<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        let text = self.as_str()?;
        toml::from_str(text).map_err(|e| AssetError::ParseError {
            path: self.path.to_string(),
            message: e.to_string(),
        })
    }
}

// ── 加载器 trait ──────────────────────────────────────────────────────────────

/// 资产加载器 trait - 同步版本（兼容 wasm）
pub trait AssetLoader: Send + Sync + 'static {
    /// 该加载器支持的文件扩展名列表
    fn extensions(&self) -> &[&str];

    /// 加载器名称
    fn name(&self) -> &str;

    /// 处理并验证加载上下文，返回经处理的上下文
    ///
    /// 默认实现直接返回原始上下文（透传）。
    /// 具体加载器可以在此做格式验证、转换等。
    fn load(&self, ctx: LoadContext) -> Result<LoadContext> {
        Ok(ctx)
    }

    /// 是否支持指定的文件扩展名
    fn supports(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();
        self.extensions()
            .iter()
            .any(|&e| e == "*" || e == ext_lower)
    }

    /// 克隆 boxed（用于注册到注册表）
    fn clone_box(&self) -> Box<dyn AssetLoader>;
}

impl Clone for Box<dyn AssetLoader> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// ── 透传加载器 ────────────────────────────────────────────────────────────────

/// 透传加载器 - 不做任何处理，直接返回原始字节
#[derive(Clone, Debug)]
pub struct PassthroughLoader {
    pub supported_extensions: Vec<&'static str>,
    pub loader_name: &'static str,
}

impl PassthroughLoader {
    pub fn new(name: &'static str, extensions: Vec<&'static str>) -> Self {
        Self {
            supported_extensions: extensions,
            loader_name: name,
        }
    }

    /// 接受所有扩展名的通用加载器
    pub fn all(name: &'static str) -> Self {
        Self {
            supported_extensions: vec!["*"],
            loader_name: name,
        }
    }
}

impl AssetLoader for PassthroughLoader {
    fn extensions(&self) -> &[&str] {
        &self.supported_extensions
    }

    fn name(&self) -> &str {
        self.loader_name
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

// ── 文件系统加载器 ────────────────────────────────────────────────────────────

/// 文件系统加载器（仅桌面平台，同步版）
#[derive(Clone, Debug)]
pub struct FileSystemLoader {
    /// 资产根目录
    pub root: std::path::PathBuf,
}

impl FileSystemLoader {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl AssetLoader for FileSystemLoader {
    fn extensions(&self) -> &[&str] {
        &["*"]
    }

    fn name(&self) -> &str {
        "FileSystemLoader"
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

// ── 内嵌加载器 ────────────────────────────────────────────────────────────────

/// 内嵌资产加载器 - 从编译时嵌入的静态字节数组加载
#[derive(Default, Debug)]
pub struct EmbeddedLoader {
    assets: HashMap<String, &'static [u8]>,
}

impl EmbeddedLoader {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    /// 注册一个内嵌资产
    pub fn register(&mut self, path: &str, data: &'static [u8]) {
        self.assets.insert(path.to_string(), data);
    }

    /// 直接读取内嵌字节（同步）
    pub fn get_bytes(&self, path: &AssetPath) -> Option<&'static [u8]> {
        self.assets.get(path.as_str()).copied()
    }
}

impl Clone for EmbeddedLoader {
    fn clone(&self) -> Self {
        // EmbeddedLoader 内含 &'static 引用，可安全克隆
        Self {
            assets: self.assets.clone(),
        }
    }
}

impl AssetLoader for EmbeddedLoader {
    fn extensions(&self) -> &[&str] {
        &["*"]
    }

    fn name(&self) -> &str {
        "EmbeddedLoader"
    }

    fn load(&self, ctx: LoadContext) -> Result<LoadContext> {
        if let Some(bytes) = self.get_bytes(&ctx.path) {
            Ok(LoadContext {
                path: ctx.path,
                bytes: bytes.to_vec(),
            })
        } else {
            Err(AssetError::NotFound {
                path: ctx.path.to_string(),
            })
        }
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

// ── 图片加载器 ────────────────────────────────────────────────────────────────

/// 图片格式验证加载器
#[derive(Clone, Debug)]
pub struct ImageLoader;

impl AssetLoader for ImageLoader {
    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "webp", "hdr"]
    }

    fn name(&self) -> &str {
        "ImageLoader"
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}
