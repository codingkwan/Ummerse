//! MCP 工具定义
//!
//! 将引擎功能暴露为 MCP 工具，供 Cline 等 AI Agent 调用。
//!
//! ## 可用工具
//! - `move_block`     - 相对移动实体（增量）
//! - `set_position`   - 设置实体绝对位置
//! - `spawn_entity`   - 在场景中生成新实体
//! - `despawn_entity` - 删除场景实体
//! - `get_scene`      - 获取完整场景快照
//! - `get_entity`     - 获取单个实体详情
//! - `set_property`   - 设置实体属性（visible/rotation/自定义）
//! - `list_entities`  - 列出所有实体（摘要）

use crate::engine_bridge::{EngineBridge, EntityKind};
use serde_json::Value;

/// 工具定义（供 MCP initialize 响应使用）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// 返回所有引擎工具的定义列表
pub fn all_tool_defs() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "move_block".to_string(),
            description: "相对移动场景中的实体（增量偏移）。例如：让 MainBlock 向右移动 50 像素"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "实体名称或 ID（如 'MainBlock'）"
                    },
                    "dx": {
                        "type": "number",
                        "description": "X 轴偏移量（像素，正=右，负=左）",
                        "default": 0
                    },
                    "dy": {
                        "type": "number",
                        "description": "Y 轴偏移量（像素，正=上，负=下）",
                        "default": 0
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDef {
            name: "set_position".to_string(),
            description: "设置实体的绝对位置坐标".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "实体名称或 ID"
                    },
                    "x": { "type": "number", "description": "X 坐标（像素）" },
                    "y": { "type": "number", "description": "Y 坐标（像素）" }
                },
                "required": ["name", "x", "y"]
            }),
        },
        ToolDef {
            name: "spawn_entity".to_string(),
            description: "在场景中生成新实体".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "新实体的名称"
                    },
                    "kind": {
                        "type": "string",
                        "description": "实体类型",
                        "enum": ["block", "circle", "player", "camera"],
                        "default": "block"
                    },
                    "x": { "type": "number", "description": "初始 X 坐标", "default": 0 },
                    "y": { "type": "number", "description": "初始 Y 坐标", "default": 0 }
                },
                "required": ["name"]
            }),
        },
        ToolDef {
            name: "despawn_entity".to_string(),
            description: "从场景中删除实体".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "要删除的实体名称或 ID"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDef {
            name: "get_scene".to_string(),
            description: "获取当前场景的完整快照（所有实体、位置等信息）".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDef {
            name: "get_entity".to_string(),
            description: "获取单个实体的详细信息（位置、属性等）".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "实体名称或 ID"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDef {
            name: "set_property".to_string(),
            description: "设置实体的属性（如 visible、rotation、scale_x、scale_y 或自定义属性）"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "实体名称或 ID"
                    },
                    "property": {
                        "type": "string",
                        "description": "属性名（visible/rotation/scale_x/scale_y/自定义）"
                    },
                    "value": {
                        "description": "属性值"
                    }
                },
                "required": ["name", "property", "value"]
            }),
        },
        ToolDef {
            name: "list_entities".to_string(),
            description: "列出场景中所有实体的摘要信息（名称、类型、位置）".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

// ── 工具分发 ──────────────────────────────────────────────────────────────────

/// 工具执行结果
pub enum ToolOutput {
    /// 成功，返回文本内容列表
    Success(Vec<Value>),
    /// 失败，返回错误信息
    Error(String),
}

impl ToolOutput {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Success(vec![
            serde_json::json!({ "type": "text", "text": s.into() }),
        ])
    }

    pub fn json(v: Value) -> Self {
        Self::Success(vec![
            serde_json::json!({ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }),
        ])
    }

    pub fn err(s: impl Into<String>) -> Self {
        Self::Error(s.into())
    }
}

