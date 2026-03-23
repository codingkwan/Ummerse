//! 资源系统 - 引擎全局/局部资源管理

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// 资源 trait - 所有可注册的全局资源需实现此 trait
pub trait Resource: Any + Send + Sync + 'static {}

/// 自动为满足约束的类型实现 Resource
impl<T: Any + Send + Sync + 'static> Resource for T {}

/// 类型擦除的资源容器
type AnyResource = Arc<RwLock<Box<dyn Any + Send + Sync>>>;

/// 资源注册表
#[derive(Default)]
pub struct ResourceRegistry {
    resources: HashMap<TypeId, AnyResource>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册资源（若已存在则覆盖）
    pub fn insert<R: Resource>(&mut self, resource: R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(
            type_id,
            Arc::new(RwLock::new(Box::new(resource))),
        );
    }

    /// 获取资源的不可变访问
    pub fn get<R: Resource>(&self) -> Option<Res<R>> {
        let type_id = TypeId::of::<R>();
        self.resources.get(&type_id).map(|arc| Res {
            guard: arc.clone(),
            _phantom: std::marker::PhantomData,
        })
    }

    /// 获取资源的可变访问
    pub fn get_mut<R: Resource>(&self) -> Option<ResMut<R>> {
        let type_id = TypeId::of::<R>();
        self.resources.get(&type_id).map(|arc| ResMut {
            guard: arc.clone(),
            _phantom: std::marker::PhantomData,
        })
    }

    /// 移除资源
    pub fn remove<R: Resource>(&mut self) -> bool {
        self.resources.remove(&TypeId::of::<R>()).is_some()
    }

    /// 资源是否存在
    pub fn contains<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }
}

// ── 智能指针封装 ──────────────────────────────────────────────────────────────

/// 资源不可变引用
pub struct Res<R: Resource> {
    guard: AnyResource,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Resource> Res<R> {
    pub fn read(&self) -> ResReadGuard<'_, R> {
        let guard = self.guard.read();
        ResReadGuard { guard, _phantom: std::marker::PhantomData }
    }
}

pub struct ResReadGuard<'a, R: Resource> {
    guard: RwLockReadGuard<'a, Box<dyn Any + Send + Sync>>,
    _phantom: std::marker::PhantomData<R>,
}

impl<'a, R: Resource> Deref for ResReadGuard<'a, R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        self.guard.downcast_ref::<R>().expect("Resource type mismatch")
    }
}

/// 资源可变引用
pub struct ResMut<R: Resource> {
    guard: AnyResource,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Resource> ResMut<R> {
    pub fn write(&self) -> ResMutGuard<'_, R> {
        let guard = self.guard.write();
        ResMutGuard { guard, _phantom: std::marker::PhantomData }
    }
}

pub struct ResMutGuard<'a, R: Resource> {
    guard: RwLockWriteGuard<'a, Box<dyn Any + Send + Sync>>,
    _phantom: std::marker::PhantomData<R>,
}

impl<'a, R: Resource> Deref for ResMutGuard<'a, R> {
    type Target = R;
    fn deref(&self) -> &Self::Target {
        self.guard.downcast_ref::<R>().expect("Resource type mismatch")
    }
}

impl<'a, R: Resource> DerefMut for ResMutGuard<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.downcast_mut::<R>().expect("Resource type mismatch")
    }
}
