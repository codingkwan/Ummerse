//! # Ummerse Scene
//!
//! 场景树系统，参考 Godot 的场景/节点架构：
//! - `SceneTree`：场景树根，管理所有节点
//! - `SceneNodeData`：带变换的节点数据
//! - `Scene`：可序列化的场景资产（.uscn 格式）
//! - `Node2d` / `Node3d`：具体节点类型及其派生类型

pub mod components;
pub mod node2d;
pub mod node3d;
pub mod scene;
pub mod scene_tree;

// ── 2D 节点导出 ───────────────────────────────────────────────────────────────
pub use node2d::{
    AnimatedSprite2d, AnimationFrame, Area2d, Camera2dNode, CharacterBody2d, CharacterMotionMode,
    CollisionShape2dNode, CpuParticles2d, Node2d, Particle2d, RayCast2dNode, Shape2dDef, Sprite2d,
    SpriteAnimation, TileDef, TileMap, TileSet,
};

// ── 3D 节点导出 ───────────────────────────────────────────────────────────────
pub use node3d::{
    Camera3dNode, DirectionalLight3d, EmitterShape, LightColor, MeshInstance3d, Node3d,
    ParticleSystem3d, PointLight3d, RigidBody3dNode, RigidBodyType3d, SpotLight3d,
};

// ── 场景导出 ──────────────────────────────────────────────────────────────────
pub use scene::{Scene, SceneAsset, SceneMetadata};
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

    /// 添加标签
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let t = tag.into();
        if !self.tags.contains(&t) {
            self.tags.push(t);
        }
    }

    /// 判断是否含有指定标签
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// 设置属性
    pub fn set_property(&mut self, key: impl Into<String>, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.properties {
            map.insert(key.into(), value);
        }
    }

    /// 获取属性
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        if let serde_json::Value::Object(ref map) = self.properties {
            map.get(key)
        } else {
            None
        }
    }
}
