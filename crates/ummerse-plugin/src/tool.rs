//! 引擎内置工具定义
//!
//! 类似 Cline 的工具调用系统，AI 助手或插件可调用这些工具
//! 来操作引擎和编辑器。

use crate::{
    protocol::{ToolCall, ToolResult, ToolResultContent},
    PluginError, Result,
};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// 引擎工具 trait - 所有工具实现此接口
#[async_trait]
pub trait EngineTool: Send + Sync + 'static {
    /// 工具名称（snake_case，唯一）
    fn name(&self) -> &str;

    /// 工具描述（供 AI 使用）
    fn description(&self) -> &str;

    /// 参数 Schema（JSON Schema 格式）
    fn parameters_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }

    /// 是否需要用户审批确认
    fn requires_approval(&self) -> bool {
        false
    }

    /// 执行工具调用
    async fn execute(&self, call: &ToolCall) -> ToolResult;
}

// ── 工具注册表 ────────────────────────────────────────────────────────────────

/// 工具注册表 - 管理所有可用工具
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn EngineTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        // 注册内置工具
        registry.register(Arc::new(ReadFileTool));
        registry.register(Arc::new(WriteFileTool));
        registry.register(Arc::new(ListFilesTool));
        registry.register(Arc::new(SearchFilesTool));
        registry.register(Arc::new(GetSceneTreeTool));
        registry.register(Arc::new(CreateNodeTool));
        registry.register(Arc::new(DeleteNodeTool));
        registry.register(Arc::new(SetNodePropertyTool));
        registry.register(Arc::new(ExecuteCommandTool));
        registry
    }

    /// 注册工具
    pub fn register(&mut self, tool: Arc<dyn EngineTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<Arc<dyn EngineTool>> {
        self.tools.get(name).cloned()
    }

    /// 执行工具调用
    pub async fn dispatch(&self, call: ToolCall) -> ToolResult {
        match self.tools.get(&call.name) {
            Some(tool) => tool.execute(&call).await,
            None => ToolResult::error(
                &call.id,
                &call.name,
                format!("Tool '{}' not found", call.name),
            ),
        }
    }

    /// 所有工具名称列表
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(String::as_str).collect()
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── 内置工具实现 ──────────────────────────────────────────────────────────────

/// 读取文件工具
pub struct ReadFileTool;

#[async_trait]
impl EngineTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file at the specified path. Returns the file content as text."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path of the file to read (relative to project root)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = match call.parameters.get("path").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => return ToolResult::error(&call.id, self.name(), "Missing 'path' parameter"),
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => ToolResult::ok_text(&call.id, self.name(), content),
            Err(e) => ToolResult::error(
                &call.id,
                self.name(),
                format!("Failed to read '{}': {}", path, e),
            ),
        }
    }
}

/// 写入文件工具
pub struct WriteFileTool;

#[async_trait]
impl EngineTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file at the specified path. Creates the file if it does not exist."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to write to"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write"
                }
            },
            "required": ["path", "content"]
        })
    }

    fn requires_approval(&self) -> bool {
        true
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = match call.parameters.get("path").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => return ToolResult::error(&call.id, self.name(), "Missing 'path' parameter"),
        };
        let content = match call.parameters.get("content").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult::error(&call.id, self.name(), "Missing 'content' parameter"),
        };

        // 创建父目录（如果不存在）
        if let Some(parent) = std::path::Path::new(&path).parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return ToolResult::error(
                    &call.id,
                    self.name(),
                    format!("Failed to create directory '{}': {}", parent.display(), e),
                );
            }
        }

        match std::fs::write(&path, &content) {
            Ok(_) => ToolResult::ok_text(
                &call.id,
                self.name(),
                format!("Successfully wrote {} bytes to '{}'", content.len(), path),
            ),
            Err(e) => ToolResult::error(
                &call.id,
                self.name(),
                format!("Failed to write '{}': {}", path, e),
            ),
        }
    }
}

/// 列出文件工具
pub struct ListFilesTool;

