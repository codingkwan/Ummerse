//! 资源系统 - 全局共享数据（类似 Bevy Resource 概念）
//!
//! 资源是全局单例数据，通过类型 ID 索引，
//! 生命周期与引擎绑定。
//!
//! ## 设计要点
//! - 使用 `AHashMap` 替代 `HashMap`（更快的哈希算法，无密码安全需求）
//! - `Res<T>` / `ResMut<T>` 包装模式防止裸指针暴露
//! - `ResourceRegistry` 支持按类型和任意类型擦除访问

use std::any::{Any, TypeId};

use ahash::AHashMap;

// ── Resource trait ────────────────────────────────────────────────────────────

/// 引擎资源 trait
///
/// 任意 `Send + Sync + 'static` 类型均可作为资源，无需手动实现。
pub trait Resource: Send + Sync + 'static {}

/// 为所有满足约束的类型自动实现
impl<T: Send + Sync + 'static> Resource for T {}

// ── 资源引用包装 ──────────────────────────────────────────────────────────────

/// 不可变资源引用（借用检查包装）
///
/// 通过 `Deref` 透明访问内部资源。
pub struct Res<'a, T: Resource> {
    value: &'a T,
}

impl<'a, T: Resource> Res<'a, T> {
    #[inline]
    pub(crate) fn new(value: &'a T) -> Self {
        Self { value }
    }
}

impl<'a, T: Resource> std::ops::Deref for Res<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Resource + std::fmt::Debug> std::fmt::Debug for Res<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Res").field("value", self.value).finish()
    }
}

/// 可变资源引用（借用检查包装）
///
/// 通过 `DerefMut` 透明访问内部资源。
pub struct ResMut<'a, T: Resource> {
    value: &'a mut T,
}

impl<'a, T: Resource> ResMut<'a, T> {
    #[inline]
    pub(crate) fn new(value: &'a mut T) -> Self {
        Self { value }
    }
}

impl<'a, T: Resource> std::ops::Deref for ResMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Resource> std::ops::DerefMut for ResMut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T: Resource + std::fmt::Debug> std::fmt::Debug for ResMut<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResMut").field("value", self.value).finish()
    }
}

// ── 资源注册表 ────────────────────────────────────────────────────────────────

/// 全局资源注册表 - 按类型索引的类型擦除存储
///
/// 使用 `AHashMap` 提升查询性能（比 `std::HashMap` 快约 30%）。
#[derive(Default)]
pub struct ResourceRegistry {
    /// TypeId → Box<dyn Any + Send + Sync>
    resources: AHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceRegistry {
    /// 创建空注册表
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建指定初始容量的注册表
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: AHashMap::with_capacity(capacity),
        }
    }

    /// 插入资源（若已存在则覆盖，返回旧值）
    pub fn insert<T: Resource>(&mut self, resource: T) -> Option<T> {
        self.resources
            .insert(TypeId::of::<T>(), Box::new(resource))
            .and_then(|old| old.downcast::<T>().ok())
            .map(|b| *b)
    }

    /// 获取资源不可变引用
    #[inline]
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }

    /// 获取资源可变引用
    #[inline]
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
    #[inline]
    pub fn contains<T: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// 当前资源数量
    #[inline]
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// 获取资源包装引用（`Res<T>`）
    ///
    /// # Panics
    /// 若资源不存在则 panic，提示类型名称。
    ///
    /// 建议在确认资源已插入的场景使用；不确定时使用 [`get`] 返回 `Option`。
    #[inline]
    pub fn res<T: Resource>(&self) -> Res<'_, T> {
        Res::new(self.get::<T>().unwrap_or_else(|| {
            panic!(
                "Resource '{}' not found in registry",
                std::any::type_name::<T>()
            )
        }))
    }

    /// 获取资源可变包装引用（`ResMut<T>`）
    ///
    /// # Panics
    /// 若资源不存在则 panic。
    #[inline]
    pub fn res_mut<T: Resource>(&mut self) -> ResMut<'_, T> {
        ResMut::new(self.get_mut::<T>().unwrap_or_else(|| {
            panic!(
                "Resource '{}' not found in registry",
                std::any::type_name::<T>()
            )
        }))
    }

    /// 若资源不存在则插入默认值，返回不可变引用
    pub fn get_or_insert_default<T: Resource + Default>(&mut self) -> &T {
        self.resources
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_ref::<T>()
            .expect("ResourceRegistry: type ID mismatch - this is a bug")
    }

    /// 若资源不存在则通过闭包插入，返回不可变引用
    pub fn get_or_insert_with<T: Resource>(&mut self, f: impl FnOnce() -> T) -> &T {
        self.resources
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(f()))
            .downcast_ref::<T>()
            .expect("ResourceRegistry: type ID mismatch - this is a bug")
    }

    /// 清空所有资源
    pub fn clear(&mut self) {
        self.resources.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MyCounter {
        value: u32,
    }

    #[derive(Debug)]
    struct GameSettings {
        volume: f32,
    }

    #[derive(Debug, Default)]
    struct Score(u32);

    #[test]
    fn test_insert_and_get() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 42 });
        assert!(reg.contains::<MyCounter>());
        assert_eq!(reg.get::<MyCounter>().unwrap().value, 42);
    }

    #[test]
    fn test_insert_returns_old_value() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 1 });
        let old = reg.insert(MyCounter { value: 2 });
        assert_eq!(old, Some(MyCounter { value: 1 }));
        assert_eq!(reg.get::<MyCounter>().unwrap().value, 2);
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
    fn test_res_deref() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 7 });
        let res = reg.res::<MyCounter>();
        assert_eq!(res.value, 7); // 通过 Deref 访问
    }

    #[test]
    fn test_res_mut_deref_mut() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 0 });
        {
            let mut r = reg.res_mut::<MyCounter>();
            r.value = 99;
        }
        assert_eq!(reg.get::<MyCounter>().unwrap().value, 99);
    }

    #[test]
    fn test_get_or_insert_default() {
        let mut reg = ResourceRegistry::new();
        let score = reg.get_or_insert_default::<Score>();
        assert_eq!(score.0, 0);
    }

    #[test]
    fn test_get_or_insert_with() {
        let mut reg = ResourceRegistry::new();
        let counter = reg.get_or_insert_with(|| MyCounter { value: 42 });
        assert_eq!(counter.value, 42);
        // 第二次调用不应执行 f
        let counter2 = reg.get_or_insert_with(|| MyCounter { value: 999 });
        assert_eq!(counter2.value, 42);
    }

    #[test]
    #[should_panic(expected = "Resource 'ummerse_core::resource::tests::MyCounter' not found")]
    fn test_res_panics_on_missing() {
        let reg = ResourceRegistry::new();
        let _r = reg.res::<MyCounter>();
    }

    #[test]
    fn test_clear() {
        let mut reg = ResourceRegistry::new();
        reg.insert(MyCounter { value: 1 });
        reg.insert(GameSettings { volume: 1.0 });
        reg.clear();
        assert!(reg.is_empty());
    }
}
