//! 应用构建器 - 链式 API 配置引擎

use crate::{
    engine::EngineConfig,
    plugin::Plugin,
    resource::ResourceRegistry,
};

/// 应用构建器（Builder 模式）
///
/// # 示例
/// ```rust
/// use ummerse_core::app::AppBuilder;
///
/// let app = AppBuilder::new()
///     .title("My Game")
///     .window_size(1280, 720)
///     .add_plugin(MyPlugin)
///     .build();
/// ```
pub struct AppBuilder {
    pub config: EngineConfig,
    pub resources: ResourceRegistry,
    plugins: Vec<Box<dyn Plugin>>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            resources: ResourceRegistry::new(),
            plugins: Vec::new(),
        }
    }

    /// 设置窗口标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.config.window.title = title.into();
        self
    }

    /// 设置窗口尺寸
    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.window.width = width;
        self.config.window.height = height;
        self
    }

    /// 设置是否全屏
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.config.window.fullscreen = fullscreen;
        self
    }

    /// 设置 VSync
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.config.window.vsync = vsync;
        self
    }

    /// 设置目标帧率（0 表示不限制）
    pub fn target_fps(mut self, fps: u32) -> Self {
        self.config.engine.target_fps = fps;
        self
    }

    /// 设置物理帧率
    pub fn physics_fps(mut self, fps: u32) -> Self {
        self.config.engine.physics_fps = fps;
        self
    }

    /// 添加插件
    pub fn add_plugin(mut self, plugin: impl Plugin) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    /// 注册全局资源
    pub fn insert_resource<R: crate::resource::Resource>(mut self, resource: R) -> Self {
        self.resources.insert(resource);
        self
    }

    /// 构建应用
    pub fn build(mut self) -> App {
        // 构建所有插件
        let plugins = std::mem::take(&mut self.plugins);
        let mut this = self;
        for plugin in &plugins {
            tracing::info!("Building plugin: {} v{}", plugin.name(), plugin.version());
            plugin.build(&mut this);
        }
        App {
            config: this.config,
            resources: this.resources,
            plugins,
        }
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 已构建的应用实例
pub struct App {
    pub config: EngineConfig,
    pub resources: ResourceRegistry,
    plugins: Vec<Box<dyn Plugin>>,
}

impl App {
    /// 创建新的 AppBuilder
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    /// 插件数量
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // 反向清理插件
        for plugin in self.plugins.iter().rev() {
            plugin.cleanup();
        }
    }
}
