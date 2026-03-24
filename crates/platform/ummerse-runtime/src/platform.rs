//! 平台抽象层 - 桌面/Web 平台差异处理

/// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Windows 桌面
    Windows,
    /// macOS 桌面
    MacOs,
    /// Linux 桌面
    Linux,
    /// WebAssembly（浏览器）
    Web,
    /// Android
    Android,
    /// iOS
    Ios,
    /// 未知平台
    Unknown,
}

impl Platform {
    /// 检测当前运行平台
    pub fn current() -> Self {
        #[cfg(target_arch = "wasm32")]
        return Self::Web;

        #[cfg(target_os = "windows")]
        return Self::Windows;

        #[cfg(target_os = "macos")]
        return Self::MacOs;

        #[cfg(target_os = "linux")]
        return Self::Linux;

        #[cfg(target_os = "android")]
        return Self::Android;

        #[cfg(target_os = "ios")]
        return Self::Ios;

        #[allow(unreachable_code)]
        Self::Unknown
    }

    /// 是否为桌面平台
    pub fn is_desktop(&self) -> bool {
        matches!(self, Self::Windows | Self::MacOs | Self::Linux)
    }

    /// 是否为移动平台
    pub fn is_mobile(&self) -> bool {
        matches!(self, Self::Android | Self::Ios)
    }

    /// 是否为 Web 平台
    pub fn is_web(&self) -> bool {
        matches!(self, Self::Web)
    }

    /// 平台名称字符串
    pub fn name(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOs => "macOS",
            Self::Linux => "Linux",
            Self::Web => "Web",
            Self::Android => "Android",
            Self::Ios => "iOS",
            Self::Unknown => "Unknown",
        }
    }

    /// 是否支持文件系统
    pub fn supports_filesystem(&self) -> bool {
        !self.is_web()
    }

    /// 是否支持 Wasm 插件加载
    pub fn supports_wasm_plugins(&self) -> bool {
        // Wasmtime 仅在桌面平台上工作
        self.is_desktop()
    }

    /// 是否支持多线程
    pub fn supports_threads(&self) -> bool {
        !self.is_web() // Web 平台线程支持有限制
    }

    /// 推荐的物理帧率
    pub fn recommended_physics_fps(&self) -> u32 {
        60
    }

    /// 推荐的最大渲染 FPS
    pub fn recommended_max_fps(&self) -> u32 {
        match self {
            Self::Web => 60, // 浏览器使用 requestAnimationFrame
            Self::Android | Self::Ios => 60,
            _ => 0, // 桌面不限制
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ── 平台特定能力 ──────────────────────────────────────────────────────────────

/// 平台能力信息（运行时查询）
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub platform: Platform,
    /// 逻辑 CPU 核心数
    pub cpu_cores: usize,
    /// 可用内存（字节，0 = 未知）
    pub available_memory: u64,
    /// 是否支持触控
    pub touch_support: bool,
    /// 是否支持游戏手柄
    pub gamepad_support: bool,
}

impl PlatformCapabilities {
    /// 检测当前平台能力
    pub fn detect() -> Self {
        let platform = Platform::current();
        Self {
            platform,
            cpu_cores: num_cpus(),
            available_memory: 0, // 需要平台特定 API
            touch_support: platform.is_mobile() || platform.is_web(),
            gamepad_support: platform.is_desktop() || platform.is_web(),
        }
    }
}

/// 获取 CPU 核心数
fn num_cpus() -> usize {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
}

// ── 构建目标信息 ──────────────────────────────────────────────────────────────

/// 构建配置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub fn current() -> Self {
        if cfg!(debug_assertions) {
            Self::Debug
        } else {
            Self::Release
        }
    }

    pub fn is_debug(&self) -> bool {
        matches!(self, Self::Debug)
    }

    pub fn is_release(&self) -> bool {
        matches!(self, Self::Release)
    }
}
