//! 事件总线 - 类型安全的发布/订阅系统
//!
//! 支持任意实现了 `Send + Sync + 'static` 的类型作为事件。
//! 参考 Bevy Events 设计，使用 TypeId 作为事件分发键。

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── 事件 ID ───────────────────────────────────────────────────────────────────

/// 事件唯一 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// 生成新的事件 ID
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

/// 引擎事件 trait
///
/// 所有引擎事件须实现此 trait，以便通过事件总线分发。
pub trait Event: Send + Sync + 'static {}

/// 为所有满足约束的类型自动实现 Event
impl<T: Send + Sync + 'static> Event for T {}

// ── 事件处理器 ────────────────────────────────────────────────────────────────

/// 事件处理函数类型（boxing 闭包）
type HandlerFn<E> = Box<dyn Fn(&E) + Send + Sync + 'static>;

/// 事件处理器包装
pub struct EventHandler<E: Event> {
    pub id: EventId,
    handler: HandlerFn<E>,
}

impl<E: Event> EventHandler<E> {
    /// 创建新处理器
    pub fn new(handler: impl Fn(&E) + Send + Sync + 'static) -> Self {
        Self {
            id: EventId::new(),
            handler: Box::new(handler),
        }
    }

    /// 调用处理器
    pub fn call(&self, event: &E) {
        (self.handler)(event);
    }
}

// ── 事件总线 ──────────────────────────────────────────────────────────────────

/// 事件队列（类型擦除）
trait AnyEventQueue: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
}

/// 类型化事件队列
struct TypedEventQueue<E: Event> {
    events: Vec<E>,
    handlers: Vec<EventHandler<E>>,
}

impl<E: Event> TypedEventQueue<E> {
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

    fn clear(&mut self) {
        self.events.clear();
    }
}

/// 事件总线 - 线程安全的全局事件分发系统
#[derive(Default)]
pub struct EventBus {
    queues: Mutex<HashMap<TypeId, Box<dyn AnyEventQueue>>>,
}

impl EventBus {
    /// 创建空事件总线
    pub fn new() -> Self {
        Self::default()
    }

    /// 发布事件（广播到所有订阅者）
    pub fn emit<E: Event + 'static>(&self, event: E) {
        let mut queues = self.queues.lock().unwrap();
        let type_id = TypeId::of::<E>();

        let queue = queues
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedEventQueue::<E>::new()));

        let typed = queue
            .as_any_mut()
            .downcast_mut::<TypedEventQueue<E>>()
            .expect("EventBus: type mismatch");

        // 立即分发给所有处理器
        for handler in &typed.handlers {
            handler.call(&event);
        }

        // 同时缓存到队列（供 drain 消费）
        typed.events.push(event);
    }

    /// 订阅事件（注册处理器，返回处理器 ID 用于取消订阅）
    pub fn subscribe<E: Event + 'static>(
        &self,
        handler: impl Fn(&E) + Send + Sync + 'static,
    ) -> EventId {
        let mut queues = self.queues.lock().unwrap();
        let type_id = TypeId::of::<E>();

        let queue = queues
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedEventQueue::<E>::new()));

        let typed = queue
            .as_any_mut()
            .downcast_mut::<TypedEventQueue<E>>()
            .expect("EventBus: type mismatch");

        let h = EventHandler::new(handler);
        let id = h.id;
        typed.handlers.push(h);
        id
    }

    /// 取消订阅（通过处理器 ID）
    pub fn unsubscribe<E: Event + 'static>(&self, handler_id: EventId) {
        let mut queues = self.queues.lock().unwrap();
        let type_id = TypeId::of::<E>();

        if let Some(queue) = queues.get_mut(&type_id) {
            if let Some(typed) = queue.as_any_mut().downcast_mut::<TypedEventQueue<E>>() {
                typed.handlers.retain(|h| h.id != handler_id);
            }
        }
    }

    /// 消费所有缓存事件（只读遍历）
    pub fn drain<E: Event + 'static>(&self, mut consumer: impl FnMut(&E)) {
        let queues = self.queues.lock().unwrap();
        let type_id = TypeId::of::<E>();

        if let Some(queue) = queues.get(&type_id) {
            if let Some(typed) = queue.as_any().downcast_ref::<TypedEventQueue<E>>() {
                for event in &typed.events {
                    consumer(event);
                }
            }
        }
    }

    /// 清空事件队列（通常在帧末调用）
    pub fn clear<E: Event + 'static>(&self) {
        let mut queues = self.queues.lock().unwrap();
        let type_id = TypeId::of::<E>();
        if let Some(queue) = queues.get_mut(&type_id) {
            queue.clear();
        }
    }

    /// 清空所有事件队列
    pub fn clear_all(&self) {
        let mut queues = self.queues.lock().unwrap();
        for queue in queues.values_mut() {
            queue.clear();
        }
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

/// 场景加载完成事件
#[derive(Debug, Clone)]
pub struct SceneLoaded {
    pub scene_name: String,
}

/// 资产加载完成事件
#[derive(Debug, Clone)]
pub struct AssetLoaded {
    pub path: String,
    pub asset_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_emit_and_subscribe() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        bus.subscribe::<WindowResized>(move |_e| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit(WindowResized { width: 1280, height: 720 });
        bus.emit(WindowResized { width: 1920, height: 1080 });

        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_drain_events() {
        let bus = EventBus::new();
        bus.emit(AssetLoaded {
            path: "player.png".to_string(),
            asset_type: "Image".to_string(),
        });

        let mut collected = Vec::new();
        bus.drain::<AssetLoaded>(|e| collected.push(e.path.clone()));
        assert_eq!(collected, vec!["player.png"]);
    }

    #[test]
    fn test_unsubscribe() {
        let bus = EventBus::new();
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let id = bus.subscribe::<WindowCloseRequested>(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.emit(WindowCloseRequested);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        bus.unsubscribe::<WindowCloseRequested>(id);
        bus.emit(WindowCloseRequested);
        // 取消订阅后不再触发
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }
}
