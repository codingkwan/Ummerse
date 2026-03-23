//! 资产句柄 - 强类型资产引用

use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 资产唯一 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl AssetId {
    /// 生成新的唯一 ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 获取内部 UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 加载状态 ──────────────────────────────────────────────────────────────────

/// 资产加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 加载失败
    Failed,
}

// ── 资产句柄 ──────────────────────────────────────────────────────────────────

struct AssetData<T> {
    pub state: parking_lot::RwLock<AssetState>,
    pub data: parking_lot::RwLock<Option<T>>,
}

/// 资产句柄 - 强引用（持有资产，防止被卸载）
pub struct AssetHandle<T> {
    pub id: AssetId,
    pub path: String,
    _marker: PhantomData<fn() -> T>,
    inner: Arc<AssetData<T>>,
}

impl<T: Send + Sync + 'static> AssetHandle<T> {
    /// 创建一个新的"加载中"句柄（内部用）
    pub fn new(id: AssetId) -> Self {
        Self {
            id,
            path: String::new(),
            _marker: PhantomData,
            inner: Arc::new(AssetData {
                state: parking_lot::RwLock::new(AssetState::Loading),
                data: parking_lot::RwLock::new(None),
            }),
        }
    }

    /// 创建已加载的资产句柄
    pub fn loaded(id: AssetId, path: impl Into<String>, data: T) -> Self {
        Self {
            id,
            path: path.into(),
            _marker: PhantomData,
            inner: Arc::new(AssetData {
                state: parking_lot::RwLock::new(AssetState::Loaded),
                data: parking_lot::RwLock::new(Some(data)),
            }),
        }
    }

    /// 创建加载中的资产句柄
    pub fn loading(id: AssetId, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            _marker: PhantomData,
            inner: Arc::new(AssetData {
                state: parking_lot::RwLock::new(AssetState::Loading),
                data: parking_lot::RwLock::new(None),
            }),
        }
    }

    /// 标记为加载完成并设置数据
    pub fn set_loaded(&self, data: T) {
        *self.inner.data.write() = Some(data);
        *self.inner.state.write() = AssetState::Loaded;
    }

    /// 标记为加载失败
    pub fn set_failed(&self) {
        *self.inner.state.write() = AssetState::Failed;
    }

    /// 获取加载状态
    pub fn state(&self) -> AssetState {
        *self.inner.state.read()
    }

    /// 是否已加载完成
    pub fn is_loaded(&self) -> bool {
        self.state() == AssetState::Loaded
    }

    /// 是否加载失败
    pub fn is_failed(&self) -> bool {
        self.state() == AssetState::Failed
    }

    /// 读取资产数据（若已加载）
    pub fn with_data<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.data.read().as_ref().map(f)
    }

    /// 创建弱引用
    pub fn downgrade(&self) -> WeakAssetHandle<T> {
        WeakAssetHandle {
            id: self.id,
            path: self.path.clone(),
            _marker: PhantomData,
            inner: Arc::downgrade(&self.inner),
        }
    }

    /// 强引用计数
    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            path: self.path.clone(),
            _marker: PhantomData,
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> std::fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle")
            .field("id", &self.id)
            .field("path", &self.path)
            .finish()
    }
}

// ── 弱引用句柄 ────────────────────────────────────────────────────────────────

/// 资产弱句柄 - 弱引用（不阻止资产被卸载）
pub struct WeakAssetHandle<T> {
    pub id: AssetId,
    pub path: String,
    _marker: PhantomData<fn() -> T>,
    inner: Weak<AssetData<T>>,
}

impl<T: Send + Sync + 'static> WeakAssetHandle<T> {
    /// 尝试升级为强引用
    pub fn upgrade(&self) -> Option<AssetHandle<T>> {
        self.inner.upgrade().map(|inner| AssetHandle {
            id: self.id,
            path: self.path.clone(),
            _marker: PhantomData,
            inner,
        })
    }

    /// 资产是否还存在（未被卸载）
    pub fn is_alive(&self) -> bool {
        self.inner.strong_count() > 0
    }
}
