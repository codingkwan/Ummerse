//! 引擎全局配置和生命周期
//!
//! 提供引擎配置结构体和生命周期状态机。
//! 配置支持从 TOML 文件加载，所有字段均可序列化供 AI 读取。

use serde::{Deserialize, Serialize};

// ── 引擎配置 ──────────────────────────────────────────────────────────────────

/// 引擎全局配置（顶层聚合）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineConfig {
    pub window: WindowConfig,
    pub engine: EngineSettings,
    pub render: RenderConfig,
    pub audio: AudioConfig,
    pub debug: DebugConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub resizable: bool,
    pub vsync: bool,
    pub min_width: u32,
    pub min_height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Ummerse Game".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
            resizable: true,
            vsync: true,
            min_width: 320,
            min_height: 240,
        }
    }
}

/// 引擎运行时设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSettings {
    /// 目标帧率（0 = 不限制）
    pub target_fps: u32,
    /// 物理帧率（Hz）
    pub physics_fps: u32,
    /// 最大帧时间（防止"死亡螺旋"，秒）
    pub max_delta_time: f32,
    /// 是否启用多线程系统执行
    pub multi_threaded: bool,
    /// 工作线程数（0 = 自动检测 CPU 核心数）
    pub worker_threads: usize,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            target_fps: 60,
            physics_fps: 60,
            max_delta_time: 0.1,
            multi_threaded: true,
            worker_threads: 0,
        }
    }
}

/// 渲染配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub backend: RenderBackend,
    /// MSAA 采样数（1/2/4/8）
    pub msaa_samples: u32,
    /// 是否启用 HDR
    pub hdr_enabled: bool,
    /// 阴影贴图分辨率
    pub shadow_map_size: u32,
    /// 曝光值（后处理）
    pub exposure: f32,
    /// Gamma 值
    pub gamma: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            backend: RenderBackend::Auto,
            msaa_samples: 4,
            hdr_enabled: true,
            shadow_map_size: 2048,
            exposure: 1.0,
            gamma: 2.2,
        }
    }
}

/// 渲染后端选择
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderBackend {
    /// 自动选择最优后端
    Auto,
    /// Vulkan
    Vulkan,
    /// Metal（macOS/iOS）
    Metal,
    /// DirectX 12（Windows）
    Dx12,
    /// WebGL2（Web）
    WebGl2,
    /// WebGPU
    WebGpu,
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub sample_rate: u32,
    pub buffer_size: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 1.0,
            sample_rate: 44100,
            buffer_size: 1024,
        }
    }
}

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// 是否显示 FPS 计数器
    pub show_fps: bool,
    /// 是否显示碰撞体边框
    pub show_collision: bool,
    /// 是否显示线框渲染
    pub show_wireframe: bool,
    /// 日志级别
    pub log_level: LogLevel,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            show_fps: false,
            show_collision: false,
            show_wireframe: false,
            log_level: LogLevel::Info,
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

// ── 引擎生命周期 ──────────────────────────────────────────────────────────────

/// 引擎状态机
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// 尚未初始化
    Uninitialized,
    /// 正在运行
    Running,
    /// 已暂停（逻辑暂停，渲染继续）
    Paused,
    /// 即将停止（下帧退出）
    Stopping,
    /// 已停止
    Stopped,
}

impl std::fmt::Display for EngineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "Uninitialized"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

/// 引擎核心 - 管理生命周期和全局状态
#[derive(Debug)]
pub struct Engine {
    pub config: EngineConfig,
    state: EngineState,
}

impl Engine {
    /// 使用默认配置创建引擎
    #[inline]
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            state: EngineState::Uninitialized,
        }
    }

    /// 使用自定义配置创建引擎
    #[inline]
    pub fn with_config(config: EngineConfig) -> Self {
        Self {
            config,
            state: EngineState::Uninitialized,
        }
    }

    /// 从 TOML 文件加载配置
    pub fn from_config_file(path: &str) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str::<EngineConfig>(&content)
            .map_err(|e| crate::error::EngineError::ConfigError(e.to_string()))?;
        Ok(Self::with_config(config))
    }

    /// 将当前配置保存到 TOML 文件
    pub fn save_config(&self, path: &str) -> crate::error::Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| crate::error::EngineError::ConfigError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 获取引擎当前状态
    #[inline]
    #[must_use]
    pub fn state(&self) -> EngineState {
        self.state
    }

    /// 初始化引擎（状态 Uninitialized → Running）
    pub fn initialize(&mut self) {
        if self.state == EngineState::Uninitialized {
            self.state = EngineState::Running;
            tracing::info!(
                engine = crate::ENGINE_NAME,
                version = crate::ENGINE_VERSION,
                "Engine initialized"
            );
        }
    }

    /// 暂停引擎（状态 Running → Paused）
    pub fn pause(&mut self) {
        if self.state == EngineState::Running {
            self.state = EngineState::Paused;
            tracing::debug!("Engine paused");
        }
    }

    /// 恢复引擎（状态 Paused → Running）
    pub fn resume(&mut self) {
        if self.state == EngineState::Paused {
            self.state = EngineState::Running;
            tracing::debug!("Engine resumed");
        }
    }

    /// 请求停止（下帧退出）
    pub fn quit(&mut self) {
        if !matches!(self.state, EngineState::Stopping | EngineState::Stopped) {
            self.state = EngineState::Stopping;
            tracing::info!("Engine stopping");
        }
    }

    /// 标记为已停止
    pub fn mark_stopped(&mut self) {
        self.state = EngineState::Stopped;
    }

    /// 是否正在运行（Running 状态）
    #[inline]
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.state == EngineState::Running
    }

    /// 是否为活跃状态（Running 或 Paused）
    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self.state, EngineState::Running | EngineState::Paused)
    }

    /// 物理步长（秒），由物理帧率计算
    #[inline]
    #[must_use]
    pub fn physics_delta(&self) -> f32 {
        1.0 / self.config.engine.physics_fps.max(1) as f32
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut engine = Engine::new();
        assert_eq!(engine.state(), EngineState::Uninitialized);

        engine.initialize();
        assert_eq!(engine.state(), EngineState::Running);
        assert!(engine.is_running());

        engine.pause();
        assert_eq!(engine.state(), EngineState::Paused);
        assert!(!engine.is_running());
        assert!(engine.is_active());

        engine.resume();
        assert_eq!(engine.state(), EngineState::Running);

        engine.quit();
        assert_eq!(engine.state(), EngineState::Stopping);
    }

    #[test]
    fn test_physics_delta() {
        let engine = Engine::new();
        let delta = engine.physics_delta();
        assert!((delta - 1.0 / 60.0).abs() < f32::EPSILON);
    }
}
