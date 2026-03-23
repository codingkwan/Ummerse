//! 事件总线系统

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use parking_lot::RwLock;

/// 事件 ID（类型名称字符串）
pub type EventId = &'static str;

/// 事件 trait - 所有事件类型需实现
pub trait Event: Any + Send + Sync + 'static {
    /// 事件名称（用于调试）
    fn event_name() -> &'static str where Self: Sized;
}

/// 事件处理函数类型
pub type EventHandler<E> = Arc<dyn Fn(&E) + Send + Sync>;

/// 类型擦除的事件处理函数
type BoxedHandler = Arc<dyn Fn(&dyn Any) + Send + Sync>;

/// 事件总线 - 发布/订阅系统
#[derive(Default)]
pub struct EventBus {
    handlers: RwLock<HashMap<TypeId, Vec<BoxedHandler>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// 订阅事件
    pub fn subscribe<E: Event>(&self, handler: impl Fn(&E) + Send + Sync + 'static) {
        let type_id = TypeId::of::<E>();
        let boxed: BoxedHandler = Arc::new(move |any_event| {
            if let Some(event) = any_event.downcast_ref::<E>() {
                handler(event);
            }
        });
        self.handlers.write().entry(type_id).or_default().push(boxed);
    }

    /// 发布事件（同步）
    pub fn emit<E: Event>(&self, event: &E) {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read();
        if let Some(handlers) = handlers.get(&type_id) {
            for handler in handlers {
                handler(event);
            }
        }
    }

    /// 清除某类型的所有订阅
    pub fn clear<E: Event>(&self) {
        let type_id = TypeId::of::<E>();
        self.handlers.write().remove(&type_id);
    }

    /// 清除所有订阅
    pub fn clear_all(&self) {
        self.handlers.write().clear();
    }
}

// ── 内置事件类型 ──────────────────────────────────────────────────────────────

/// 窗口事件
#[derive(Debug, Clone)]
pub struct WindowResizeEvent {
    pub width: u32,
    pub height: u32,
}
impl Event for WindowResizeEvent {
    fn event_name() -> &'static str { "WindowResizeEvent" }
}

/// 窗口关闭事件
#[derive(Debug, Clone)]
pub struct WindowCloseEvent;
impl Event for WindowCloseEvent {
    fn event_name() -> &'static str { "WindowCloseEvent" }
}

/// 键盘按键事件
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key_code: u32,
    pub pressed: bool,
    pub modifiers: KeyModifiers,
}
impl Event for KeyEvent {
    fn event_name() -> &'static str { "KeyEvent" }
}

/// 键盘修饰键状态
#[derive(Debug, Clone, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

/// 鼠标移动事件
#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
}
impl Event for MouseMoveEvent {
    fn event_name() -> &'static str { "MouseMoveEvent" }
}

/// 鼠标按键事件
#[derive(Debug, Clone)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub pressed: bool,
    pub x: f32,
    pub y: f32,
}
impl Event for MouseButtonEvent {
    fn event_name() -> &'static str { "MouseButtonEvent" }
}

/// 鼠标按键枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// 场景加载事件
#[derive(Debug, Clone)]
pub struct SceneLoadedEvent {
    pub scene_path: String,
}
impl Event for SceneLoadedEvent {
    fn event_name() -> &'static str { "SceneLoadedEvent" }
}
