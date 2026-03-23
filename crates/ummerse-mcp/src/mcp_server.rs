//! MCP Server 核心实现
//!
//! 基于 JSON-RPC 2.0 over stdio 实现 MCP 协议，让 Cline 等 AI Agent
//! 能够通过标准输入/输出与引擎通信。
//!
//! ## MCP 协议流程
//! ```text
//! Cline ──► initialize  ──► capabilities + tool list
//! Cline ──► tools/call  ──► tool result
//! Cline ──► ping        ──► pong
//! ```
//!
//! ## 协议规范
//! - 每条消息为一行 JSON，以 `\n` 结尾
//! - 请求格式: `{"jsonrpc":"2.0","id":1,"method":"...","params":{...}}`
//! - 通知格式: `{"jsonrpc":"2.0","method":"...","params":{...}}`（无 id）

use crate::engine_bridge::EngineBridge;
use crate::tools::{ToolOutput, all_tool_defs, dispatch_tool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

// ── JSON-RPC 2.0 数据结构 ─────────────────────────────────────────────────────

/// JSON-RPC 2.0 请求（同时兼容通知，通知无 id 字段）
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    /// 请求 ID（通知消息无此字段）
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 响应（成功）
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    pub result: Value,
}

/// JSON-RPC 2.0 错误响应
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub id: Value,
    pub error: RpcError,
}

/// JSON-RPC 错误对象
#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// 标准 JSON-RPC 错误码
#[allow(dead_code)]
const PARSE_ERROR: i32 = -32700;
#[allow(dead_code)]
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

// ── MCP 服务器 ────────────────────────────────────────────────────────────────

/// MCP Server 主体
///
/// 维护引擎桥接句柄，通过 stdio 与 AI 客户端通信。
pub struct McpServer {
    /// 引擎桥接（状态共享）
    bridge: EngineBridge,
    /// 服务器名称
    name: String,
    /// 服务器版本
    version: String,
}

impl std::fmt::Debug for McpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServer")
            .field("name", &self.name)
            .field("version", &self.version)
            .finish()
    }
}

impl McpServer {
    /// 创建新的 MCP Server
    pub fn new(bridge: EngineBridge) -> Self {
        Self {
            bridge,
            name: "ummerse-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// 启动 stdio 事件循环（阻塞运行直到 stdin 关闭）
    pub fn run_stdio(&self) -> anyhow::Result<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());

        tracing::info!(
            "🚀 Ummerse MCP Server v{} started (stdio transport)",
            self.version
        );
        tracing::info!("📡 Waiting for MCP client connection...");

        let reader = stdin.lock();
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("stdin read error: {e}");
                    break;
                }
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            tracing::debug!("◀ recv: {}", trimmed);

