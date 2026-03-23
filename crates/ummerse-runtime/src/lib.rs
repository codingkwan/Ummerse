//! # Ummerse Runtime
//!
//! 游戏运行时核心，负责：
//! - 主循环管理（固定物理步 + 可变渲染步）
//! - 平台抽象层（桌面/Web）
//! - 各子系统协调（场景树、物理、脚本、音频）
//! - 启动配置解析

pub mod game_loop;
pub mod platform;
pub mod systems;

pub use game_loop::{GameLoop, LoopConfig};
pub use platform::Platform;

use ummerse_core::{engine::EngineConfig, error::Result};
use tracing::{info, warn};

/// 游戏运行时 - 整合所有子系统
pub struct GameRuntime {
    pub config: EngineConfig,
    loop_config: LoopConfig,
}

impl GameRuntime {
    /// 从引擎配置创建运行时
    pub fn new(config: EngineConfig) -> Self {
        let loop_config = LoopConfig {
            target_fps: config.engine.target_fps,
            physics_fps: config.engine.physics_fps,
            max_delta: config.engine.max_delta_time,
        };
        Self { config, loop_config }
    }

    /// 从默认配置创建运行时
    pub fn default_config() -> Self {
        Self::new(EngineConfig::default())
    }

    /// 从 TOML 配置文件创建运行时
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: EngineConfig = toml::from_str(&content)
            .map_err(|e| ummerse_core::error::EngineError::ConfigError(e.to_string()))?;
        Ok(Self::new(config))
    }

    /// 启动运行时
    pub fn run(self) {
        info!(
            "Starting {} v{}",
            ummerse_core::ENGINE_NAME,
            ummerse_core::ENGINE_VERSION
        );
        info!("Window: {}x{} '{}'",
            self.config.window.width,
            self.config.window.height,
            self.config.window.title
        );
        // 主循环由平台层启动（此处为占位）
        // 实际运行时应通过 bevy App 驱动
        warn!("GameRuntime::run() is a stub - use GameRuntime::build_bevy_app() to get a Bevy App");
    }

    /// 获取主循环配置
    pub fn loop_config(&self) -> &LoopConfig {
        &self.loop_config
    }
}

impl Default for GameRuntime {
    fn default() -> Self {
        Self::default_config()
    }
}
