//! 插件系统接口

use crate::error::Result;

/// 插件 trait - 所有引擎插件需实现此接口
///
/// # 示例
/// ```rust
/// use ummerse_core::plugin::Plugin;
/// use ummerse_core::app::AppBuilder;
///
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn name(&self) -> &str { "MyPlugin" }
///     fn build(&self, app: &mut AppBuilder) {
///         // 注册系统、资源等
///     }
/// }
/// ```
pub trait Plugin: Send + Sync + 'static {
    /// 插件名称
    fn name(&self) -> &str;

    /// 插件版本（语义化版本字符串）
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// 插件描述
    fn description(&self) -> &str {
        ""
    }

    /// 构建插件（注册系统、资源、事件等）
    fn build(&self, app: &mut crate::app::AppBuilder);

    /// 插件清理（可选）
    fn cleanup(&self) {}

    /// 依赖的其他插件名称列表
    fn dependencies(&self) -> &[&str] {
        &[]
    }
}

/// 插件注册表
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: impl Plugin) -> Result<()> {
        // 检查重名插件
        if self.plugins.iter().any(|p| p.name() == plugin.name()) {
            return Err(crate::error::EngineError::PluginError {
                name: plugin.name().to_string(),
                reason: "Plugin already registered".to_string(),
            });
        }
        self.plugins.push(Box::new(plugin));
        Ok(())
    }

    /// 获取所有插件名
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// 构建所有插件
    pub fn build_all(&self, app: &mut crate::app::AppBuilder) {
        for plugin in &self.plugins {
            tracing::info!("Building plugin: {} v{}", plugin.name(), plugin.version());
            plugin.build(app);
        }
    }

    /// 清理所有插件
    pub fn cleanup_all(&self) {
        for plugin in &self.plugins {
            plugin.cleanup();
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
