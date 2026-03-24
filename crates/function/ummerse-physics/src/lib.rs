//! # Ummerse Physics
//!
//! 物理引擎模块：
//! - 2D 物理：刚体、静态体、碰撞体、关节约束
//! - 3D 物理：基础刚体和碰撞检测（占位，可接入 rapier3d）
//! - 碰撞事件系统
//! - 物理世界步进

pub mod body;
pub mod collider;
pub mod joint;
pub mod world;

pub use body::{RigidBody2d, RigidBody3d};
pub use collider::{Collider2d, Collider3d, ColliderShape2d, ColliderShape3d};
pub use joint::{DistanceJoint, Joint2d, RevoluteJoint};
pub use world::{PhysicsWorld2d, PhysicsWorld3d, RaycastHit2d, RaycastHit3d};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

// ── 刚体类型 ──────────────────────────────────────────────────────────────────

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RigidBodyType {
    /// 动态刚体（受力影响）
    #[default]
    Dynamic,
    /// 静态刚体（不运动）
    Static,
    /// 运动学刚体（手动控制位置）
    Kinematic,
}

// ── 碰撞事件 ──────────────────────────────────────────────────────────────────

/// 碰撞接触点
#[derive(Debug, Clone)]
pub struct ContactPoint {
    /// 接触点世界坐标
    pub point: Vec3,
    /// 碰撞法线（从 A 指向 B）
    pub normal: Vec3,
    /// 穿透深度
    pub penetration: f32,
}

impl ContactPoint {
    pub fn new(point: Vec3, normal: Vec3, penetration: f32) -> Self {
        Self {
            point,
            normal,
            penetration,
        }
    }
}

/// 碰撞事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventKind {
    /// 开始接触（本帧首次碰撞）
    Entered,
    /// 持续接触（保持碰撞状态）
    Persisted,
    /// 结束接触（本帧分离）
    Exited,
}

/// 碰撞事件
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    /// 第一个物体 ID
    pub body_a: u64,
    /// 第二个物体 ID
    pub body_b: u64,
    /// 接触点（世界坐标）
    pub contact_point: Vec3,
    /// 碰撞法线
    pub contact_normal: Vec3,
    /// 穿透深度
    pub penetration: f32,
    /// 事件类型
    pub kind: CollisionEventKind,
}

impl CollisionEvent {
    /// 创建碰撞开始事件
    pub fn entered(body_a: u64, body_b: u64, point: Vec3, normal: Vec3, penetration: f32) -> Self {
        Self {
            body_a,
            body_b,
            contact_point: point,
            contact_normal: normal,
            penetration,
            kind: CollisionEventKind::Entered,
        }
    }

    /// 创建碰撞持续事件
    pub fn persisted(
        body_a: u64,
        body_b: u64,
        point: Vec3,
        normal: Vec3,
        penetration: f32,
    ) -> Self {
        Self {
            body_a,
            body_b,
            contact_point: point,
            contact_normal: normal,
            penetration,
            kind: CollisionEventKind::Persisted,
        }
    }
}

// ── 物理材质 ──────────────────────────────────────────────────────────────────

/// 物理材质 - 定义摩擦力和弹性
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsMaterial {
    /// 摩擦系数（0.0 = 无摩擦，1.0 = 高摩擦）
    pub friction: f32,
    /// 恢复系数（弹性，0.0 = 完全非弹性，1.0 = 完全弹性）
    pub restitution: f32,
    /// 密度（kg/m²）
    pub density: f32,
}

impl PhysicsMaterial {
    pub fn new(friction: f32, restitution: f32, density: f32) -> Self {
        Self {
            friction,
            restitution,
            density,
        }
    }

    /// 标准材质（类似橡胶）
    pub fn default_material() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.3,
            density: 1.0,
        }
    }

    /// 冰面（低摩擦）
    pub fn ice() -> Self {
        Self {
            friction: 0.05,
            restitution: 0.1,
            density: 0.9,
        }
    }

    /// 弹球（高弹性）
    pub fn bouncy() -> Self {
        Self {
            friction: 0.3,
            restitution: 0.9,
            density: 0.5,
        }
    }

    /// 石头（高密度、低弹性）
    pub fn stone() -> Self {
        Self {
            friction: 0.8,
            restitution: 0.1,
            density: 2.5,
        }
    }
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self::default_material()
    }
}

// ── 碰撞层/掩码 ───────────────────────────────────────────────────────────────

/// 碰撞层掩码（32位，最多32层）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollisionLayers {
    /// 该物体所在的层
    pub membership: u32,
    /// 该物体与哪些层碰撞
    pub filter: u32,
}

impl CollisionLayers {
    /// 与所有层碰撞（默认）
    pub fn all() -> Self {
        Self {
            membership: u32::MAX,
            filter: u32::MAX,
        }
    }

    /// 只与指定层碰撞
    pub fn only(layer: u32) -> Self {
        Self {
            membership: 1 << layer,
            filter: 1 << layer,
        }
    }

    /// 创建自定义层配置
    pub fn new(membership: u32, filter: u32) -> Self {
        Self { membership, filter }
    }

    /// 检查两个层是否会碰撞
    pub fn interacts_with(&self, other: &CollisionLayers) -> bool {
        (self.filter & other.membership) != 0 || (other.filter & self.membership) != 0
    }
}

impl Default for CollisionLayers {
    fn default() -> Self {
        Self::all()
    }
}

// ── 射线检测 ──────────────────────────────────────────────────────────────────

/// 2D 射线
#[derive(Debug, Clone, Copy)]
pub struct Ray2d {
    pub origin: Vec2,
    pub direction: Vec2,
    pub max_distance: f32,
}

impl Ray2d {
    pub fn new(origin: Vec2, direction: Vec2, max_distance: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize_or_zero(),
            max_distance,
        }
    }

    /// 计算射线上某点
    pub fn point_at(&self, t: f32) -> Vec2 {
        self.origin + self.direction * t
    }
}

/// 射线检测结果
#[derive(Debug, Clone)]
pub struct RayCastHit2d {
    /// 命中位置
    pub point: Vec2,
    /// 命中法线
    pub normal: Vec2,
    /// 距离
    pub distance: f32,
    /// 命中物体 ID
    pub body_id: u64,
}

// ── 物理错误 ──────────────────────────────────────────────────────────────────

/// 物理系统错误
#[derive(Debug, thiserror::Error)]
pub enum PhysicsError {
    #[error("Body not found: {0}")]
    BodyNotFound(u64),

    #[error("Invalid collider shape: {0}")]
    InvalidShape(String),

    #[error("Joint error: {0}")]
    JointError(String),
}

/// 物理系统 Result
pub type Result<T> = std::result::Result<T, PhysicsError>;
