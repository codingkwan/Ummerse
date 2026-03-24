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
    /// 工具名称（snake_case）
    pub name: String,
    /// 工具参数（JSON 对象）
    pub parameters: serde_json::Value,
}

impl ToolCall {
    /// 创建新工具调用（自动生成 ID）
    pub fn new(name: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            parameters,
        }
    }

    /// 使用指定 ID 创建工具调用
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parameters,
        }
    }

    /// 获取字符串参数
    pub fn get_str<'a>(&'a self, key: &str) -> Option<&'a str> {
        self.parameters.get(key)?.as_str()
    }

    /// 获取布尔参数
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.parameters.get(key)?.as_bool()
    }

    /// 获取数字参数（f64）
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.parameters.get(key)?.as_f64()
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
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
    /// 结果内容列表（可包含多段不同类型的内容）
    pub content: Vec<ToolResultContent>,
    /// 是否出错
    pub is_error: bool,
}

impl ToolResult {
    /// 创建成功结果
    pub fn success(
        id: impl Into<String>,
        name: impl Into<String>,
        content: Vec<ToolResultContent>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            content,
            is_error: false,
        }
    }

    /// 创建错误结果
    pub fn error(
        id: impl Into<String>,
        name: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            content: vec![ToolResultContent::Text { text: message.into() }],
            is_error: true,
        }
    }

    /// 创建纯文本成功结果（快捷方式）
    pub fn ok_text(
        id: impl Into<String>,
        name: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self::success(id, name, vec![ToolResultContent::Text { text: text.into() }])
    }

    /// 获取第一段文本内容
    pub fn text(&self) -> Option<&str> {
        for c in &self.content {
            if let ToolResultContent::Text { text } = c {
                return Some(text.as_str());
            }
        }
        None
    }
}

/// 工具结果内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContent {
    /// 纯文本内容
    Text { text: String },
    /// JSON 结构化数据
    Json { data: serde_json::Value },
    /// 二进制数据（Base64 编码）
    Binary { data: String, mime_type: String },
    /// 图像（Base64 编码）
    Image { data: String, media_type: String },
}

// ── 插件消息协议 ──────────────────────────────────────────────────────────────

/// 插件消息（引擎 ↔ 插件双向通信）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginMessage {
    /// 插件初始化完成通知
    Initialize {
        plugin_id: String,
        version: String,
        capabilities: Vec<String>,
    },
    /// 工具调用请求（插件 → 引擎）
    ToolCall(ToolCall),
    /// 工具调用响应（引擎 → 插件）
    ToolResult(ToolResult),
    /// 日志输出（插件 → 引擎）
    Log {
        level: LogLevel,
        message: String,
        plugin_id: String,
    },
    /// 订阅编辑器事件
    SubscribeEvent { event_type: String },
    /// 取消订阅编辑器事件
    UnsubscribeEvent { event_type: String },
    /// 编辑器事件通知（引擎 → 插件）
    EditorEvent {
        event_type: String,
        data: serde_json::Value,
    },
    /// 心跳（保持连接）
    Ping { seq: u64 },
    /// 心跳响应
    Pong { seq: u64 },
    /// 错误
    Error { code: String, message: String },
    /// 关闭
    Shutdown,
}

impl PluginMessage {
    /// 序列化为 JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// 从 JSON 反序列化
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// 是否为控制消息（不需要等待响应）
    pub fn is_fire_and_forget(&self) -> bool {
        matches!(
            self,
            Self::Log { .. }
                | Self::SubscribeEvent { .. }
                | Self::UnsubscribeEvent { .. }
                | Self::Shutdown
        )
    }
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

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_call_serialization() {
        let call = ToolCall::new("read_file", serde_json::json!({ "path": "src/main.rs" }));
        let json = call.to_json().unwrap();
        let deserialized = ToolCall::from_json(&json).unwrap();
        assert_eq!(deserialized.name, "read_file");
        assert_eq!(deserialized.get_str("path"), Some("src/main.rs"));
    }

    #[test]
    fn test_tool_result_ok_text() {
        let result = ToolResult::ok_text("id-1", "read_file", "file contents here");
        assert!(!result.is_error);
        assert_eq!(result.text(), Some("file contents here"));
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("id-2", "write_file", "Permission denied");
        assert!(result.is_error);
        assert_eq!(result.text(), Some("Permission denied"));
    }

    #[test]
    fn test_plugin_message_serialization() {
        let msg = PluginMessage::ToolCall(ToolCall::new(
            "get_scene_tree",
            serde_json::json!({}),
        ));
        let json = msg.to_json().unwrap();
        let deserialized = PluginMessage::from_json(&json).unwrap();
        assert!(matches!(deserialized, PluginMessage::ToolCall(_)));
    }
}