/// 执行工具调用
///
/// # 参数
/// - `tool_name`: 工具名称
/// - `params`: 工具参数（JSON 对象）
/// - `bridge`: 引擎桥接句柄
pub fn dispatch_tool(tool_name: &str, params: &Value, bridge: &EngineBridge) -> ToolOutput {
    match tool_name {
        "move_block" => {
            let name = match params.get("name").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'name'"),
            };
            let dx = params.get("dx").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let dy = params.get("dy").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

            match bridge.move_entity(name, dx, dy) {
                Ok((entity_name, new_pos)) => ToolOutput::text(format!(
                    "✅ 已移动 '{}' → 新位置: ({:.1}, {:.1})  [偏移: dx={:.1}, dy={:.1}]",
                    entity_name, new_pos.x, new_pos.y, dx, dy
                )),
                Err(e) => ToolOutput::err(e),
            }
        }

        "set_position" => {
            let name = match params.get("name").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'name'"),
            };
            let x = match params.get("x").and_then(|v| v.as_f64()) {
                Some(v) => v as f32,
                None => return ToolOutput::err("缺少必需参数: 'x'"),
            };
            let y = match params.get("y").and_then(|v| v.as_f64()) {
                Some(v) => v as f32,
                None => return ToolOutput::err("缺少必需参数: 'y'"),
            };

            match bridge.set_position(name, x, y) {
                Ok(msg) => ToolOutput::text(format!("✅ {msg}")),
                Err(e) => ToolOutput::err(e),
            }
        }

        "spawn_entity" => {
            let name = match params.get("name").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'name'"),
            };
            let kind = match params.get("kind").and_then(|v| v.as_str()).unwrap_or("block") {
                "circle" => EntityKind::Circle,
                "player" => EntityKind::Player,
                "camera" => EntityKind::Camera,
                other => {
                    if other == "block" {
                        EntityKind::Block
                    } else {
                        EntityKind::Custom(other.to_string())
                    }
                }
            };
            let x = params.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let y = params.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

            let id = bridge.spawn_entity(name, kind, x, y);
            ToolOutput::text(format!(
                "✅ 已生成实体 '{}' 在 ({:.1}, {:.1})，ID: {}",
                name, x, y, id
            ))
        }

        "despawn_entity" => {
            let name = match params.get("name").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'name'"),
            };

            match bridge.despawn_entity(name) {
                Ok(msg) => ToolOutput::text(format!("✅ {msg}")),
                Err(e) => ToolOutput::err(e),
            }
        }

        "get_scene" => {
            let snapshot = bridge.get_scene_snapshot();
            ToolOutput::json(snapshot)
        }

        "get_entity" => {
            let name = match params.get("name").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'name'"),
            };

            match bridge.get_entity(name) {
                Some(entity) => ToolOutput::json(entity),
                None => ToolOutput::err(format!("实体 '{}' 不存在", name)),
            }
        }

        "set_property" => {
            let name = match params.get("name").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'name'"),
            };
            let property = match params.get("property").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => return ToolOutput::err("缺少必需参数: 'property'"),
            };
            let value = match params.get("value") {
                Some(v) => v.clone(),
                None => return ToolOutput::err("缺少必需参数: 'value'"),
            };

            match bridge.set_property(name, property, value) {
                Ok(msg) => ToolOutput::text(format!("✅ {msg}")),
                Err(e) => ToolOutput::err(e),
            }
        }

        "list_entities" => {
            let snapshot = bridge.get_scene_snapshot();
            let entities = snapshot["entities"].clone();
            ToolOutput::json(serde_json::json!({
                "scene": snapshot["scene_name"],
                "frame": snapshot["frame_count"],
                "count": snapshot["entity_count"],
                "entities": entities,
            }))
        }

        unknown => ToolOutput::err(format!(
            "未知工具: '{}'. 可用工具: move_block, set_position, spawn_entity, despawn_entity, get_scene, get_entity, set_property, list_entities",
            unknown
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_bridge::EngineBridge;

    fn bridge() -> EngineBridge {
        EngineBridge::new_with_demo()
    }

    #[test]
    fn test_move_block() {
        let b = bridge();
        let out = dispatch_tool(
            "move_block",
            &serde_json::json!({ "name": "MainBlock", "dx": 50.0 }),
            &b,
        );
        assert!(matches!(out, ToolOutput::Success(_)));
    }

    #[test]
    fn test_get_scene() {
        let b = bridge();
        let out = dispatch_tool("get_scene", &serde_json::json!({}), &b);
        assert!(matches!(out, ToolOutput::Success(_)));
    }

    #[test]
    fn test_spawn_and_despawn() {
        let b = bridge();
        // 生成
        let out = dispatch_tool(
            "spawn_entity",
            &serde_json::json!({ "name": "TestBox", "kind": "block", "x": 100.0, "y": 200.0 }),
            &b,
        );
        assert!(matches!(out, ToolOutput::Success(_)));

        // 删除
        let out = dispatch_tool(
            "despawn_entity",
            &serde_json::json!({ "name": "TestBox" }),
            &b,
        );
        assert!(matches!(out, ToolOutput::Success(_)));
    }

    #[test]
    fn test_all_tool_defs() {
        let defs = all_tool_defs();
        assert_eq!(defs.len(), 8, "Should have exactly 8 tools");
        for def in &defs {
            assert!(!def.name.is_empty(), "Tool name should not be empty");
            assert!(
                !def.description.is_empty(),
                "Tool description should not be empty"
            );
        }
    }
}