#[async_trait]
impl EngineTool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories in the specified path. Use recursive=true to list all files recursively."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list recursively",
                    "default": false
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = match call.parameters.get("path").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => return ToolResult::error(&call.id, self.name(), "Missing 'path' parameter"),
        };
        let recursive = call
            .parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut entries = Vec::new();

        if recursive {
            let walker = walkdir::WalkDir::new(&path)
                .min_depth(1)
                .max_depth(10)
                .into_iter();
            for entry in walker
                .filter_map(|e: std::result::Result<walkdir::DirEntry, walkdir::Error>| e.ok())
            {
                let path_str = entry.path().display().to_string();
                let is_dir = entry.file_type().is_dir();
                entries.push(format!("{}{}", path_str, if is_dir { "/" } else { "" }));
            }
        } else {
            match std::fs::read_dir(&path) {
                Ok(dir) => {
                    for entry in dir.filter_map(|e: std::io::Result<std::fs::DirEntry>| e.ok()) {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        entries.push(format!("{}{}", name, if is_dir { "/" } else { "" }));
                    }
                    entries.sort();
                }
                Err(e) => {
                    return ToolResult::error(
                        &call.id,
                        self.name(),
                        format!("Failed to list '{}': {}", path, e),
                    )
                }
            }
        }

        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({ "path": path, "entries": entries }),
            }],
        )
    }
}

/// 搜索文件工具
pub struct SearchFilesTool;

#[async_trait]
impl EngineTool for SearchFilesTool {
    fn name(&self) -> &str {
        "search_files"
    }

    fn description(&self) -> &str {
        "Search for text patterns in files within a directory."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Directory to search in" },
                "pattern": { "type": "string", "description": "Text pattern to search for" },
                "file_pattern": { "type": "string", "description": "File glob pattern (e.g., '*.rs')" }
            },
            "required": ["path", "pattern"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = match call.parameters.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call.id, self.name(), "Missing 'path' parameter"),
        };
        let pattern = match call.parameters.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error(&call.id, self.name(), "Missing 'pattern' parameter"),
        };

        let mut matches = Vec::new();
        let walker = walkdir::WalkDir::new(path).into_iter();

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                let file_path = entry.path().display().to_string();
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        matches.push(serde_json::json!({
                            "file": file_path,
                            "line": line_num + 1,
                            "content": line.trim()
                        }));
                    }
                }
            }
        }

        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({ "matches": matches, "count": matches.len() }),
            }],
        )
    }
}

/// 获取场景树工具
pub struct GetSceneTreeTool;

#[async_trait]
impl EngineTool for GetSceneTreeTool {
    fn name(&self) -> &str {
        "get_scene_tree"
    }
    fn description(&self) -> &str {
        "Get the current scene tree structure as JSON."
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        // 返回占位数据（完整实现需要访问运行时场景树）
        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({
                    "root": { "name": "Root", "type": "Node", "children": [] }
                }),
            }],
        )
    }
}

/// 创建节点工具
pub struct CreateNodeTool;

#[async_trait]
impl EngineTool for CreateNodeTool {
    fn name(&self) -> &str {
        "create_node"
    }

    fn description(&self) -> &str {
        "Create a new node in the scene tree."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "type": { "type": "string", "description": "Node type (e.g., Node2d, Node3d, Sprite2d)" },
                "parent": { "type": "string", "description": "Parent node path" }
            },
            "required": ["name", "type"]
        })
    }

    fn requires_approval(&self) -> bool {
        true
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let name = call
            .parameters
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Node");
        let node_type = call
            .parameters
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("Node");
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!(
                "Created {} node '{}' (stub - scene tree integration pending)",
                node_type, name
            ),
        )
    }
}

/// 删除节点工具
pub struct DeleteNodeTool;

#[async_trait]
impl EngineTool for DeleteNodeTool {
    fn name(&self) -> &str {
        "delete_node"
    }
    fn description(&self) -> &str {
        "Delete a node from the scene tree by path."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Node path to delete" }
            },
            "required": ["path"]
        })
    }

    fn requires_approval(&self) -> bool {
        true
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = call
            .parameters
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!("Deleted node at '{}' (stub)", path),
        )
    }
}

/// 设置节点属性工具
pub struct SetNodePropertyTool;

#[async_trait]
impl EngineTool for SetNodePropertyTool {
    fn name(&self) -> &str {
        "set_node_property"
    }
    fn description(&self) -> &str {
        "Set a property value on a scene node."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "property": { "type": "string" },
                "value": {}
            },
            "required": ["path", "property", "value"]
        })
    }

    fn requires_approval(&self) -> bool {
        true
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = call
            .parameters
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let property = call
            .parameters
            .get("property")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let value = call.parameters.get("value").cloned().unwrap_or(Value::Null);
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!("Set {}.{} = {} (stub)", path, property, value),
        )
    }
}

/// 执行编辑器命令工具
pub struct ExecuteCommandTool;

#[async_trait]
impl EngineTool for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }
    fn description(&self) -> &str {
        "Execute an editor command by its ID."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Command ID to execute" },
                "args": { "type": "object", "description": "Optional command arguments" }
            },
            "required": ["command"]
        })
    }

    fn requires_approval(&self) -> bool {
        true
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let command = call
            .parameters
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!("Executed command '{}' (stub)", command),
        )
    }
}
