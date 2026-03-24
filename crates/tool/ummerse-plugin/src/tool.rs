//! 引擎内置工具定义
//!
//! 类似 Cline/MCP 的工具调用系统，AI 助手或插件可调用这些工具
//! 来操作引擎和编辑器。
//!
//! ## 设计原则
//! - 每个工具实现 `EngineTool` trait
//! - 工具通过 `ToolRegistry` 统一管理和分发
//! - 需要写操作的工具标记 `requires_approval = true`
//! - 参数通过 JSON Schema 描述，供 AI 解析

use crate::protocol::{ToolCall, ToolResult, ToolResultContent};
use async_trait::async_trait;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

// ── EngineTool trait ──────────────────────────────────────────────────────────

/// 引擎工具 trait - 所有工具实现此接口
#[async_trait]
pub trait EngineTool: Send + Sync + 'static {
    /// 工具名称（snake_case，全局唯一）
    fn name(&self) -> &str;

    /// 工具描述（供 AI 助手理解工具用途）
    fn description(&self) -> &str;

    /// 参数 Schema（JSON Schema 格式，供 AI 生成调用参数）
    fn parameters_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }

    /// 是否需要用户审批确认（写操作应设为 true）
    fn requires_approval(&self) -> bool {
        false
    }

    /// 工具分类（用于 UI 分组展示）
    fn category(&self) -> ToolCategory {
        ToolCategory::General
    }

    /// 执行工具调用，返回结果
    async fn execute(&self, call: &ToolCall) -> ToolResult;
}

/// 工具分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    General,
    FileSystem,
    SceneTree,
    Physics,
    Rendering,
    Script,
    Editor,
    Build,
}

impl ToolCategory {
    pub fn display_name(&self) -> &str {
        match self {
            Self::General => "General",
            Self::FileSystem => "File System",
            Self::SceneTree => "Scene Tree",
            Self::Physics => "Physics",
            Self::Rendering => "Rendering",
            Self::Script => "Script",
            Self::Editor => "Editor",
            Self::Build => "Build",
        }
    }
}

// ── 工具注册表 ────────────────────────────────────────────────────────────────

/// 工具注册表 - 管理所有可用工具
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn EngineTool>>,
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tool_count", &self.tools.len())
            .field("tools", &self.tool_names())
            .finish()
    }
}

impl ToolRegistry {
    /// 创建并注册所有内置工具
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        // ── 文件系统工具 ──
        registry.register(Arc::new(ReadFileTool));
        registry.register(Arc::new(WriteFileTool));
        registry.register(Arc::new(ListFilesTool));
        registry.register(Arc::new(SearchFilesTool));
        registry.register(Arc::new(DeleteFileTool));
        // ── 场景树工具 ──
        registry.register(Arc::new(GetSceneTreeTool));
        registry.register(Arc::new(CreateNodeTool));
        registry.register(Arc::new(DeleteNodeTool));
        registry.register(Arc::new(SetNodePropertyTool));
        registry.register(Arc::new(GetNodePropertyTool));
        // ── 编辑器工具 ──
        registry.register(Arc::new(ExecuteCommandTool));
        registry.register(Arc::new(GetProjectInfoTool));
        registry
    }

    /// 注册自定义工具
    pub fn register(&mut self, tool: Arc<dyn EngineTool>) {
        let name = tool.name().to_string();
        tracing::debug!("Registered tool: {name}");
        self.tools.insert(name, tool);
    }

    /// 获取工具（按名称）
    pub fn get(&self, name: &str) -> Option<Arc<dyn EngineTool>> {
        self.tools.get(name).cloned()
    }

    /// 分发工具调用（主入口）
    pub async fn dispatch(&self, call: ToolCall) -> ToolResult {
        match self.tools.get(&call.name) {
            Some(tool) => {
                tracing::debug!(tool = %call.name, id = %call.id, "Dispatching tool call");
                tool.execute(&call).await
            }
            None => ToolResult::error(
                &call.id,
                &call.name,
                format!(
                    "Tool '{}' not found. Available: {:?}",
                    call.name,
                    self.tool_names()
                ),
            ),
        }
    }

    /// 所有工具名称列表（已排序）
    pub fn tool_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tools.keys().map(String::as_str).collect();
        names.sort();
        names
    }

    /// 获取所有工具的 Schema（供 AI 使用）
    pub fn all_schemas(&self) -> Vec<Value> {
        let mut schemas: Vec<_> = self
            .tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "parameters": t.parameters_schema(),
                    "requires_approval": t.requires_approval(),
                })
            })
            .collect();
        schemas.sort_by_key(|s| s["name"].as_str().unwrap_or("").to_string());
        schemas
    }

    /// 工具数量
    #[inline]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// 是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── 辅助宏：提取必需参数 ──────────────────────────────────────────────────────

