//! # Ummerse Scene
//!
//! 场景树系统，参考 Godot 的场景/节点架构：
//! - `SceneTree`：场景树根，管理所有节点
//! - `SceneNode`：带变换的节点数据
//! - `Scene`：可序列化的场景资产（.uscn 格式）

pub mod components;
pub mod node2d;
pub mod node3d;
pub mod scene;
pub mod scene_tree;

pub use node2d::Node2d;
pub use node3d::Node3d;
pub use scene::{Scene, SceneAsset};
pub use scene_tree::SceneTree;

use serde::{Deserialize, Serialize};
use ummerse_core::node::{NodeId, NodeType};

/// 场景节点的通用数据（对所有节点类型共用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNodeData {
    pub id: NodeId,
    pub name: String,
    pub node_type: NodeType,
    pub enabled: bool,
    pub visible: bool,
    pub tags: Vec<String>,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    /// 节点自定义属性（JSON 序列化）
    pub properties: serde_json::Value,
}

impl SceneNodeData {
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            node_type,
            enabled: true,
            visible: true,
            tags: Vec::new(),
            parent: None,
            children: Vec::new(),
            properties: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}
