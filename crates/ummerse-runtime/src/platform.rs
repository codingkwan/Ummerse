//! 平台抽象层 - 处理桌面/Web 平台差异

/// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Desktop,
    Web,
    Mobile,
}

/// 平台信息
pub struct Platform {
    pub kind: PlatformKind,
    pub os: &'static str,
    pub arch: &'static str,
}

impl Platform {
    /// 检测当前平台
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                kind: PlatformKind::Web,
                os: "web",
                arch: "wasm32",
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
        {
            Self {
                kind: PlatformKind::Desktop,
                os: "windows",
                arch: std::env::consts::ARCH,
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
        {
            Self {
                kind: PlatformKind::Desktop,
                os: "macos",
                arch: std::env::consts::ARCH,
            }
        }

        #[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
        {
            Self {
                kind: PlatformKind::Desktop,
                os: "linux",
                arch: std::env::consts::ARCH,
            }
        }

        #[cfg(all(
            not(target_arch = "wasm32"),
            not(target_os = "windows"),
            not(target_os = "macos"),
            not(target_os = "linux")
        ))]
        {
            Self {
                kind: PlatformKind::Desktop,
                os: std::env::consts::OS,
                arch: std::env::consts::ARCH,
            }
        }
    }

    /// 是否为 Web 平台
    #[inline]
    pub fn is_web(&self) -> bool {
        self.kind == PlatformKind::Web
    }

    /// 是否为桌面平台
    #[inline]
    pub fn is_desktop(&self) -> bool {
        self.kind == PlatformKind::Desktop
    }

    /// 是否支持文件系统访问
    #[inline]
    pub fn supports_filesystem(&self) -> bool {
        !self.is_web()
    }

    /// 是否支持多线程
    #[inline]
    pub fn supports_threads(&self) -> bool {
        // WASM 在没有 SharedArrayBuffer 时不支持真正的多线程
        !self.is_web()
    }

    /// 平台描述字符串
    pub fn description(&self) -> String {
        format!("{}/{}", self.os, self.arch)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.os, self.arch)
    }
}