macro_rules! require_str {
    ($call:expr, $key:expr) => {
        match $call.parameters.get($key).and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return ToolResult::error(
                    &$call.id,
                    $call.name.as_str(),
                    format!("Missing required parameter: '{}'", $key),
                )
            }
        }
    };
}

// ── 文件系统工具 ──────────────────────────────────────────────────────────────

/// 读取文件工具
#[derive(Debug)]
pub struct ReadFileTool;

#[async_trait]
impl EngineTool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read the contents of a file at the specified path. Returns the file content as text."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path to read (absolute or relative to project root)"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        match std::fs::read_to_string(path) {
            Ok(content) => ToolResult::success(
                &call.id,
                self.name(),
                vec![
                    ToolResultContent::Text {
                        text: content.clone(),
                    },
                    ToolResultContent::Json {
                        data: serde_json::json!({
                            "path": path,
                            "size": content.len(),
                            "lines": content.lines().count(),
                        }),
                    },
                ],
            ),
            Err(e) => {
                ToolResult::error(&call.id, self.name(), format!("Cannot read '{path}': {e}"))
            }
        }
    }
}

/// 写入文件工具（需要审批）
#[derive(Debug)]
pub struct WriteFileTool;

#[async_trait]
impl EngineTool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "Write content to a file. Creates parent directories if they don't exist."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":    { "type": "string", "description": "Destination file path" },
                "content": { "type": "string", "description": "File content to write" }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        let content = require_str!(call, "content");

        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return ToolResult::error(
                        &call.id,
                        self.name(),
                        format!("Cannot create directory '{}': {e}", parent.display()),
                    );
                }
            }
        }

        match std::fs::write(path, content) {
            Ok(_) => ToolResult::ok_text(
                &call.id,
                self.name(),
                format!("Wrote {} bytes to '{path}'", content.len()),
            ),
            Err(e) => {
                ToolResult::error(&call.id, self.name(), format!("Cannot write '{path}': {e}"))
            }
        }
    }
}

/// 列出目录工具
#[derive(Debug)]
pub struct ListFilesTool;

#[async_trait]
impl EngineTool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    fn description(&self) -> &str {
        "List files and directories at the given path. Set recursive=true for deep listing."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":      { "type": "string", "description": "Directory path" },
                "recursive": { "type": "boolean", "description": "Recurse into subdirectories", "default": false }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        let recursive = call
            .parameters
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut entries: Vec<Value> = Vec::new();

        if recursive {
            for entry in walkdir::WalkDir::new(path)
                .min_depth(1)
                .max_depth(20)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let is_dir = entry.file_type().is_dir();
                let size = if is_dir {
                    None
                } else {
                    entry.metadata().ok().map(|m| m.len())
                };
                entries.push(serde_json::json!({
                    "path": entry.path().display().to_string(),
                    "is_dir": is_dir,
                    "size": size,
                }));
            }
        } else {
            match std::fs::read_dir(path) {
                Ok(dir) => {
                    let mut raw: Vec<_> = dir.filter_map(|e| e.ok()).collect();
                    raw.sort_by_key(|e| e.file_name());
                    for entry in raw {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let size = if is_dir {
                            None
                        } else {
                            entry.metadata().ok().map(|m| m.len())
                        };
                        entries.push(serde_json::json!({
                            "name": name,
                            "is_dir": is_dir,
                            "size": size,
                        }));
                    }
                }
                Err(e) => {
                    return ToolResult::error(
                        &call.id,
                        self.name(),
                        format!("Cannot list '{path}': {e}"),
                    );
                }
            }
        }

        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({ "path": path, "recursive": recursive, "entries": entries }),
            }],
        )
    }
}

/// 搜索文件内容工具
#[derive(Debug)]
pub struct SearchFilesTool;

