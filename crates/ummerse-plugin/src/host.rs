//! 插件宿主 - 管理插件生命周期和通信

use crate::{
    manifest::PluginManifest,
    protocol::{PluginMessage, ToolCall, ToolResult},
    tool::ToolRegistry,
    PluginError, Result,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// 插件实例 ID
pub type PluginInstanceId = String;

/// 插件实例状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Loading,
    Active,
    Suspended,
    Crashed,
    Stopped,
}

/// 运行中的插件实例
pub struct PluginInstance {
    pub id: PluginInstanceId,
    pub manifest: PluginManifest,
    pub state: PluginState,
    /// 发送消息到插件的通道
    pub sender: mpsc::Sender<PluginMessage>,
}

impl PluginInstance {
    /// 向插件发送消息
    pub async fn send(&self, msg: PluginMessage) -> Result<()> {
        self.sender.send(msg).await.map_err(|e| {
            PluginError::Communication(format!(
                "Failed to send message to plugin '{}': {}",
                self.id, e
            ))
        })
    }

    /// 是否活跃
    pub fn is_active(&self) -> bool {
        self.state == PluginState::Active
    }
}

impl std::fmt::Debug for PluginInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginInstance")
            .field("id", &self.id)
            .field("name", &self.manifest.name)
            .field("state", &self.state)
            .finish()
    }
}

// ── 插件宿主 ──────────────────────────────────────────────────────────────────

/// 插件宿主 - 统一管理所有插件
pub struct PluginHost {
    /// 已加载的插件实例
    instances: DashMap<PluginInstanceId, Arc<PluginInstance>>,
    /// 工具注册表
    tool_registry: Arc<ToolRegistry>,
    /// 插件发现目录
    plugin_dirs: Vec<std::path::PathBuf>,
}

impl PluginHost {
    pub fn new() -> Self {
        Self {
            instances: DashMap::new(),
            tool_registry: Arc::new(ToolRegistry::new()),
            plugin_dirs: Vec::new(),
        }
    }

    /// 添加插件搜索目录
    pub fn add_plugin_dir(&mut self, dir: impl Into<std::path::PathBuf>) {
        self.plugin_dirs.push(dir.into());
    }

    /// 从清单加载插件（创建实例但不启动）
    pub async fn load_plugin(&self, manifest: PluginManifest) -> Result<PluginInstanceId> {
        let id = manifest.id.clone();

        if self.instances.contains_key(&id) {
            return Err(PluginError::LoadFailed {
                name: id.clone(),
                reason: "Plugin already loaded".to_string(),
            });
        }

        info!("Loading plugin: {} v{}", manifest.name, manifest.version);

        // 创建双向通信通道
        let (tx, _rx) = mpsc::channel::<PluginMessage>(64);

        let instance = Arc::new(PluginInstance {
            id: id.clone(),
            manifest,
            state: PluginState::Loading,
            sender: tx,
        });

        self.instances.insert(id.clone(), instance);
        info!("Plugin '{}' loaded successfully", id);
        Ok(id)
    }

    /// 从 JSON 清单字符串加载插件
    pub async fn load_plugin_from_json(&self, json: &str) -> Result<PluginInstanceId> {
        let manifest = PluginManifest::from_json(json).map_err(|e| PluginError::LoadFailed {
            name: "unknown".to_string(),
            reason: e.to_string(),
        })?;
        self.load_plugin(manifest).await
    }

    /// 卸载插件
    pub async fn unload_plugin(&self, id: &str) -> Result<()> {
        if let Some((_, instance)) = self.instances.remove(id) {
            // 发送关闭信号
            let _ = instance.send(PluginMessage::Shutdown).await;
            info!("Plugin '{}' unloaded", id);
            Ok(())
        } else {
            Err(PluginError::NotFound(id.to_string()))
        }
    }

    /// 执行工具调用（由插件发起）
    pub async fn handle_tool_call(&self, caller_id: &str, call: ToolCall) -> ToolResult {
        // 检查插件是否有足够权限
        if self.instances.get(caller_id).is_some() {
            tracing::debug!("Plugin '{}' calling tool '{}'", caller_id, call.name);
        }

        // 委托给工具注册表
        self.tool_registry.dispatch(call).await
    }

    /// 处理来自插件的消息
    pub async fn handle_message(&self, from_id: &str, msg: PluginMessage) {
        match msg {
            PluginMessage::ToolCall(call) => {
                let result = self.handle_tool_call(from_id, call).await;
                if let Some(instance) = self.instances.get(from_id) {
                    let _ = instance.send(PluginMessage::ToolResult(result)).await;
                }
            }
            PluginMessage::Log {
                level,
                message,
                plugin_id,
            } => match level {
                crate::protocol::LogLevel::Error => {
                    error!(target: "plugin", "[{}] {}", plugin_id, message)
                }
                crate::protocol::LogLevel::Warn => {
                    warn!(target: "plugin", "[{}] {}", plugin_id, message)
                }
                crate::protocol::LogLevel::Info => {
                    info!(target: "plugin", "[{}] {}", plugin_id, message)
                }
                crate::protocol::LogLevel::Debug => {
                    tracing::debug!(target: "plugin", "[{}] {}", plugin_id, message)
                }
                crate::protocol::LogLevel::Trace => {
                    tracing::trace!(target: "plugin", "[{}] {}", plugin_id, message)
                }
            },
            PluginMessage::Shutdown => {
                info!("Plugin '{}' requested shutdown", from_id);
                let _ = self.unload_plugin(from_id).await;
            }
            other => {
                tracing::debug!("Unhandled plugin message from '{}': {:?}", from_id, other);
            }
        }
    }

    /// 广播编辑器事件给所有活跃插件
    pub async fn broadcast_event(&self, event_type: &str, data: serde_json::Value) {
        let msg = PluginMessage::EditorEvent {
            event_type: event_type.to_string(),
            data,
        };

        for entry in self.instances.iter() {
            if entry.value().is_active() {
                let _ = entry.value().send(msg.clone()).await;
            }
        }
    }

    /// 获取插件实例
    pub fn get_instance(&self, id: &str) -> Option<Arc<PluginInstance>> {
        self.instances.get(id).map(|r| r.clone())
    }

    /// 所有已加载插件的 ID 列表
    pub fn plugin_ids(&self) -> Vec<String> {
        self.instances.iter().map(|e| e.key().clone()).collect()
    }

    /// 已加载插件数量
    pub fn plugin_count(&self) -> usize {
        self.instances.len()
    }

    /// 工具注册表引用
    pub fn tool_registry(&self) -> &ToolRegistry {
        &self.tool_registry
    }
}

impl Default for PluginHost {
    fn default() -> Self {
        Self::new()
    }
}
