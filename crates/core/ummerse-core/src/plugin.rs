//! 插件系统接口
//!
//! 所有引擎功能均以插件形式组织，类似 Bevy 的插件模式。
//! 插件可注册 ECS 系统、资源、事件等，并支持依赖声明。

use crate::error::Result;

/// 插件 trait - 所有引擎插件须实现此接口
///
/// # 示例
/// ```rust,no_run
/// use ummerse_core::plugin::Plugin;
/// use ummerse_core::app::AppBuilder;
///
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn name(&self) -> &str { "MyPlugin" }
///
///     fn build(&self, app: &mut AppBuilder) {
///         // 注册系统、资源、事件等
///         tracing::info!("MyPlugin built");
///     }
/// }
/// ```
pub trait Plugin: Send + Sync + 'static {
    /// 插件唯一名称（用于依赖解析和日志）
    fn name(&self) -> &str;

    /// 语义化版本字符串（默认 "0.1.0"）
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// 插件功能描述
    fn description(&self) -> &str {
        ""
    }

    /// 构建插件（注册系统、资源、事件等）
    ///
    /// 此方法在 [`AppBuilder::build`] 时调用。
    fn build(&self, app: &mut crate::app::AppBuilder);

    /// 插件清理（可选，在 App drop 时逆序调用）
    fn cleanup(&self) {}

    /// 声明依赖的其他插件名称列表
    ///
    /// 依赖的插件会在此插件 `build` 之前构建。
    fn dependencies(&self) -> &[&str] {
        &[]
    }

    /// 是否允许重复注册（默认 false，重复注册时报错）
    fn allow_duplicates(&self) -> bool {
        false
    }
}

// ── 插件注册表 ────────────────────────────────────────────────────────────────

/// 插件注册表 - 按依赖顺序管理所有插件
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("plugin_count", &self.plugins.len())
            .finish_non_exhaustive()
    }
}

impl PluginRegistry {
    /// 创建空的插件注册表
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// 注册插件
    ///
    /// # 错误
    /// - 插件名重复且 `allow_duplicates = false` 时返回错误
    pub fn register(&mut self, plugin: impl Plugin) -> Result<()> {
        let name = plugin.name().to_string();

        if !plugin.allow_duplicates() && self.plugins.iter().any(|p| p.name() == name) {
            return Err(crate::error::EngineError::PluginError {
                name,
                reason: "Plugin already registered".to_string(),
            });
        }

        tracing::debug!("Registered plugin: {} v{}", plugin.name(), plugin.version());
        self.plugins.push(Box::new(plugin));
        Ok(())
    }

    /// 获取所有已注册插件名称
    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// 构建所有插件（按注册顺序）
    pub fn build_all(&self, app: &mut crate::app::AppBuilder) {
        for plugin in &self.plugins {
            tracing::info!("Building plugin: {} v{}", plugin.name(), plugin.version());
            plugin.build(app);
        }
    }

    /// 清理所有插件（逆序）
    pub fn cleanup_all(&self) {
        for plugin in self.plugins.iter().rev() {
            plugin.cleanup();
        }
    }

    /// 插件数量
    #[inline]
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppBuilder;

    struct DummyPlugin;
    impl Plugin for DummyPlugin {
        fn name(&self) -> &str {
            "DummyPlugin"
        }
        fn build(&self, _app: &mut AppBuilder) {}
    }

    #[test]
    fn test_register_plugin() {
        let mut registry = PluginRegistry::new();
        assert!(registry.register(DummyPlugin).is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_duplicate_plugin_rejected() {
        let mut registry = PluginRegistry::new();
        registry.register(DummyPlugin).unwrap();
        let result = registry.register(DummyPlugin);
        assert!(result.is_err());
    }
}
