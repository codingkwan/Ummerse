//! 资产句柄 - 强类型资产引用

use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 资产唯一 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl AssetId {
    /// 生成新的唯一资产 ID
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// 获取内部 UUID
    #[inline]
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

/// 资产内部数据（引用计数共享）
struct AssetInner<T> {
    pub id: AssetId,
    pub data: Option<T>,
    pub state: AssetState,
    pub path: String,
}

/// 强类型资产句柄（强引用，持有资产存活）
///
/// 克隆句柄只增加引用计数，不复制底层数据。
pub struct AssetHandle<T> {
    inner: Arc<RwLock<AssetInner<T>>>,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> AssetHandle<T> {
    /// 创建已加载资产句柄
    pub fn loaded(id: AssetId, path: impl Into<String>, data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AssetInner {
                id,
                data: Some(data),
                state: AssetState::Loaded,
                path: path.into(),
            })),
            _phantom: PhantomData,
        }
    }

    /// 创建加载中占位句柄（数据尚未就绪）
    pub fn loading(id: AssetId, path: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AssetInner {
                id,
                data: None,
                state: AssetState::Loading,
                path: path.into(),
            })),
            _phantom: PhantomData,
        }
    }

    /// 创建加载失败句柄
    pub fn failed(id: AssetId, path: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AssetInner {
                id,
                data: None,
                state: AssetState::Failed,
                path: path.into(),
            })),
            _phantom: PhantomData,
        }
    }

    /// 资产 ID
    #[inline]
    pub fn id(&self) -> AssetId {
        self.inner.read().id
    }

    /// 资产路径
    #[inline]
    pub fn path(&self) -> String {
        self.inner.read().path.clone()
    }

    /// 资产加载状态
    #[inline]
    pub fn state(&self) -> AssetState {
        self.inner.read().state
    }

    /// 资产是否已加载完成
    #[inline]
    pub fn is_loaded(&self) -> bool {
        self.state() == AssetState::Loaded
    }

    /// 是否加载失败
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.state() == AssetState::Failed
    }

    /// 读取资产数据（若已加载）
    ///
    /// 返回 `None` 表示数据尚未就绪或加载失败。
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

    /// 写入/更新资产数据（用于异步加载完成时填充）
    pub fn set_loaded(&self, data: T) {
        let mut inner = self.inner.write();
        inner.data = Some(data);
        inner.state = AssetState::Loaded;
    }

    /// 标记为加载失败
    pub fn set_failed(&self) {
        let mut inner = self.inner.write();
        inner.data = None;
        inner.state = AssetState::Failed;
    }

    /// 创建弱引用（不持有资产，可能失效）
    pub fn downgrade(&self) -> WeakAssetHandle<T> {
        WeakAssetHandle {
            inner: Arc::downgrade(&self.inner),
            _phantom: PhantomData,
        }
    }

    /// 当前强引用计数
    #[inline]
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

// ── 弱资产句柄 ────────────────────────────────────────────────────────────────

/// 弱资产句柄（不持有资产，可能失效）
///
/// 通过 [`AssetHandle::downgrade`] 创建，通过 [`WeakAssetHandle::upgrade`] 尝试还原。
pub struct WeakAssetHandle<T> {
    inner: Weak<RwLock<AssetInner<T>>>,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> WeakAssetHandle<T> {
    /// 尝试升级为强引用（返回 `None` 表示原资产已释放）
    pub fn upgrade(&self) -> Option<AssetHandle<T>> {
        self.inner.upgrade().map(|inner| AssetHandle {
            inner,
            _phantom: PhantomData,
        })
    }

    /// 弱引用是否仍有效（对应的强引用计数 > 0）
    #[inline]
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

impl<T: Send + Sync + 'static> std::fmt::Debug for WeakAssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakAssetHandle")
            .field("alive", &self.is_alive())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAsset {
        name: String,
    }

    #[test]
    fn test_loaded_handle() {
        let id = AssetId::new();
        let handle: AssetHandle<MockAsset> = AssetHandle::loaded(
            id,
            "assets/test.png",
            MockAsset { name: "test".to_string() },
        );
        assert!(handle.is_loaded());
        assert_eq!(handle.path(), "assets/test.png");
        assert!(handle.read().is_some());
    }

    #[test]
    fn test_loading_handle() {
        let id = AssetId::new();
        let handle: AssetHandle<MockAsset> = AssetHandle::loading(id, "assets/loading.png");
        assert!(!handle.is_loaded());
        assert_eq!(handle.state(), AssetState::Loading);
        assert!(handle.read().is_none());
    }

    #[test]
    fn test_set_loaded() {
        let id = AssetId::new();
        let handle: AssetHandle<MockAsset> = AssetHandle::loading(id, "assets/async.png");
        handle.set_loaded(MockAsset { name: "async".to_string() });
        assert!(handle.is_loaded());
        assert!(handle.read().is_some());
    }

    #[test]
    fn test_weak_handle() {
        let id = AssetId::new();
        let strong: AssetHandle<MockAsset> = AssetHandle::loaded(
            id,
            "assets/weak.png",
            MockAsset { name: "weak".to_string() },
        );
        let weak = strong.downgrade();
        assert!(weak.is_alive());

        {
            let upgraded = weak.upgrade();
            assert!(upgraded.is_some());
        }

        drop(strong);
        // 强引用已 drop，弱引用应失效
        assert!(!weak.is_alive());
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_clone_handle() {
        let id = AssetId::new();
        let h1: AssetHandle<MockAsset> =
            AssetHandle::loaded(id, "assets/clone.png", MockAsset { name: "clone".to_string() });
        let h2 = h1.clone();
        assert_eq!(h1.id(), h2.id());
        assert_eq!(h1.strong_count(), 2);
    }
}