            // 解析 JSON-RPC 请求
            match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                Ok(req) => {
                    let response = self.handle_request(&req);
                    if let Some(resp) = response {
                        let json = serde_json::to_string(&resp)?;
                        tracing::debug!("▶ send: {}", json);
                        writeln!(out, "{json}")?;
                        out.flush()?;
                    }
                }
                Err(e) => {
                    // 解析失败，返回 parse error（id 未知，使用 null）
                    let err = JsonRpcError {
                        jsonrpc: "2.0".to_string(),
                        id: Value::Null,
                        error: RpcError {
                            code: -32700, // Parse Error
                            message: format!("Parse error: {e}"),
                            data: None,
                        },
                    };
                    let json = serde_json::to_string(&err)?;
                    writeln!(out, "{json}")?;
                    out.flush()?;
                }
            }
        }

        tracing::info!("MCP Server: stdin closed, shutting down");
        Ok(())
    }

    /// 处理单条 JSON-RPC 请求
    ///
    /// 返回 None 表示是通知消息（无需响应）
    fn handle_request(&self, req: &JsonRpcRequest) -> Option<Value> {
        // 通知消息（无 id）不需要回复
        let id = match &req.id {
            Some(id) => id.clone(),
            None => {
                // 处理通知，不回复
                self.handle_notification(req);
                return None;
            }
        };

        let result = match req.method.as_str() {
            // ── MCP 握手 ────────────────────────────────────────────────────
            "initialize" => self.handle_initialize(&req.params),

            // ── 工具列表 ─────────────────────────────────────────────────────
            "tools/list" => self.handle_tools_list(),

            // ── 工具调用 ─────────────────────────────────────────────────────
            "tools/call" => self.handle_tools_call(&req.params),

            // ── 心跳 ─────────────────────────────────────────────────────────
            "ping" => Ok(serde_json::json!({})),

            // ── 资源列表（基础实现）──────────────────────────────────────────
            "resources/list" => Ok(serde_json::json!({ "resources": [] })),

            // ── 提示列表（基础实现）──────────────────────────────────────────
            "prompts/list" => Ok(serde_json::json!({ "prompts": [] })),

            // ── 未知方法 ─────────────────────────────────────────────────────
            unknown => Err(RpcError {
                code: METHOD_NOT_FOUND,
                message: format!(
                    "Method '{}' not found. Supported: initialize, tools/list, tools/call, ping",
                    unknown
                ),
                data: None,
            }),
        };

        Some(match result {
            Ok(value) => serde_json::to_value(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: value,
            })
            .unwrap_or_else(|e| {
                serde_json::to_value(JsonRpcError {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    error: RpcError {
                        code: INTERNAL_ERROR,
                        message: format!("Serialization error: {e}"),
                        data: None,
                    },
                })
                .unwrap()
            }),
            Err(err) => serde_json::to_value(JsonRpcError {
                jsonrpc: "2.0".to_string(),
                id,
                error: err,
            })
            .unwrap_or_else(|_| serde_json::json!({"jsonrpc":"2.0","error":{"code":-32603,"message":"internal error"}})),
        })
    }

    /// 处理通知消息（notifications/initialized 等）
    fn handle_notification(&self, req: &JsonRpcRequest) {
        tracing::debug!("notification: {}", req.method);
    }

    // ── 协议处理器 ────────────────────────────────────────────────────────────

    /// 处理 `initialize` 握手
    ///
    /// 返回服务器能力声明，包含支持的工具列表
    fn handle_initialize(&self, _params: &Value) -> Result<Value, RpcError> {
        let tools = all_tool_defs();
        let tool_list: Vec<Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": self.name,
                "version": self.version,
            },
            "_tools_preview": tool_list,
        }))
    }

    /// 处理 `tools/list` 请求
    fn handle_tools_list(&self) -> Result<Value, RpcError> {
        let tools = all_tool_defs();
        let tool_list: Vec<Value> = tools
            .into_iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema,
                })
            })
            .collect();

        Ok(serde_json::json!({ "tools": tool_list }))
    }

    /// 处理 `tools/call` 请求
    ///
    /// params 格式: `{"name": "move_block", "arguments": {...}}`
    fn handle_tools_call(&self, params: &Value) -> Result<Value, RpcError> {
        // 提取工具名称
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError {
                code: INVALID_PARAMS,
                message: "Missing required parameter: 'name'".to_string(),
                data: None,
            })?;

        // 提取工具参数（arguments 字段）
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        tracing::info!("🔧 Tool call: {} with {:?}", tool_name, arguments);

        // 分发工具调用
        let output = dispatch_tool(tool_name, &arguments, &self.bridge);

        match output {
            ToolOutput::Success(content) => Ok(serde_json::json!({
                "content": content,
                "isError": false,
            })),
            ToolOutput::Error(msg) => {
                tracing::warn!("Tool '{}' failed: {}", tool_name, msg);
                Ok(serde_json::json!({
                    "content": [{ "type": "text", "text": msg }],
                    "isError": true,
                }))
            }
        }
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_bridge::EngineBridge;

    fn server() -> McpServer {
        McpServer::new(EngineBridge::new_with_demo())
    }

    fn make_req(method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: method.to_string(),
            params,
        }
    }

    #[test]
    fn test_initialize() {
        let srv = server();
        let req = make_req(
            "initialize",
            serde_json::json!({ "protocolVersion": "2024-11-05" }),
        );
        let resp = srv.handle_request(&req).unwrap();
        // 应该有 result 字段
        assert!(
            resp.get("result").is_some(),
            "initialize should return result"
        );
        let result = &resp["result"];
        assert_eq!(result["serverInfo"]["name"], "ummerse-mcp");
    }

    #[test]
    fn test_tools_list() {
        let srv = server();
        let req = make_req("tools/list", serde_json::json!({}));
        let resp = srv.handle_request(&req).unwrap();
        let tools = &resp["result"]["tools"];
        assert!(tools.is_array(), "tools should be array");
        assert!(
            tools.as_array().unwrap().len() >= 8,
            "should have at least 8 tools"
        );
    }

    #[test]
    fn test_tools_call_move_block() {
        let srv = server();
        let req = make_req(
            "tools/call",
            serde_json::json!({
                "name": "move_block",
                "arguments": { "name": "MainBlock", "dx": 50.0 }
            }),
        );
        let resp = srv.handle_request(&req).unwrap();
        assert!(resp.get("result").is_some());
        let result = &resp["result"];
        assert_eq!(result["isError"], false);
    }

    #[test]
    fn test_tools_call_get_scene() {
        let srv = server();
        let req = make_req(
            "tools/call",
            serde_json::json!({ "name": "get_scene", "arguments": {} }),
        );
        let resp = srv.handle_request(&req).unwrap();
        assert_eq!(resp["result"]["isError"], false);
    }

    #[test]
    fn test_unknown_method() {
        let srv = server();
        let req = make_req("unknown/method", serde_json::json!({}));
        let resp = srv.handle_request(&req).unwrap();
        assert!(
            resp.get("error").is_some(),
            "unknown method should return error"
        );
        assert_eq!(resp["error"]["code"], METHOD_NOT_FOUND);
    }

    #[test]
    fn test_notification_no_response() {
        let srv = server();
        // 通知（无 id）
        let notif = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "notifications/initialized".to_string(),
            params: serde_json::json!({}),
        };
        let resp = srv.handle_request(&notif);
        assert!(resp.is_none(), "notifications should not return response");
    }

    #[test]
    fn test_ping() {
        let srv = server();
        let req = make_req("ping", serde_json::json!({}));
        let resp = srv.handle_request(&req).unwrap();
        assert!(resp.get("result").is_some());
    }
}
