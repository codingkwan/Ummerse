//! # Ummerse Core
//!
//! 引擎核心库，提供：
//! - ECS（通过 Bevy ECS）
//! - 节点系统（Godot 风格场景树节点）
//! - 事件总线（类型安全，pub/sub）
//! - 信号系统（Godot 风格节点间通信）
//! - 引擎配置与生命周期
//! - 错误类型
//! - 日志初始化
//! - 输入系统
//! - 资源注册表
//! - 时间管理 & 定时器

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

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use app::{App, AppBuilder};
pub use engine::{Engine, EngineConfig, EngineState};
pub use error::{EngineError, Result};
pub use event::{
    AssetLoadFailed, AssetLoaded, EnginePaused, EngineResumed, Event, EventBus, EventId,
    SceneLoaded, SceneUnloaded, SharedEventBus, WindowCloseRequested, WindowFocused,
    WindowResized, WindowUnfocused,
};
pub use input::{InputAction, InputManager, KeyCode, MouseButton};
pub use node::{Node, NodeId, NodeMeta, NodePath, NodeType};
pub use plugin::{Plugin, PluginRegistry};
pub use resource::{Res, ResMut, Resource, ResourceRegistry};
pub use signal::{ConnectionId, ScriptSignalBus, Signal, SignalBus};
pub use time::{Time, Timer, TimerMode};

// Re-export bevy_ecs 核心组件（轻量，不引入完整 Bevy）
pub use bevy_ecs::prelude::*;

/// Ummerse 引擎版本
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// 引擎名称
pub const ENGINE_NAME: &str = "Ummerse";

/// 初始化引擎日志系统（tracing）
///
/// 应在 `main` 函数最开始调用一次。
/// 从环境变量 `RUST_LOG` 读取过滤规则（默认 `info`）。
///
/// # 示例
/// ```rust,no_run
/// ummerse_core::init_logging();
/// ```
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,wgpu=warn,naga=warn,bevy_render=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(false)
        .with_file(false)
        .compact()
        .init();
}
