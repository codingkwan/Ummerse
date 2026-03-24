//! 资产服务器 - 统一管理资产的加载、缓存和卸载

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use tracing::info;

use crate::{
    AssetError, Result,
    handle::{AssetHandle, AssetId},
    loader::{AssetLoader, LoadContext},
    types::AssetPath,
};

/// 资产服务器 - 同步加载和缓存资产
pub struct AssetServer {
    /// 资产根目录
    root: PathBuf,
    /// 路径 -> ID 缓存（避免重复分配 ID）
    path_cache: DashMap<AssetPath, AssetId>,
    /// 已注册的加载器（按扩展名）
    loaders: DashMap<String, Arc<dyn AssetLoader>>,
}

impl std::fmt::Debug for AssetServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetServer")
            .field("root", &self.root)
            .field("cached_count", &self.path_cache.len())
            .finish_non_exhaustive()
    }
}

impl AssetServer {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            path_cache: DashMap::new(),
            loaders: DashMap::new(),
        }
    }

    /// 注册资产加载器（按支持的扩展名索引）
    pub fn register_loader<L: AssetLoader>(&self, loader: L) {
        let loader: Arc<dyn AssetLoader> = Arc::new(loader);
        for &ext in loader.extensions() {
            self.loaders.insert(ext.to_string(), Arc::clone(&loader));
        }
    }

    /// 解析资产绝对路径
    pub fn resolve_path(&self, path: &AssetPath) -> PathBuf {
        if Path::new(path.as_str()).is_absolute() {
            PathBuf::from(path.as_str())
        } else {
            self.root.join(path.as_str())
        }
    }

    /// 同步加载原始字节
    pub fn load_bytes(&self, path: &AssetPath) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path);
        std::fs::read(&full_path).map_err(|e| AssetError::IoError {
            path: full_path.display().to_string(),
            message: e.to_string(),
        })
    }

    /// 获取或生成资产 ID（通过路径）
    pub fn get_or_create_id(&self, path: &AssetPath) -> AssetId {
        if let Some(id) = self.path_cache.get(path) {
            return *id;
        }
        let id = AssetId::new();
        self.path_cache.insert(path.clone(), id);
        id
    }

    /// 加载资产句柄（懒加载 - 仅生成句柄，不立即读取文件）
    ///
    /// 用于资产跟踪，实际数据由 `load_bytes` / `load_typed` 获取。
    pub fn load<T: Send + Sync + 'static>(&self, path: impl Into<AssetPath>) -> AssetHandle<T> {
        let path = path.into();
        let id = self.get_or_create_id(&path);
        info!("Asset queued: {}", path.as_str());
        AssetHandle::loading(id, path.as_str())
    }

    /// 同步加载并返回包含数据的句柄
    pub fn load_sync<T, F>(&self, path: impl Into<AssetPath>, decoder: F) -> Result<AssetHandle<T>>
    where
        T: Send + Sync + 'static,
        F: FnOnce(&LoadContext) -> Result<T>,
    {
        let path = path.into();
        let bytes = self.load_bytes(&path)?;
        let ctx = LoadContext::new(path.clone(), bytes);
        let data = decoder(&ctx)?;
        let id = self.get_or_create_id(&path);
        Ok(AssetHandle::loaded(id, path.as_str(), data))
    }

    /// 加载并解析 JSON
    pub fn load_json<T: serde::de::DeserializeOwned>(
        &self,
        path: impl Into<AssetPath>,
    ) -> Result<T> {
        let path = path.into();
        let bytes = self.load_bytes(&path)?;
        let ctx = LoadContext::new(path, bytes);
        ctx.parse_json()
    }

    /// 加载并解析 TOML
    pub fn load_toml<T: serde::de::DeserializeOwned>(
        &self,
        path: impl Into<AssetPath>,
    ) -> Result<T> {
        let path = path.into();
        let bytes = self.load_bytes(&path)?;
        let ctx = LoadContext::new(path, bytes);
        ctx.parse_toml()
    }

    /// 使用注册的加载器处理资产
    pub fn load_with_loader(&self, path: impl Into<AssetPath>) -> Result<LoadContext> {
        let path = path.into();
        let ext = Path::new(path.as_str())
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 先查找具体扩展名加载器，再查找通配符加载器
        let loader = self
            .loaders
            .get(&ext)
            .or_else(|| self.loaders.get("*"))
            .ok_or_else(|| AssetError::UnsupportedFormat {
                extension: ext.clone(),
            })?
            .clone();

        let bytes = self.load_bytes(&path)?;
        let ctx = LoadContext::new(path, bytes);
        loader.load(ctx)
    }

    /// 资产根目录
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 已缓存的资产数量
    pub fn cached_count(&self) -> usize {
        self.path_cache.len()
    }

    /// 清除路径 -> ID 缓存
    pub fn clear_cache(&self) {
        self.path_cache.clear();
    }

    /// 检查资产文件是否存在
    pub fn exists(&self, path: &AssetPath) -> bool {
        self.resolve_path(path).exists()
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new("assets")
    }
}
