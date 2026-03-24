//! 应用构建器 - 链式 API 配置引擎
//!
//! 采用 Builder 模式，支持链式调用配置引擎各项参数和插件。

use crate::{engine::EngineConfig, plugin::Plugin, resource::ResourceRegistry};

/// 应用构建器（Builder 模式）
///
/// # 示例
/// ```rust
/// use ummerse_core::app::AppBuilder;
///
/// let app = AppBuilder::new()
///     .title("My Game")
///     .window_size(1280, 720)
///     .build();
/// ```
pub struct AppBuilder {
    /// 引擎配置
    pub config: EngineConfig,
    /// 全局资源注册表
    pub resources: ResourceRegistry,
    /// 待构建的插件列表
    plugins: Vec<Box<dyn Plugin>>,
}

impl std::fmt::Debug for AppBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppBuilder")
            .field("config", &self.config)
            .field("resources", &self.resources)
            .field("plugin_count", &self.plugins.len())
            .finish_non_exhaustive()
    }
}

impl AppBuilder {
    /// 创建默认 AppBuilder
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            resources: ResourceRegistry::new(),
            plugins: Vec::new(),
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

    /// 设置目标帧率（0 表示不限制）
    #[inline]
    pub fn target_fps(mut self, fps: u32) -> Self {
        self.config.engine.target_fps = fps;
        self
    }

    /// 设置物理更新帧率
    #[inline]
    pub fn physics_fps(mut self, fps: u32) -> Self {
        self.config.engine.physics_fps = fps;
        self
    }

    /// 添加插件（接受任何实现 [`Plugin`] 的类型）
    pub fn add_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// 注册全局资源
    pub fn insert_resource<R: crate::resource::Resource>(mut self, resource: R) -> Self {
        self.resources.insert(resource);
        self
    }

    /// 构建最终 [`App`]，触发所有插件的 `build` 回调
    pub fn build(mut self) -> App {
        let plugins = std::mem::take(&mut self.plugins);
        for plugin in &plugins {
            tracing::debug!("Building plugin: {} v{}", plugin.name(), plugin.version());
            plugin.build(&mut self);
        }
        App {
            config: self.config,
            resources: self.resources,
            plugins,
        }
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── 已构建的应用实例 ──────────────────────────────────────────────────────────

/// 已构建的应用实例
///
/// 通过 [`AppBuilder::build`] 创建，持有所有插件、资源和配置。
pub struct App {
    /// 引擎全局配置
    pub config: EngineConfig,
    /// 全局资源注册表
    pub resources: ResourceRegistry,
    /// 已注册的插件列表
    plugins: Vec<Box<dyn Plugin>>,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("config", &self.config)
            .field("resources", &self.resources)
            .field("plugin_count", &self.plugins.len())
            .finish_non_exhaustive()
    }
}

impl App {
    /// 创建新的 [`AppBuilder`]
    #[inline]
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    /// 返回当前插件数量
    #[inline]
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// 获取引擎配置（不可变）
    #[inline]
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // 按注册逆序清理插件，确保依赖正确释放
        for plugin in self.plugins.iter().rev() {
            plugin.cleanup();
        }
    }
}
