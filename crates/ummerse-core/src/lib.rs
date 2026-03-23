//! # Ummerse Core
//!
//! 引擎核心库，提供：
//! - ECS（通过 Bevy ECS）
//! - 节点系统（Godot 风格场景树节点）
//! - 事件总线
//! - 引擎配置
//! - 错误类型
//! - 日志初始化
//! - 输入系统

pub mod app;
pub mod engine;
pub mod error;
pub mod event;
pub mod input;
pub mod node;
pub mod plugin;
pub mod resource;
pub mod signal;
pub mod time;

// Re-export 常用类型
pub use app::{App, AppBuilder};
pub use engine::{Engine, EngineConfig};
pub use error::{EngineError, Result};
pub use event::{Event, EventBus, EventHandler, EventId};
pub use input::{InputAction, InputManager, KeyCode, MouseButton};
pub use node::{Node, NodeId, NodePath, NodeType};
pub use plugin::Plugin;
pub use resource::{Res, ResMut, Resource};
pub use signal::{Signal, SignalBus};
pub use time::Time;

// Re-export bevy_ecs 核心组件（轻量，不引入完整 Bevy）
pub use bevy_ecs::prelude::*;

/// Ummerse 引擎版本
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// 引擎名称
pub const ENGINE_NAME: &str = "Ummerse";
