//! # Ummerse Plugin
//!
//! 插件系统与扩展 API，参考 VSCode + Cline 模式：
//! - 插件宿主（Plugin Host）
//! - 工具调用协议（Tool Call Protocol）- 类似 Cline 的 AI 工具
//! - Wasm 插件支持（sandboxed 执行）
//! - 插件通信（消息传递）

pub mod host;
pub mod manifest;
pub mod protocol;
pub mod tool;
pub mod wasm_plugin;

pub use host::{PluginHost, PluginInstance};
pub use manifest::{PluginManifest, PluginCapability};
pub use protocol::{ToolCall, ToolResult, ToolResultContent};
pub use tool::{EngineTool, ToolRegistry};

use thiserror::Error;

/// 插件系统错误
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("插件未找到: {0}")]
    NotFound(String),
    #[error("插件加载失败: {name}, 原因: {reason}")]
    LoadFailed { name: String, reason: String },
    #[error("工具调用失败: {tool}, 原因: {reason}")]
    ToolCallFailed { tool: String, reason: String },
    #[error("插件通信错误: {0}")]
    Communication(String),
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    #[error("序列化错误: {0}")]
    Serialization(String),
}

/// 插件系统 Result
pub type Result<T> = std::result::Result<T, PluginError>;
