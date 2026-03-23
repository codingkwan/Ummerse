//! 事件总线 - 类型安全的发布/订阅系统
//!
//! 支持任意实现了 `Send + Sync + 'static` 的类型作为事件。
//! 使用 `TypeId` 作为事件分发键，`parking_lot::RwLock` 提升并发性能。
//!
//! ## 设计要点
//! - **立即分发**：`emit` 时同步触发所有已注册的处理器
//! - **帧缓存**：同时将事件写入队列，供 `drain` 在帧内遍历
//! - **线程安全**：所有公开 API 均可跨线程调用
//! - **取消订阅**：返回 `EventId`，通过 `unsubscribe` 精确移除

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── 事件 ID ───────────────────────────────────────────────────────────────────

/// 事件处理器唯一 ID（用于取消订阅）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// 生成新的事件处理器 ID
    #[inline]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 事件 trait ────────────────────────────────────────────────────────────────

/// 引擎事件 trait（blanket impl，所有 `Send + Sync + 'static` 自动实现）
pub trait Event: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Event for T {}

// ── 类型擦除事件队列 ──────────────────────────────────────────────────────────

/// 类型擦除的事件队列接口
trait AnyEventQueue: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// 清空帧缓存队列（帧末调用）
    fn clear_events(&mut self);
    /// 当前队列中的事件数
    fn event_count(&self) -> usize;
}

/// 类型化事件队列
struct TypedEventQueue<E: Event> {
    /// 帧内事件缓存（供 drain 消费）
    events: Vec<E>,
    /// 已注册的处理器列表
    handlers: Vec<(EventId, Box<dyn Fn(&E) + Send + Sync + 'static>)>,
}

impl<E: Event> TypedEventQueue<E> {
    #[inline]
    fn new() -> Self {
        Self {
            events: Vec::new(),
            handlers: Vec::new(),
        }
    }
}

impl<E: Event + 'static> AnyEventQueue for TypedEventQueue<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clear_events(&mut self) {
        self.events.clear();
    }
    fn event_count(&self) -> usize {
        self.events.len()
    }
}

// ── 事件总线 ──────────────────────────────────────────────────────────────────

/// 事件总线 - 线程安全的全局事件分发系统
///
/// # 并发模型
/// - 读操作（`drain`）使用读锁，允许多读并发
/// - 写操作（`emit`、`subscribe`）使用写锁
///
/// # 示例
/// ```rust
/// use ummerse_core::event::{EventBus, WindowResized};
///
/// let bus = EventBus::new();
/// let id = bus.subscribe::<WindowResized>(|e| {
///     println!("Resized to {}x{}", e.width, e.height);
/// });
/// bus.emit(WindowResized { width: 1920, height: 1080 });
/// bus.unsubscribe::<WindowResized>(id);
/// ```
#[derive(Default)]
pub struct EventBus {
    /// TypeId → 类型化队列（写操作用 Mutex 保证原子性）
    queues: Mutex<AHashMap<TypeId, Box<dyn AnyEventQueue>>>,
}

impl EventBus {
    /// 创建空事件总线
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// 发布事件：同步触发所有处理器，并缓存到帧队列
    pub fn emit<E: Event + 'static>(&self, event: E) {
        let mut queues = self.queues.lock();
        let type_id = TypeId::of::<E>();

        let queue = queues
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedEventQueue::<E>::new()));

        let typed = queue
            .as_any_mut()
            .downcast_mut::<TypedEventQueue<E>>()
            .expect("EventBus: TypeId mismatch – this is a bug");

        // 立即同步分发给所有处理器
        for (_, handler) in &typed.handlers {
            handler(&event);
        }

        // 将事件推入帧缓存
        typed.events.push(event);
    }

    /// 订阅事件，返回处理器 ID（用于取消订阅）
    pub fn subscribe<E: Event + 'static>(
        &self,
        handler: impl Fn(&E) + Send + Sync + 'static,
    ) -> EventId {
        let id = EventId::new();
        let mut queues = self.queues.lock();
        let type_id = TypeId::of::<E>();

        let queue = queues
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedEventQueue::<E>::new()));

        let typed = queue
            .as_any_mut()
            .downcast_mut::<TypedEventQueue<E>>()
            .expect("EventBus: TypeId mismatch");

        typed.handlers.push((id, Box::new(handler)));
        id
    }

    /// 取消订阅（通过处理器 ID）
    pub fn unsubscribe<E: Event + 'static>(&self, handler_id: EventId) {
        let mut queues = self.queues.lock();
        if let Some(queue) = queues.get_mut(&TypeId::of::<E>()) {
            if let Some(typed) = queue.as_any_mut().downcast_mut::<TypedEventQueue<E>>() {
                typed.handlers.retain(|(id, _)| *id != handler_id);
            }
        }
    }

    /// 遍历帧缓存中的所有事件（只读）
    ///
    /// 通常在帧逻辑中使用，收集该帧内的所有事件。
    pub fn drain<E: Event + 'static>(&self, mut consumer: impl FnMut(&E)) {
        let queues = self.queues.lock();
        if let Some(queue) = queues.get(&TypeId::of::<E>()) {
            if let Some(typed) = queue.as_any().downcast_ref::<TypedEventQueue<E>>() {
                for event in &typed.events {
                    consumer(event);
                }
            }
        }
    }

    /// 清空指定类型的事件队列（帧末调用）
    pub fn clear<E: Event + 'static>(&self) {
        let mut queues = self.queues.lock();
        if let Some(queue) = queues.get_mut(&TypeId::of::<E>()) {
            queue.clear_events();
        }
    }

    /// 清空所有事件队列（帧末统一调用）
    pub fn clear_all(&self) {
        let mut queues = self.queues.lock();
        for queue in queues.values_mut() {
            queue.clear_events();
        }
    }

    /// 获取指定类型当前帧的事件数量
    pub fn event_count<E: Event + 'static>(&self) -> usize {
        let queues = self.queues.lock();
        queues
            .get(&TypeId::of::<E>())
            .map(|q| q.event_count())
            .unwrap_or(0)
    }

    /// 当前注册的处理器数量（指定类型）
    pub fn handler_count<E: Event + 'static>(&self) -> usize {
        let queues = self.queues.lock();
        queues
            .get(&TypeId::of::<E>())
            .and_then(|q| q.as_any().downcast_ref::<TypedEventQueue<E>>())
            .map(|typed| typed.handlers.len())
            .unwrap_or(0)
    }
}

