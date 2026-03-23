//! # Ummerse Runtime
//!
//! 游戏运行时核心：
//! - Bevy App 构建（ECS + 渲染）
//! - 固定物理步 + 可变渲染步主循环
//! - 平台抽象（桌面/Web）
//! - 各子系统协调

pub mod game_loop;
pub mod platform;
pub mod systems;

pub use game_loop::{GameLoop, LoopConfig};
pub use platform::{BuildProfile, Platform, PlatformCapabilities};
pub use systems::{
    AudioPlayerComponent, Camera2dComponent, Camera3dComponent, Collider2dComponent,
    ColliderShape, MeshInstance3dComponent, NodeName, NodeVisible, RigidBody2dComponent,
    RigidBodyType, ScriptComponent, SpriteComponent, UmmerseCorePlugin, UmmerseTransform2d,
    UmmerseTransform3d,
};

use bevy::prelude::*;
use tracing::{info, warn};
use ummerse_core::engine::EngineConfig;

// ── Bevy Resource 包装 ────────────────────────────────────────────────────────

/// 引擎配置 Bevy Resource
#[derive(Resource, Clone, Debug)]
pub struct EngineConfigResource(pub EngineConfig);

/// 主循环配置 Bevy Resource
#[derive(Resource, Clone, Debug)]
pub struct LoopConfigResource(pub LoopConfig);

// ── Bevy 调度 SystemSet 标签 ──────────────────────────────────────────────────

/// Ummerse 自定义 SystemSet（用于系统排序和依赖声明）
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UmmerseSystemSet {
    /// 输入处理阶段
    Input,
    /// 脚本执行阶段
    Script,
    /// 物理更新阶段（FixedUpdate 中）
    Physics,
    /// 场景树同步阶段
    Scene,
    /// 渲染准备阶段
    RenderPrep,
}

// ── 游戏运行时 ────────────────────────────────────────────────────────────────

/// 游戏运行时 - 整合所有子系统，构建 Bevy App
pub struct GameRuntime {
    /// 引擎配置
    pub config: EngineConfig,
    /// 当前运行平台
    pub platform: Platform,
    /// 主循环配置
    loop_config: LoopConfig,
}

impl GameRuntime {
    /// 从引擎配置创建运行时
    pub fn new(config: EngineConfig) -> Self {
        let platform = Platform::current();
        let loop_config = LoopConfig {
            target_fps: config.engine.target_fps,
            physics_fps: config.engine.physics_fps,
            max_delta: config.engine.max_delta_time,
        };

        info!(
            platform = %platform.name(),
            build = if BuildProfile::current().is_debug() { "debug" } else { "release" },
            "Runtime initialized"
        );

        Self { config, platform, loop_config }
    }

    /// 使用默认配置创建运行时
    #[inline]
    pub fn default_config() -> Self {
        Self::new(EngineConfig::default())
    }

    /// 从 TOML 配置文件加载运行时
    pub fn from_file(path: &str) -> ummerse_core::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: EngineConfig = toml::from_str(&content)
            .map_err(|e| ummerse_core::error::EngineError::ConfigError(e.to_string()))?;
        Ok(Self::new(config))
    }

    /// 构建并返回完整的 Bevy App（含所有 Ummerse 插件）
    ///
    /// # 示例
    /// ```rust,no_run
    /// use ummerse_runtime::GameRuntime;
    /// let runtime = GameRuntime::default_config();
    /// let mut app = runtime.build_bevy_app();
    /// app.run();
    /// ```
    pub fn build_bevy_app(self) -> App {
        let mut app = App::new();

        // ── DefaultPlugins（含窗口、渲染、输入等）─────────────────────
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: self.config.window.title.clone(),
                        resolution: bevy::window::WindowResolution::new(
                            self.config.window.width as f32,
                            self.config.window.height as f32,
                        ),
                        resizable: self.config.window.resizable,
                        present_mode: if self.config.window.vsync {
                            bevy::window::PresentMode::AutoVsync
                        } else {
                            bevy::window::PresentMode::AutoNoVsync
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(bevy::log::LogPlugin {
                    level: map_log_level(&self.config.debug.log_level),
                    filter: "wgpu=warn,naga=warn,bevy_render=info".to_string(),
                    ..Default::default()
                }),
        );

        // ── Ummerse 核心 ECS 插件 ─────────────────────────────────────
        app.add_plugins(UmmerseCorePlugin);

        // ── 全局 Resource 注入 ────────────────────────────────────────
        app.insert_resource(EngineConfigResource(self.config.clone()));
        app.insert_resource(LoopConfigResource(self.loop_config.clone()));

        // ── 固定时间步（物理更新频率）────────────────────────────────
        let physics_delta = 1.0 / self.loop_config.physics_fps.max(1) as f64;
        app.insert_resource(Time::<Fixed>::from_seconds(physics_delta));

        if self.config.debug.show_fps {
            info!("FPS overlay enabled (TODO: UI overlay)");
        }

        info!(
            width = self.config.window.width,
            height = self.config.window.height,
            title = %self.config.window.title,
            physics_hz = self.loop_config.physics_fps,
            "Bevy app built"
        );

        app
    }

    /// 构建无窗口无头运行时（用于服务器/测试）
    pub fn build_headless_app(self) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(UmmerseCorePlugin);
        app.insert_resource(EngineConfigResource(self.config));
        app.insert_resource(LoopConfigResource(self.loop_config));
        app
    }

    /// 获取主循环配置
    #[inline]
    pub fn loop_config(&self) -> &LoopConfig {
        &self.loop_config
    }
}

