//! 引擎全局配置和生命周期

use serde::{Deserialize, Serialize};

/// 引擎全局配置
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
    /// 物理帧率
    pub physics_fps: u32,
    /// 最大帧时间（防止"死亡螺旋"）
    pub max_delta_time: f32,
    /// 是否使用多线程渲染
    pub multi_threaded: bool,
    /// 工作线程数（0 = 自动）
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
    pub msaa_samples: u32,
    pub hdr_enabled: bool,
    pub shadow_map_size: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            backend: RenderBackend::Auto,
            msaa_samples: 4,
            hdr_enabled: true,
            shadow_map_size: 2048,
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
    pub show_fps: bool,
    pub show_collision: bool,
    pub show_wireframe: bool,
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

// ── 引擎实例 ──────────────────────────────────────────────────────────────────

/// 引擎状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Uninitialized,
    Running,
    Paused,
    Stopping,
    Stopped,
}

/// 引擎核心 - 管理生命周期和全局状态
#[derive(Debug)]
pub struct Engine {
    pub config: EngineConfig,
    state: EngineState,
}

impl Engine {
    /// 使用默认配置创建引擎
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            state: EngineState::Uninitialized,
        }
    }

    /// 使用自定义配置创建引擎
    pub fn with_config(config: EngineConfig) -> Self {
        Self {
            config,
            state: EngineState::Uninitialized,
        }
    }

    /// 从 TOML 文件加载配置
    pub fn from_config_file(path: &str) -> crate::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: EngineConfig = toml::from_str::<EngineConfig>(&content)
            .map_err(|e: toml::de::Error| crate::error::EngineError::ConfigError(e.to_string()))?;
        Ok(Self::with_config(config))
    }

    /// 引擎当前状态
    #[must_use]
    pub fn state(&self) -> EngineState {
        self.state
    }

    /// 初始化引擎
    pub fn initialize(&mut self) {
        self.state = EngineState::Running;
        tracing::info!(
            "{} v{} initialized",
            crate::ENGINE_NAME,
            crate::ENGINE_VERSION
        );
    }

    /// 暂停引擎
    pub fn pause(&mut self) {
        if self.state == EngineState::Running {
            self.state = EngineState::Paused;
        }
    }

    /// 恢复引擎
    pub fn resume(&mut self) {
        if self.state == EngineState::Paused {
            self.state = EngineState::Running;
        }
    }

    /// 请求停止
    pub fn quit(&mut self) {
        self.state = EngineState::Stopping;
    }

    /// 是否正在运行
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.state == EngineState::Running
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