/// 全局事件总线的线程安全共享引用
pub type SharedEventBus = Arc<EventBus>;

// ── 内置引擎事件 ──────────────────────────────────────────────────────────────

/// 窗口大小变化事件
#[derive(Debug, Clone)]
pub struct WindowResized {
    pub width: u32,
    pub height: u32,
}

/// 窗口关闭请求事件
#[derive(Debug, Clone)]
pub struct WindowCloseRequested;

/// 窗口获得焦点
#[derive(Debug, Clone)]
pub struct WindowFocused;

/// 窗口失去焦点
#[derive(Debug, Clone)]
pub struct WindowUnfocused;

/// 场景加载完成事件
#[derive(Debug, Clone)]
pub struct SceneLoaded {
    pub scene_name: String,
}

/// 场景卸载事件
#[derive(Debug, Clone)]
pub struct SceneUnloaded {
    pub scene_name: String,
}

/// 资产加载完成事件
#[derive(Debug, Clone)]
pub struct AssetLoaded {
    pub path: String,
    pub asset_type: String,
}

/// 资产加载失败事件
#[derive(Debug, Clone)]
pub struct AssetLoadFailed {
    pub path: String,
    pub reason: String,
}

/// 引擎暂停事件
#[derive(Debug, Clone)]
pub struct EnginePaused;

/// 引擎恢复事件
#[derive(Debug, Clone)]
pub struct EngineResumed;

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    #[test]
    fn test_emit_and_subscribe() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();

        bus.subscribe::<WindowResized>(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(WindowResized { width: 1280, height: 720 });
        bus.emit(WindowResized { width: 1920, height: 1080 });

        assert_eq!(count.load(Ordering::Relaxed), 2);
        assert_eq!(bus.event_count::<WindowResized>(), 2);
    }

    #[test]
    fn test_drain_events() {
        let bus = EventBus::new();
        bus.emit(AssetLoaded {
            path: "player.png".into(),
            asset_type: "Image".into(),
        });
        bus.emit(AssetLoaded {
            path: "enemy.png".into(),
            asset_type: "Image".into(),
        });

        let mut paths = Vec::new();
        bus.drain::<AssetLoaded>(|e| paths.push(e.path.clone()));
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], "player.png");
    }

    #[test]
    fn test_unsubscribe() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicU32::new(0));
        let c = count.clone();

        let id = bus.subscribe::<WindowCloseRequested>(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(WindowCloseRequested);
        assert_eq!(count.load(Ordering::Relaxed), 1);

        bus.unsubscribe::<WindowCloseRequested>(id);
        bus.emit(WindowCloseRequested);
        // 取消订阅后不再触发
        assert_eq!(count.load(Ordering::Relaxed), 1);
        assert_eq!(bus.handler_count::<WindowCloseRequested>(), 0);
    }

    #[test]
    fn test_clear_all() {
        let bus = EventBus::new();
        bus.emit(WindowResized { width: 800, height: 600 });
        assert_eq!(bus.event_count::<WindowResized>(), 1);

        bus.clear_all();
        assert_eq!(bus.event_count::<WindowResized>(), 0);
    }

    #[test]
    fn test_multiple_event_types() {
        let bus = EventBus::new();
        let resize_count = Arc::new(AtomicU32::new(0));
        let close_count = Arc::new(AtomicU32::new(0));
        let rc = resize_count.clone();
        let cc = close_count.clone();

        bus.subscribe::<WindowResized>(move |_| {
            rc.fetch_add(1, Ordering::Relaxed);
        });
        bus.subscribe::<WindowCloseRequested>(move |_| {
            cc.fetch_add(1, Ordering::Relaxed);
        });

        bus.emit(WindowResized { width: 1280, height: 720 });
        bus.emit(WindowCloseRequested);
        bus.emit(WindowResized { width: 800, height: 600 });

        assert_eq!(resize_count.load(Ordering::Relaxed), 2);
        assert_eq!(close_count.load(Ordering::Relaxed), 1);
    }
}
