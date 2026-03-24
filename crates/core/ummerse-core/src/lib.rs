//! # Ummerse Core
//!
//! 引擎核心库，提供：
//! - **ECS**（通过 Bevy ECS）– 实体、组件、系统
//! - **节点系统**（Godot 风格场景树节点）
//! - **事件总线**（类型安全，pub/sub）
//! - **信号系统**（Godot 风格节点间通信）
//! - **引擎配置与生命周期**
//! - **错误类型**
//! - **输入系统**
//! - **资源注册表**
//! - **时间管理 & 定时器**
//! - **插件接口**
//!
//! ## 快速入门
//! ```rust,no_run
//! use ummerse_core::prelude::*;
//!
//! // 初始化日志
//! ummerse_core::init_logging();
//!
//! // 创建引擎
//! let mut engine = Engine::new();
//! engine.initialize();
//! ```

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

// ── 常用 prelude 模块 ─────────────────────────────────────────────────────────

/// 常用类型一次性导入
///
/// ```rust
/// use ummerse_core::prelude::*;
/// ```
pub mod prelude {
    pub use crate::app::{App, AppBuilder};
    pub use crate::engine::{Engine, EngineConfig, EngineState};
    pub use crate::error::{EngineError, Result};
    pub use crate::event::{EventBus, SharedEventBus};
    pub use crate::input::{InputAction, InputManager, KeyCode, MouseButton};
    pub use crate::node::{Node, NodeId, NodeMeta, NodePath, NodeType};
    pub use crate::plugin::{Plugin, PluginRegistry};
    pub use crate::resource::{Res, ResMut, Resource, ResourceRegistry};
    pub use crate::signal::{ConnectionId, Signal, SignalBus};
    pub use crate::time::{Time, Timer, TimerMode};

    // Re-export bevy_ecs 常用类型
    pub use bevy_ecs::prelude::*;
    // Re-export math 常用类型
    pub use ummerse_math::{Color, Transform2d, Transform3d, Vec2, Vec3};
}

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use app::{App, AppBuilder};
pub use engine::{Engine, EngineConfig, EngineSettings, EngineState};
pub use error::{EngineError, Result};
pub use event::{
    AssetLoadFailed, AssetLoaded, EnginePaused, EngineResumed, Event, EventBus, EventId,
    SceneLoaded, SceneUnloaded, SharedEventBus, WindowCloseRequested, WindowFocused, WindowResized,
    WindowUnfocused,
};
pub use input::{InputAction, InputManager, KeyCode, MouseButton};
pub use node::{Node, NodeId, NodeMeta, NodePath, NodeType};
pub use plugin::{Plugin, PluginRegistry};
pub use resource::{Res, ResMut, Resource, ResourceRegistry};
pub use signal::{ConnectionId, ScriptSignalBus, Signal, SignalBus};
pub use time::{Time, Timer, TimerMode};

// Re-export bevy_ecs 核心（轻量，不引入完整 Bevy）
pub use bevy_ecs::prelude::*;

// ── 全局常量 ──────────────────────────────────────────────────────────────────

/// Ummerse 引擎版本
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// 引擎名称
pub const ENGINE_NAME: &str = "Ummerse";

// ── 日志初始化 ────────────────────────────────────────────────────────────────

/// 初始化引擎日志系统（tracing）
///
/// 应在 `main` 函数最开始调用一次。
/// 从环境变量 `RUST_LOG` 读取过滤规则（默认 `info`）。
/// 若已初始化则静默跳过（使用 `try_init`）。
///
/// # 示例
/// ```rust,no_run
/// ummerse_core::init_logging();
/// ```
pub fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,wgpu=warn,naga=warn,bevy_render=info"));

    let _ = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(false)
        .with_file(false)
        .compact()
        .try_init();
}
