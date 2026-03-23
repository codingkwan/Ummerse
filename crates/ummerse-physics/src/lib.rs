//! # Ummerse Physics
//!
//! 物理仿真系统（2D/3D）：
//! - 刚体动力学
//! - 碰撞检测
//! - 物理材质
//! - 关节约束

pub mod body;
pub mod collider;
pub mod world;

pub use body::{RigidBody2d, RigidBody3d};
pub use collider::{Collider2d, Collider3d, ColliderShape2d, ColliderShape3d};
pub use world::{PhysicsWorld2d, PhysicsWorld3d};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 物理材质
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.0,
            density: 1.0,
        }
    }
}

/// 碰撞事件
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub body_a: u64,
    pub body_b: u64,
    pub contact_point: Vec3,
    pub contact_normal: Vec3,
    pub penetration: f32,
}
