//! 引擎桥接层 - 在 MCP Server 与游戏引擎状态之间建立通信
//!
//! 使用共享的 `Arc<Mutex<SceneState>>` 来持有场景状态，
//! MCP 工具通过此 bridge 读写引擎状态。
//!
//! 在真实接入 Bevy 时，可将 SceneState 替换为跨线程 channel 或
//! Bevy Resource，但接口保持不变。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ── 场景实体数据模型 ──────────────────────────────────────────────────────────

/// 2D 位置
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Block,
    Circle,
    Player,
    Camera,
    Custom(String),
}

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Block => write!(f, "Block"),
            EntityKind::Circle => write!(f, "Circle"),
            EntityKind::Player => write!(f, "Player"),
            EntityKind::Camera => write!(f, "Camera"),
            EntityKind::Custom(s) => write!(f, "Custom({s})"),
        }
    }
}

/// 场景中的一个实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体唯一 ID
    pub id: String,
    /// 实体名称
    pub name: String,
    /// 实体类型
    pub kind: EntityKind,
    /// 2D 位置（像素坐标）
    pub position: Vec2,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// 缩放
    pub scale: Vec2,
    /// 是否可见
    pub visible: bool,
    /// 自定义属性（key-value）
    pub properties: HashMap<String, serde_json::Value>,
}

impl Entity {
    /// 创建新实体（默认在原点）
    pub fn new(name: impl Into<String>, kind: EntityKind) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            kind,
            position: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            visible: true,
            properties: HashMap::new(),
        }
    }

    /// 设置位置
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.position = Vec2::new(x, y);
        self
    }
}

// ── 场景状态 ──────────────────────────────────────────────────────────────────

/// 当前场景的完整状态（线程安全）
#[derive(Debug, Default)]
pub struct SceneState {
    /// 所有实体，按 ID 索引
    pub entities: HashMap<String, Entity>,
    /// 场景名称
    pub scene_name: String,
    /// 引擎运行帧数
    pub frame_count: u64,
}

impl SceneState {
    /// 新建场景状态，并创建一些演示实体
    pub fn with_demo() -> Self {
        let mut state = SceneState {
            entities: HashMap::new(),
            scene_name: "Demo Scene".to_string(),
            frame_count: 0,
        };

        // 主方块（蓝色）
        let block = Entity::new("MainBlock", EntityKind::Block).at(0.0, 0.0);
        state.entities.insert(block.id.clone(), block);

        // 红色方块
        let red = Entity::new("RedBlock", EntityKind::Block).at(-200.0, 0.0);
        state.entities.insert(red.id.clone(), red);

        // 绿色圆形
        let circle = Entity::new("GreenCircle", EntityKind::Circle).at(200.0, 0.0);
        state.entities.insert(circle.id.clone(), circle);

        state
    }

    /// 按名称查找实体 ID
    pub fn find_by_name(&self, name: &str) -> Option<&Entity> {
        self.entities.values().find(|e| e.name == name)
    }

    /// 按名称查找实体 ID（可变）
    pub fn find_by_name_mut(&mut self, name: &str) -> Option<&mut Entity> {
        self.entities.values_mut().find(|e| e.name == name)
    }

    /// 获取所有实体的摘要列表
    pub fn entity_list(&self) -> Vec<serde_json::Value> {
        let mut list: Vec<_> = self
            .entities
            .values()
            .map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "name": e.name,
                    "kind": e.kind,
                    "position": { "x": e.position.x, "y": e.position.y },
                    "rotation": e.rotation,
                    "visible": e.visible,
                })
            })
            .collect();
        // 按名称排序，方便阅读
        list.sort_by_key(|v| v["name"].as_str().unwrap_or("").to_string());
        list
    }
}

// ── 引擎桥接句柄 ──────────────────────────────────────────────────────────────

/// 引擎桥接句柄 - MCP 工具通过此访问引擎状态
///
/// 内部持有 `Arc<Mutex<SceneState>>`，可在多个 tokio 任务间共享。
#[derive(Debug, Clone)]
pub struct EngineBridge {
    state: Arc<Mutex<SceneState>>,
}

impl EngineBridge {
    /// 创建带演示场景的引擎桥接
    pub fn new_with_demo() -> Self {
        Self {
            state: Arc::new(Mutex::new(SceneState::with_demo())),
        }
    }

    /// 获取场景状态的共享引用（供工具使用）
    pub fn state(&self) -> Arc<Mutex<SceneState>> {
        Arc::clone(&self.state)
    }

    // ── 工具方法 ──────────────────────────────────────────────────────────