impl Default for GameRuntime {
    fn default() -> Self {
        Self::default_config()
    }
}

// ── 日志级别映射 ──────────────────────────────────────────────────────────────

fn map_log_level(level: &ummerse_core::engine::LogLevel) -> bevy::log::Level {
    use ummerse_core::engine::LogLevel;
    match level {
        LogLevel::Error => bevy::log::Level::ERROR,
        LogLevel::Warn => bevy::log::Level::WARN,
        LogLevel::Info => bevy::log::Level::INFO,
        LogLevel::Debug => bevy::log::Level::DEBUG,
        LogLevel::Trace => bevy::log::Level::TRACE,
    }
}

// ── 游戏应用构建器 ────────────────────────────────────────────────────────────

/// 游戏应用构建器 - 提供简洁的游戏启动 API
///
/// # 示例
/// ```rust,no_run
/// use ummerse_runtime::GameAppBuilder;
///
/// GameAppBuilder::new()
///     .title("My Awesome Game")
///     .window_size(1280, 720)
///     .setup(|app| {
///         app.add_systems(Startup, setup_scene);
///     })
///     .run();
///
/// fn setup_scene(mut commands: Commands) {
///     // 添加相机、精灵等
/// }
/// ```
pub struct GameAppBuilder {
    config: EngineConfig,
    setup_fns: Vec<Box<dyn FnOnce(&mut App)>>,
}

impl GameAppBuilder {
    /// 创建默认构建器
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            setup_fns: Vec::new(),
        }
    }

    /// 设置窗口标题
    #[inline]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.window.title = title.into();
        self
    }

    /// 设置窗口尺寸
    #[inline]
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.window.width = width;
        self.config.window.height = height;
        self
    }

    /// 设置全屏模式
    #[inline]
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.config.window.fullscreen = fullscreen;
        self
    }

    /// 设置垂直同步
    #[inline]
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.config.window.vsync = vsync;
        self
    }

    /// 设置物理更新帧率
    #[inline]
    pub fn physics_fps(mut self, fps: u32) -> Self {
        self.config.engine.physics_fps = fps;
        self
    }

    /// 设置是否显示 FPS
    #[inline]
    pub fn show_fps(mut self, show: bool) -> Self {
        self.config.debug.show_fps = show;
        self
    }

    /// 添加用户自定义设置函数
    ///
    /// 可调用多次，按添加顺序执行。
    pub fn setup(mut self, f: impl FnOnce(&mut App) + 'static) -> Self {
        self.setup_fns.push(Box::new(f));
        self
    }

    /// 构建并运行游戏（阻塞直到窗口关闭）
    pub fn run(self) {
        let runtime = GameRuntime::new(self.config);
        let mut app = runtime.build_bevy_app();
        for setup_fn in self.setup_fns {
            setup_fn(&mut app);
        }
        app.run();
    }

    /// 仅构建 App，不运行（用于测试）
    pub fn build(self) -> App {
        let runtime = GameRuntime::new(self.config);
        let mut app = runtime.build_bevy_app();
        for setup_fn in self.setup_fns {
            setup_fn(&mut app);
        }
        app
    }
}

impl Default for GameAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = GameRuntime::default_config();
        assert_eq!(runtime.loop_config().target_fps, 60);
        assert_eq!(runtime.loop_config().physics_fps, 60);
        assert_eq!(runtime.loop_config().max_delta, 0.1);
    }

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert!(platform.is_desktop() || platform.is_web() || platform.is_mobile());
    }

    #[test]
    fn test_headless_app_builds() {
        let runtime = GameRuntime::default_config();
        let _app = runtime.build_headless_app();
        // 无窗口 App 可以正常构建
    }

    #[test]
    fn test_app_builder_config() {
        let builder = GameAppBuilder::new()
            .title("Test")
            .window_size(800, 600)
            .vsync(false)
            .physics_fps(120);

        assert_eq!(builder.config.window.title, "Test");
        assert_eq!(builder.config.window.width, 800);
        assert_eq!(builder.config.window.height, 600);
        assert!(!builder.config.window.vsync);
        assert_eq!(builder.config.engine.physics_fps, 120);
    }
}
