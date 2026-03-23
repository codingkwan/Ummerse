//! 工具调用协议 - 类似 Cline/MCP 的工具调用接口
//!
//! 设计参考：
//! - Anthropic Claude Tool Use API
//! - Model Context Protocol (MCP)
//! - VSCode 扩展 API
//!
//! 插件通过此协议向引擎发送工具调用请求，
//! 引擎执行后返回结果。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── 工具调用请求 ──────────────────────────────────────────────────────────────

/// 工具调用请求（插件 → 引擎）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 请求唯一 ID（用于匹配响应）
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 工具参数（JSON 对象）
    pub parameters: serde_json::Value,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            parameters,
        }
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

// ── 工具调用结果 ──────────────────────────────────────────────────────────────

/// 工具调用结果（引擎 → 插件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 对应请求的 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 结果内容列表
    pub content: Vec<ToolResultContent>,
    /// 是否出错
    pub is_error: bool,
}

impl ToolResult {
    /// 创建成功结果
    pub fn success(id: impl Into<String>, name: impl Into<String>, content: Vec<ToolResultContent>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            content,
            is_error: false,
        }
    }

    /// 创建错误结果
    pub fn error(id: impl Into<String>, name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            content: vec![ToolResultContent::Text { text: message.into() }],
            is_error: true,
        }
    }

    /// 创建文本成功结果（快捷方式）
    pub fn ok_text(id: impl Into<String>, name: impl Into<String>, text: impl Into<String>) -> Self {
        Self::success(id, name, vec![ToolResultContent::Text { text: text.into() }])
    }
}

/// 工具结果内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContent {
    /// 纯文本
    Text { text: String },
    /// JSON 数据
    Json { data: serde_json::Value },
    /// 二进制数据（Base64 编码）
    Binary { data: String, mime_type: String },
    /// 图像（Base64 编码）
    Image { data: String, media_type: String },
}

// ── 插件消息协议 ──────────────────────────────────────────────────────────────

/// 插件消息（双向通信）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginMessage {
    /// 插件初始化完成通知
    Initialize {
        plugin_id: String,
        version: String,
    },
    /// 工具调用请求
    ToolCall(ToolCall),
    /// 工具调用响应
    ToolResult(ToolResult),
    /// 日志输出
    Log {
        level: LogLevel,
        message: String,
        plugin_id: String,
    },
    /// 编辑器事件订阅
    SubscribeEvent {
        event_type: String,
    },
    /// 编辑器事件通知
    EditorEvent {
        event_type: String,
        data: serde_json::Value,
    },
    /// 错误
    Error {
        code: String,
        message: String,
    },
    /// 关闭
    Shutdown,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl PluginMessage {
    /// 序列化为 JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
