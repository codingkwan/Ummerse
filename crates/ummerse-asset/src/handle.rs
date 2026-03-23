//! 资产句柄 - 强类型资产引用

use std::sync::{Arc, Weak};
use std::marker::PhantomData;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// 资产唯一 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl AssetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

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

/// 资产加载状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetState {
    /// 未开始加载
    Unloaded,
    /// 正在加载
    Loading,
    /// 加载完成
    Loaded,
    /// 加载失败
    Failed,
}

/// 资产内部数据（引用计数）
struct AssetInner<T> {
    pub id: AssetId,
    pub data: Option<T>,
    pub state: AssetState,
    pub path: String,
}

/// 强类型资产句柄（强引用，持有资产存活）
pub struct AssetHandle<T> {
    inner: Arc<parking_lot::RwLock<AssetInner<T>>>,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> AssetHandle<T> {
    /// 创建已加载资产句柄
    pub fn loaded(id: AssetId, path: String, data: T) -> Self {
        Self {
            inner: Arc::new(parking_lot::RwLock::new(AssetInner {
                id,
                data: Some(data),
                state: AssetState::Loaded,
                path,
            })),
            _phantom: PhantomData,
        }
    }

    /// 创建加载中占位句柄
    pub fn loading(id: AssetId, path: String) -> Self {
        Self {
            inner: Arc::new(parking_lot::RwLock::new(AssetInner {
                id,
                data: None,
                state: AssetState::Loading,
                path,
            })),
            _phantom: PhantomData,
        }
    }

    /// 资产 ID
    pub fn id(&self) -> AssetId {
        self.inner.read().id
    }

    /// 资产路径
    pub fn path(&self) -> String {
        self.inner.read().path.clone()
    }

    /// 资产加载状态
    pub fn state(&self) -> AssetState {
        self.inner.read().state
    }

    /// 资产是否已加载
    pub fn is_loaded(&self) -> bool {
        self.state() == AssetState::Loaded
    }

    /// 读取资产数据（若已加载）
    pub fn read(&self) -> Option<parking_lot::MappedRwLockReadGuard<'_, T>> {
        let guard = self.inner.read();
        if guard.data.is_some() {
            Some(parking_lot::RwLockReadGuard::map(guard, |inner| {
                inner.data.as_ref().unwrap()
            }))
        } else {
            None
        }
    }

    /// 创建弱引用（不持有资产）
    pub fn downgrade(&self) -> WeakAssetHandle<T> {
        WeakAssetHandle {
            inner: Arc::downgrade(&self.inner),
            _phantom: PhantomData,
        }
    }

    /// 强引用计数
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }
}

impl<T: Send + Sync + 'static> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            _phantom: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> std::fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.read();
        f.debug_struct("AssetHandle")
            .field("id", &inner.id)
            .field("path", &inner.path)
            .field("state", &inner.state)
            .finish()
    }
}

/// 弱资产句柄（不持有资产，可能失效）
pub struct WeakAssetHandle<T> {
    inner: Weak<parking_lot::RwLock<AssetInner<T>>>,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> WeakAssetHandle<T> {
    /// 尝试升级为强引用
    pub fn upgrade(&self) -> Option<AssetHandle<T>> {
        self.inner.upgrade().map(|inner| AssetHandle {
            inner,
            _phantom: PhantomData,
        })
    }

    /// 弱引用是否仍有效
    pub fn is_alive(&self) -> bool {
        self.inner.strong_count() > 0
    }
}

impl<T: Send + Sync + 'static> Clone for WeakAssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Weak::clone(&self.inner),
            _phantom: PhantomData,
        }
    }
}
