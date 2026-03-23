//! 资源系统 - 全局共享数据（类似 Bevy Resource 概念）
//!
//! 资源是全局单例数据，通过类型 ID 索引，
//! 生命周期与引擎绑定。

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

// ── Resource trait ────────────────────────────────────────────────────────────

/// 引擎资源 trait
///
/// 任意 `Send + Sync + 'static` 类型均可作为资源。
pub trait Resource: Send + Sync + 'static {}

/// 为所有满足约束的类型自动实现
impl<T: Send + Sync + 'static> Resource for T {}

// ── 资源引用包装 ──────────────────────────────────────────────────────────────

/// 不可变资源引用（借用检查包装）
pub struct Res<'a, T: Resource> {
    value: &'a T,
}

impl<'a, T: Resource> Res<'a, T> {
    pub(crate) fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<'a, T: Resource> std::ops::Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

/// 可变资源引用（借用检查包装）
pub struct ResMut<'a, T: Resource> {
    value: &'a mut T,
}

impl<'a, T: Resource> ResMut<'a, T> {
    pub(crate) fn new(value: &'a mut T) -> Self {
        Self { value }
    }
}

impl<'a, T: Resource> std::ops::Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Resource> std::ops::DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

// ── 资源注册表 ────────────────────────────────────────────────────────────────

/// 全局资源注册表 - 按类型索引的类型擦除存储
#[derive(Default)]
pub struct ResourceRegistry {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入资源（若已存在则覆盖）
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// 获取资源不可变引用
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }

    /// 获取资源可变引用
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut::<T>())
    }

    /// 移除资源，返回所有权
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|r| r.downcast::<T>().ok())
            .map(|b| *b)
    }

    /// 是否包含该类型资源
    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// 当前资源数量
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// 获取资源包装引用（Res<T>）
    ///
    /// # Panics
    /// 若资源不存在则 panic。
    pub fn res<T: Resource>(&self) -> Res<'_, T> {
        Res::new(
            self.get::<T>()
                .unwrap_or_else(|| panic!("Resource '{}' not found", std::any::type_name::<T>())),
        )
    }

    /// 获取资源可变包装引用（ResMut<T>）
    ///
    /// # Panics
    /// 若资源不存在则 panic。
    pub fn res_mut<T: Resource>(&mut self) -> ResMut<'_, T> {
        ResMut::new(
            self.get_mut::<T>()
                .unwrap_or_else(|| panic!("Resource '{}' not found", std::any::type_name::<T>())),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyCounter {
        value: u32,
    }

    struct GameSettings {
        volume: f32,
    }

    #[test]
    fn test_insert_and_get() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 42 });
        assert!(reg.contains::<MyCounter>());
        assert_eq!(reg.get::<MyCounter>().unwrap().value, 42);
    }

    #[test]
    fn test_get_mut() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 0 });
        reg.get_mut::<MyCounter>().unwrap().value = 100;
        assert_eq!(reg.get::<MyCounter>().unwrap().value, 100);
    }

    #[test]
    fn test_remove() {
        let mut reg = ResourceRegistry::new();
        reg.insert(GameSettings { volume: 0.8 });
        let removed = reg.remove::<GameSettings>().unwrap();
        assert!((removed.volume - 0.8).abs() < f32::EPSILON);
        assert!(!reg.contains::<GameSettings>());
    }

    #[test]
    fn test_multiple_types() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 5 });
        reg.insert(GameSettings { volume: 1.0 });
        assert_eq!(reg.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Resource")]
    fn test_res_panics_on_missing() {
        let reg = ResourceRegistry::new();
        let _r = reg.res::<MyCounter>();
    }
}
