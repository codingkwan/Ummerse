//! 2D 节点

use ummerse_core::node::{NodeId, NodeType};
use ummerse_math::transform::Transform2d;
use serde::{Deserialize, Serialize};

/// 2D 节点 - 带 2D 变换的场景节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node2d {
    pub id: NodeId,
    pub name: String,
    pub transform: Transform2d,
    pub z_index: i32,
    pub z_as_relative: bool,
}

impl Node2d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            transform: Transform2d::IDENTITY,
            z_index: 0,
            z_as_relative: true,
        }
    }

    pub fn node_type() -> NodeType {
        NodeType::Node2d
    }

    /// 全局位置（考虑父节点变换）
    pub fn global_position(&self) -> glam::Vec2 {
        self.transform.position
    }
}