#[async_trait]
impl EngineTool for SearchFilesTool {
    fn name(&self) -> &str {
        "search_files"
    }
    fn description(&self) -> &str {
        "Search for text patterns in files. Returns matching lines with context."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":         { "type": "string",  "description": "Directory to search in" },
                "pattern":      { "type": "string",  "description": "Text pattern to find" },
                "case_sensitive": { "type": "boolean", "description": "Case sensitive search", "default": false },
                "max_results":  { "type": "integer", "description": "Max results to return", "default": 50 }
            },
            "required": ["path", "pattern"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        let pattern = require_str!(call, "pattern");
        let case_sensitive = call
            .parameters
            .get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_results = call
            .parameters
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let pattern_cmp = if case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let mut matches: Vec<Value> = Vec::new();

        'outer: for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() {
                continue;
            }
            // 跳过二进制文件（通过扩展名过滤）
            let ext = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if matches!(
                ext,
                "png" | "jpg" | "jpeg" | "gif" | "bmp" | "wasm" | "bin" | "dll" | "exe"
            ) {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                for (line_num, line) in content.lines().enumerate() {
                    let cmp_line = if case_sensitive {
                        line.to_string()
                    } else {
                        line.to_lowercase()
                    };
                    if cmp_line.contains(&pattern_cmp) {
                        matches.push(serde_json::json!({
                            "file": entry.path().display().to_string(),
                            "line": line_num + 1,
                            "content": line.trim(),
                        }));
                        if matches.len() >= max_results {
                            break 'outer;
                        }
                    }
                }
            }
        }

        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({
                    "pattern": pattern,
                    "count": matches.len(),
                    "matches": matches,
                }),
            }],
        )
    }
}

/// 删除文件工具（需要审批）
#[derive(Debug)]
pub struct DeleteFileTool;

#[async_trait]
impl EngineTool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }
    fn description(&self) -> &str {
        "Delete a file or empty directory. Use with caution."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::FileSystem
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path to delete" }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        let p = std::path::Path::new(path);

        let result = if p.is_dir() {
            std::fs::remove_dir(p)
        } else {
            std::fs::remove_file(p)
        };

        match result {
            Ok(_) => ToolResult::ok_text(&call.id, self.name(), format!("Deleted '{path}'")),
            Err(e) => ToolResult::error(
                &call.id,
                self.name(),
                format!("Cannot delete '{path}': {e}"),
            ),
        }
    }
}

// ── 场景树工具 ────────────────────────────────────────────────────────────────

/// 获取场景树结构工具
#[derive(Debug)]
pub struct GetSceneTreeTool;

#[async_trait]
impl EngineTool for GetSceneTreeTool {
    fn name(&self) -> &str {
        "get_scene_tree"
    }
    fn description(&self) -> &str {
        "Get the current scene tree structure as a JSON hierarchy."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::SceneTree
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        // 占位实现 - 完整版需通过 Bevy ECS 或共享的 SceneTree 访问
        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({
                    "note": "Scene tree access requires runtime integration",
                    "root": {
                        "name": "Root",
                        "type": "Node",
                        "id": "00000000-0000-0000-0000-000000000000",
                        "children": []
                    }
                }),
            }],
        )
    }
}

/// 创建节点工具（需要审批）
#[derive(Debug)]
pub struct CreateNodeTool;

#[async_trait]
impl EngineTool for CreateNodeTool {
    fn name(&self) -> &str {
        "create_node"
    }
    fn description(&self) -> &str {
        "Create a new node in the scene tree at the specified parent path."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::SceneTree
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name":       { "type": "string", "description": "Node name" },
                "node_type":  { "type": "string", "description": "Node type (e.g., Node2d, Sprite2d, Camera3d)" },
                "parent":     { "type": "string", "description": "Parent node path (default: scene root)" }
            },
            "required": ["name", "node_type"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let name = require_str!(call, "name");
        let node_type = require_str!(call, "node_type");
        let parent = call
            .parameters
            .get("parent")
            .and_then(|v| v.as_str())
            .unwrap_or("/Root");
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!(
                "Created {node_type} node '{name}' under '{parent}' (pending runtime integration)"
            ),
        )
    }
}

/// 删除节点工具（需要审批）
#[derive(Debug)]
pub struct DeleteNodeTool;

#[async_trait]
impl EngineTool for DeleteNodeTool {
    fn name(&self) -> &str {
        "delete_node"
    }
    fn description(&self) -> &str {
        "Delete a node and all its children from the scene tree."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::SceneTree
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Node path to delete (e.g., /Root/Player)" }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!("Deleted node at '{path}' (pending runtime integration)"),
        )
    }
}

/// 设置节点属性工具（需要审批）
#[derive(Debug)]
pub struct SetNodePropertyTool;

#[async_trait]
impl EngineTool for SetNodePropertyTool {
    fn name(&self) -> &str {
        "set_node_property"
    }
    fn description(&self) -> &str {
        "Set a property value on a scene node. Supports position, rotation, scale, and custom properties."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::SceneTree
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":     { "type": "string", "description": "Node path" },
                "property": { "type": "string", "description": "Property name (e.g., position, rotation, scale)" },
                "value":    { "description": "New property value" }
            },
            "required": ["path", "property", "value"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        let property = require_str!(call, "property");
        let value = call.parameters.get("value").cloned().unwrap_or(Value::Null);
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!("Set {path}.{property} = {value} (pending runtime integration)"),
        )
    }
}