    /// 移动实体（按名称或 ID）
    ///
    /// 返回: 移动后的新位置
    pub fn move_entity(
        &self,
        name_or_id: &str,
        dx: f32,
        dy: f32,
    ) -> Result<(String, Vec2), String> {
        let mut state = self.state.lock().unwrap();

        // 先尝试按 ID 查找，再按名称
        let entity = if state.entities.contains_key(name_or_id) {
            state.entities.get_mut(name_or_id)
        } else {
            state.entities.values_mut().find(|e| e.name == name_or_id)
        };

        match entity {
            Some(e) => {
                e.position.x += dx;
                e.position.y += dy;
                Ok((e.name.clone(), e.position))
            }
            None => Err(format!(
                "Entity '{}' not found. Available: [{}]",
                name_or_id,
                state
                    .entities
                    .values()
                    .map(|e| e.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    /// 设置实体位置（绝对坐标）
    pub fn set_position(&self, name_or_id: &str, x: f32, y: f32) -> Result<String, String> {
        let mut state = self.state.lock().unwrap();

        let entity = if state.entities.contains_key(name_or_id) {
            state.entities.get_mut(name_or_id)
        } else {
            state.entities.values_mut().find(|e| e.name == name_or_id)
        };

        match entity {
            Some(e) => {
                e.position = Vec2::new(x, y);
                Ok(format!("Entity '{}' moved to ({:.1}, {:.1})", e.name, x, y))
            }
            None => Err(format!("Entity '{}' not found", name_or_id)),
        }
    }

    /// 生成新实体
    pub fn spawn_entity(&self, name: &str, kind: EntityKind, x: f32, y: f32) -> String {
        let entity = Entity::new(name, kind).at(x, y);
        let id = entity.id.clone();
        self.state
            .lock()
            .unwrap()
            .entities
            .insert(id.clone(), entity);
        tracing::info!(
            "Spawned entity '{}' at ({:.1}, {:.1}), id={}",
            name,
            x,
            y,
            id
        );
        id
    }

    /// 删除实体
    pub fn despawn_entity(&self, name_or_id: &str) -> Result<String, String> {
        let mut state = self.state.lock().unwrap();

        let id = if state.entities.contains_key(name_or_id) {
            Some(name_or_id.to_string())
        } else {
            state
                .entities
                .values()
                .find(|e| e.name == name_or_id)
                .map(|e| e.id.clone())
        };

        match id {
            Some(id) => {
                let removed = state.entities.remove(&id).unwrap();
                Ok(format!("Despawned entity '{}'", removed.name))
            }
            None => Err(format!("Entity '{}' not found", name_or_id)),
        }
    }

    /// 获取场景 JSON 快照（供 AI 读取场景状态）
    pub fn get_scene_snapshot(&self) -> serde_json::Value {
        let state = self.state.lock().unwrap();
        serde_json::json!({
            "scene_name": state.scene_name,
            "frame_count": state.frame_count,
            "entity_count": state.entities.len(),
            "entities": state.entity_list(),
        })
    }

    /// 获取单个实体详情
    pub fn get_entity(&self, name_or_id: &str) -> Option<serde_json::Value> {
        let state = self.state.lock().unwrap();

        let entity = if state.entities.contains_key(name_or_id) {
            state.entities.get(name_or_id)
        } else {
            state.entities.values().find(|e| e.name == name_or_id)
        };

        entity.map(|e| serde_json::to_value(e).unwrap_or(serde_json::Value::Null))
    }

    /// 设置实体属性
    pub fn set_property(
        &self,
        name_or_id: &str,
        property: &str,
        value: serde_json::Value,
    ) -> Result<String, String> {
        let mut state = self.state.lock().unwrap();

        let entity = if state.entities.contains_key(name_or_id) {
            state.entities.get_mut(name_or_id)
        } else {
            state.entities.values_mut().find(|e| e.name == name_or_id)
        };

        match entity {
            Some(e) => {
                // 处理内置属性
                match property {
                    "visible" => {
                        if let Some(b) = value.as_bool() {
                            e.visible = b;
                            return Ok(format!("Set '{}'.visible = {}", e.name, b));
                        }
                    }
                    "rotation" => {
                        if let Some(f) = value.as_f64() {
                            e.rotation = f as f32;
                            return Ok(format!("Set '{}'.rotation = {:.3} rad", e.name, f));
                        }
                    }
                    "scale_x" => {
                        if let Some(f) = value.as_f64() {
                            e.scale.x = f as f32;
                            return Ok(format!("Set '{}'.scale.x = {:.2}", e.name, f));
                        }
                    }
                    "scale_y" => {
                        if let Some(f) = value.as_f64() {
                            e.scale.y = f as f32;
                            return Ok(format!("Set '{}'.scale.y = {:.2}", e.name, f));
                        }
                    }
                    _ => {}
                }
                // 存入自定义属性
                e.properties.insert(property.to_string(), value.clone());
                Ok(format!("Set '{}'.{} = {}", e.name, property, value))
            }
            None => Err(format!("Entity '{}' not found", name_or_id)),
        }
    }

    /// 推进模拟帧数（用于测试/演示）
    pub fn tick(&self) {
        let mut state = self.state.lock().unwrap();
        state.frame_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let bridge = EngineBridge::new_with_demo();
        let snapshot = bridge.get_scene_snapshot();
        assert_eq!(snapshot["entity_count"], 3);
        assert_eq!(snapshot["scene_name"], "Demo Scene");
    }

    #[test]
    fn test_move_entity() {
        let bridge = EngineBridge::new_with_demo();
        let (name, pos) = bridge.move_entity("MainBlock", 50.0, 0.0).unwrap();
        assert_eq!(name, "MainBlock");
        assert!((pos.x - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_spawn_despawn() {
        let bridge = EngineBridge::new_with_demo();
        let id = bridge.spawn_entity("TestBlock", EntityKind::Block, 100.0, 100.0);
        assert!(!id.is_empty());

        // 验证已生成
        let before = bridge.get_scene_snapshot();
        assert_eq!(before["entity_count"], 4);

        // 删除
        bridge.despawn_entity("TestBlock").unwrap();
        let after = bridge.get_scene_snapshot();
        assert_eq!(after["entity_count"], 3);
    }

    #[test]
    fn test_move_unknown_entity() {
        let bridge = EngineBridge::new_with_demo();
        let result = bridge.move_entity("NonExistent", 10.0, 0.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
