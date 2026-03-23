//! AI 助手 - Cline 风格的 AI 工具调用集成
//!
//! 提供类似 Cline 的 AI 助手功能：
//! - 与 LLM 对话（支持接入 Claude/OpenAI 等）
//! - AI 发起工具调用（修改场景、编写脚本等）
//! - 对话历史管理
//! - 用户确认机制

use serde::{Deserialize, Serialize};

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiRole {
    /// 用户
    User,
    /// AI 助手
    Assistant,
    /// 系统提示
    System,
    /// 工具结果
    Tool,
}

/// AI 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: AiRole,
    pub content: MessageContent,
    /// 时间戳（Unix ms）
    pub timestamp: u64,
}

impl AiMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: AiRole::User,
            content: MessageContent::Text(text.into()),
            timestamp: current_timestamp(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: AiRole::Assistant,
            content: MessageContent::Text(text.into()),
            timestamp: current_timestamp(),
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: AiRole::System,
            content: MessageContent::Text(text.into()),
            timestamp: current_timestamp(),
        }
    }

    pub fn tool_result(tool_name: impl Into<String>, result: impl Into<String>) -> Self {
        Self {
            role: AiRole::Tool,
            content: MessageContent::ToolResult {
                tool_name: tool_name.into(),
                result: result.into(),
            },
            timestamp: current_timestamp(),
        }
    }
}

/// 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    /// 纯文本
    Text(String),
    /// 工具调用（AI 发起）
    ToolCall {
        tool_name: String,
        parameters: serde_json::Value,
    },
    /// 工具调用结果（引擎返回）
    ToolResult { tool_name: String, result: String },
    /// 思考过程（扩展推理）
    Thinking { thought: String },
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ── 对话状态机 ────────────────────────────────────────────────────────────────

/// 对话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    /// 等待用户输入
    WaitingForUser,
    /// 正在生成 AI 响应
    Generating,
    /// 等待工具调用确认
    WaitingForApproval,
    /// 正在执行工具调用
    ExecutingTool,
    /// 出错
    Error,
}

/// 待执行的工具调用（等待用户确认）
#[derive(Debug, Clone)]
pub struct PendingToolCall {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub description: String,
    pub requires_approval: bool,
}

/// AI 助手实例
pub struct AiAssistant {
    /// 对话历史
    pub messages: Vec<AiMessage>,
    /// 当前状态
    pub state: ConversationState,
    /// 待确认的工具调用队列
    pub pending_calls: Vec<PendingToolCall>,
    /// 系统提示
    system_prompt: String,
    /// LLM 后端配置
    pub backend: AiBackendConfig,
}

impl AiAssistant {
    pub fn new() -> Self {
        let system_prompt = include_str!("system_prompt.txt").to_string();
        Self {
            messages: Vec::new(),
            state: ConversationState::WaitingForUser,
            pending_calls: Vec::new(),
            system_prompt,
            backend: AiBackendConfig::default(),
        }
    }

    /// 发送用户消息
    pub fn send_user_message(&mut self, text: impl Into<String>) {
        let msg = AiMessage::user(text);
        self.messages.push(msg);
        self.state = ConversationState::Generating;
    }

    /// 添加 AI 响应（流式或完整）
    pub fn add_assistant_message(&mut self, text: impl Into<String>) {
        let msg = AiMessage::assistant(text);
        self.messages.push(msg);
        self.state = ConversationState::WaitingForUser;
    }

    /// 添加工具调用（AI 发起）
    pub fn add_tool_call(
        &mut self,
        tool_name: impl Into<String>,
        params: serde_json::Value,
        description: impl Into<String>,
        needs_approval: bool,
    ) {
        let call = PendingToolCall {
            tool_name: tool_name.into(),
            parameters: params,
            description: description.into(),
            requires_approval: needs_approval,
        };

        if needs_approval {
            self.pending_calls.push(call);
            self.state = ConversationState::WaitingForApproval;
        } else {
            self.pending_calls.push(call);
            self.state = ConversationState::ExecutingTool;
        }
    }

    /// 用户批准下一个工具调用
    pub fn approve_next_call(&mut self) -> Option<PendingToolCall> {
        if !self.pending_calls.is_empty() {
            let call = self.pending_calls.remove(0);
            self.state = ConversationState::ExecutingTool;
            Some(call)
        } else {
            None
        }
    }

    /// 用户拒绝下一个工具调用
    pub fn reject_next_call(&mut self, reason: impl Into<String>) {
        if !self.pending_calls.is_empty() {
            let call = self.pending_calls.remove(0);
            let msg = AiMessage::tool_result(
                &call.tool_name,
                format!("Tool call rejected by user: {}", reason.into()),
            );
            self.messages.push(msg);
            self.state = ConversationState::WaitingForUser;
        }
    }

    /// 添加工具执行结果
    pub fn add_tool_result(&mut self, tool_name: impl Into<String>, result: impl Into<String>) {
        let msg = AiMessage::tool_result(tool_name, result);
        self.messages.push(msg);
        if self.pending_calls.is_empty() {
            self.state = ConversationState::Generating;
        }
    }

    /// 清除对话历史
    pub fn clear_history(&mut self) {
        self.messages.clear();
        self.pending_calls.clear();
        self.state = ConversationState::WaitingForUser;
    }

    /// 获取所有消息（供 LLM API 使用），首条为 system 提示，返回拥有数据
    pub fn messages_for_api(&self) -> Vec<AiMessage> {
        let mut result = Vec::with_capacity(self.messages.len() + 1);
        result.push(AiMessage {
            role: AiRole::System,
            content: MessageContent::Text(self.system_prompt.clone()),
            timestamp: 0,
        });
        result.extend(self.messages.iter().cloned());
        result
    }

    /// 系统提示文本
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    /// 更新系统提示
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = prompt.into();
    }
}

impl Default for AiAssistant {
    fn default() -> Self {
        Self::new()
    }
}

// ── LLM 后端配置 ──────────────────────────────────────────────────────────────

/// LLM 后端类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiBackendKind {
    /// Anthropic Claude API
    Claude,
    /// OpenAI GPT API
    OpenAi,
    /// 本地 Ollama
    Ollama,
    /// 自定义 OpenAI 兼容 API
    Custom,
    /// 未配置（离线模式）
    None,
}

/// LLM 后端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBackendConfig {
    pub kind: AiBackendKind,
    /// API 密钥（存储在系统密钥环或环境变量中）
    pub api_key_env: Option<String>,
    /// API 基础 URL
    pub base_url: Option<String>,
    /// 模型名称
    pub model: String,
    /// 最大 Token 数
    pub max_tokens: u32,
    /// Temperature（0.0 ~ 1.0）
    pub temperature: f32,
}

impl Default for AiBackendConfig {
    fn default() -> Self {
        Self {
            kind: AiBackendKind::None,
            api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
            base_url: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 8192,
            temperature: 0.7,
        }
    }
}