/// 获取节点属性工具
#[derive(Debug)]
pub struct GetNodePropertyTool;

#[async_trait]
impl EngineTool for GetNodePropertyTool {
    fn name(&self) -> &str {
        "get_node_property"
    }
    fn description(&self) -> &str {
        "Get the current value of a property on a scene node."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::SceneTree
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path":     { "type": "string", "description": "Node path" },
                "property": { "type": "string", "description": "Property name" }
            },
            "required": ["path", "property"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let path = require_str!(call, "path");
        let property = require_str!(call, "property");
        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({
                    "path": path,
                    "property": property,
                    "value": null,
                    "note": "Pending runtime integration"
                }),
            }],
        )
    }
}

// ── 编辑器工具 ────────────────────────────────────────────────────────────────

/// 执行编辑器命令工具（需要审批）
#[derive(Debug)]
pub struct ExecuteCommandTool;

#[async_trait]
impl EngineTool for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }
    fn description(&self) -> &str {
        "Execute an editor command by its ID. Use get_project_info to see available commands."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Editor
    }
    fn requires_approval(&self) -> bool {
        true
    }
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Command ID (e.g., file.save, scene.run)" },
                "args":    { "type": "object", "description": "Optional command arguments" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        let command = require_str!(call, "command");
        ToolResult::ok_text(
            &call.id,
            self.name(),
            format!("Executed command '{command}' (pending editor integration)"),
        )
    }
}

/// 获取项目信息工具
#[derive(Debug)]
pub struct GetProjectInfoTool;

#[async_trait]
impl EngineTool for GetProjectInfoTool {
    fn name(&self) -> &str {
        "get_project_info"
    }
    fn description(&self) -> &str {
        "Get current project metadata, available commands, and engine version."
    }
    fn category(&self) -> ToolCategory {
        ToolCategory::Editor
    }

    async fn execute(&self, call: &ToolCall) -> ToolResult {
        ToolResult::success(
            &call.id,
            self.name(),
            vec![ToolResultContent::Json {
                data: serde_json::json!({
                    "engine": "Ummerse",
                    "version": env!("CARGO_PKG_VERSION"),
                    "available_node_types": [
                        "Node", "Node2d", "Node3d",
                        "Sprite2d", "AnimatedSprite2d", "Camera2d", "TileMap",
                        "MeshInstance3d", "Camera3d", "DirectionalLight3d", "PointLight3d",
                        "RigidBody2d", "StaticBody2d", "CharacterBody2d", "Area2d",
                        "RigidBody3d", "StaticBody3d", "CharacterBody3d",
                        "AudioStreamPlayer", "AnimationPlayer", "ScriptNode"
                    ],
                    "available_commands": [
                        "file.save", "file.save_all", "file.new_project", "file.open_project",
                        "scene.new_scene", "scene.run", "scene.stop",
                        "build.export_web", "build.export_desktop",
                        "view.toggle_ai", "view.toggle_scene_tree"
                    ]
                }),
            }],
        )
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_registry_has_all_tools() {
        let registry = ToolRegistry::new();
        assert!(
            registry.len() >= 12,
            "Expected at least 12 tools, got {}",
            registry.len()
        );
        // 验证关键工具已注册
        assert!(registry.get("read_file").is_some());
        assert!(registry.get("write_file").is_some());
        assert!(registry.get("get_scene_tree").is_some());
        assert!(registry.get("create_node").is_some());
        assert!(registry.get("execute_command").is_some());
    }

    #[test]
    fn test_read_file_missing_param() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let registry = ToolRegistry::new();
            let call = ToolCall::new("read_file", serde_json::json!({}));
            let result = registry.dispatch(call).await;
            assert!(result.is_error, "Should fail on missing 'path' parameter");
        });
    }

    #[test]
    fn test_unknown_tool() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let registry = ToolRegistry::new();
            let call = ToolCall::new("nonexistent_tool", serde_json::json!({}));
            let result = registry.dispatch(call).await;
            assert!(result.is_error);
            assert!(result.text().unwrap_or("").contains("not found"));
        });
    }

    #[test]
    fn test_all_schemas() {
        let registry = ToolRegistry::new();
        let schemas = registry.all_schemas();
        assert_eq!(schemas.len(), registry.len());
        // 每个 schema 都必须有 name 和 description
        for schema in &schemas {
            assert!(schema["name"].is_string(), "Schema missing 'name'");
            assert!(
                schema["description"].is_string(),
                "Schema missing 'description'"
            );
        }
    }

    #[test]
    fn test_write_requires_approval() {
        let tool = WriteFileTool;
        assert!(tool.requires_approval());

        let read_tool = ReadFileTool;
        assert!(!read_tool.requires_approval());
    }
}
