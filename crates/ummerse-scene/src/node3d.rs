//! 3D 节点

use ummerse_core::node::{NodeId, NodeType};
use ummerse_math::transform::Transform3d;
use serde::{Deserialize, Serialize};

/// 3D 节点 - 带 3D 变换的场景节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node3d {
    pub id: NodeId,
    pub name: String,
    pub transform: Transform3d,
}

impl Node3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            transform: Transform3d::IDENTITY,
        }
    }

    pub fn node_type() -> NodeType {
        NodeType::Node3d
    }

    pub fn global_position(&self) -> glam::Vec3 {
        self.transform.position
    }

    pub fn forward(&self) -> glam::Vec3 {
        self.transform.forward()
    }

    pub fn right(&self) -> glam::Vec3 {
        self.transform.right()
    }

    pub fn up(&self) -> glam::Vec3 {
        self.transform.up()
    }
}
