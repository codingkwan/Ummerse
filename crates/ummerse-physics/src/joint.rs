//! 物理关节/约束系统
//!
//! 支持多种 2D 关节类型，参考 Box2D 的关节系统：
//! - 距离关节（弹簧效果）
//! - 转动关节（铰链）
//! - 棱柱关节（滑轨）
//! - 滑轮关节
//! - 焊接关节（固定连接）

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// 关节唯一 ID
pub type JointId = u64;

// ── 关节 Trait ────────────────────────────────────────────────────────────────

/// 关节基础信息（所有关节共享）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointBase {
    /// 关节 ID
    pub id: JointId,
    /// 第一个刚体 ID
    pub body_a: u64,
    /// 第二个刚体 ID
    pub body_b: u64,
    /// 关节 A 的锚点（相对于 body_a 局部坐标）
    pub anchor_a: Vec2,
    /// 关节 B 的锚点（相对于 body_b 局部坐标）
    pub anchor_b: Vec2,
    /// 是否允许两个物体相互碰撞
    pub collide_connected: bool,
}

impl JointBase {
    pub fn new(body_a: u64, body_b: u64) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            body_a,
            body_b,
            anchor_a: Vec2::ZERO,
            anchor_b: Vec2::ZERO,
            collide_connected: false,
        }
    }

    pub fn with_anchors(mut self, anchor_a: Vec2, anchor_b: Vec2) -> Self {
        self.anchor_a = anchor_a;
        self.anchor_b = anchor_b;
        self
    }

    pub fn collide_connected(mut self, collide: bool) -> Self {
        self.collide_connected = collide;
        self
    }
}

// ── 距离关节 ──────────────────────────────────────────────────────────────────

/// 距离关节 - 保持两点间距离（可带弹簧）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceJoint {
    pub base: JointBase,
    /// 静止长度（米）
    pub length: f32,
    /// 最小长度（米，-1 = 无限制）
    pub min_length: f32,
    /// 最大长度（米，-1 = 无限制）
    pub max_length: f32,
    /// 弹簧刚度（0 = 硬约束）
    pub stiffness: f32,
    /// 弹簧阻尼
    pub damping: f32,
}

impl DistanceJoint {
    pub fn new(body_a: u64, body_b: u64, length: f32) -> Self {
        Self {
            base: JointBase::new(body_a, body_b),
            length,
            min_length: -1.0,
            max_length: -1.0,
            stiffness: 0.0,
            damping: 0.0,
        }
    }

    /// 配置为弹簧
    pub fn spring(mut self, stiffness: f32, damping: f32) -> Self {
        self.stiffness = stiffness;
        self.damping = damping;
        self
    }

    /// 设置长度范围（绳子效果）
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }
}

// ── 转动关节（铰链）──────────────────────────────────────────────────────────

/// 转动关节 - 允许绕锚点旋转（铰链/轴销）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevoluteJoint {
    pub base: JointBase,
    /// 是否限制旋转角度
    pub enable_limit: bool,
    /// 最小角度（弧度）
    pub lower_angle: f32,
    /// 最大角度（弧度）
    pub upper_angle: f32,
    /// 是否启用电机
    pub enable_motor: bool,
    /// 电机目标速度（弧度/秒）
    pub motor_speed: f32,
    /// 电机最大扭矩（N·m）
    pub max_motor_torque: f32,
}

impl RevoluteJoint {
    pub fn new(body_a: u64, body_b: u64, world_anchor: Vec2) -> Self {
        let mut base = JointBase::new(body_a, body_b);
        base.anchor_a = world_anchor;
        base.anchor_b = world_anchor;
        Self {
            base,
            enable_limit: false,
            lower_angle: -std::f32::consts::PI,
            upper_angle: std::f32::consts::PI,
            enable_motor: false,
            motor_speed: 0.0,
            max_motor_torque: 0.0,
        }
    }

    /// 限制旋转角度范围
    pub fn with_limits(mut self, lower: f32, upper: f32) -> Self {
        self.enable_limit = true;
        self.lower_angle = lower;
        self.upper_angle = upper;
        self
    }

    /// 启用电机
    pub fn with_motor(mut self, speed: f32, max_torque: f32) -> Self {
        self.enable_motor = true;
        self.motor_speed = speed;
        self.max_motor_torque = max_torque;
        self
    }
}

// ── 棱柱关节（滑轨）──────────────────────────────────────────────────────────

/// 棱柱关节 - 沿指定轴平移，防止旋转
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismaticJoint {
    pub base: JointBase,
    /// 允许平移的轴方向（归一化）
    pub axis: Vec2,
    /// 是否限制移动范围
    pub enable_limit: bool,
    /// 最小偏移（米）
    pub lower_translation: f32,
    /// 最大偏移（米）
    pub upper_translation: f32,
    /// 是否启用电机
    pub enable_motor: bool,
    /// 电机目标速度（米/秒）
    pub motor_speed: f32,
    /// 电机最大力（N）
    pub max_motor_force: f32,
}

impl PrismaticJoint {
    pub fn new(body_a: u64, body_b: u64, axis: Vec2) -> Self {
        Self {
            base: JointBase::new(body_a, body_b),
            axis: axis.normalize_or_zero(),
            enable_limit: false,
            lower_translation: -f32::MAX,
            upper_translation: f32::MAX,
            enable_motor: false,
            motor_speed: 0.0,
            max_motor_force: 0.0,
        }
    }

    /// 限制平移范围
    pub fn with_limits(mut self, lower: f32, upper: f32) -> Self {
        self.enable_limit = true;
        self.lower_translation = lower;
        self.upper_translation = upper;
        self
    }
}

// ── 焊接关节 ──────────────────────────────────────────────────────────────────

/// 焊接关节 - 固定两个物体的相对位置和角度（可带弹性）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeldJoint {
    pub base: JointBase,
    /// 参考角度（弧度）
    pub reference_angle: f32,
    /// 角频率（Hz，0 = 硬约束）
    pub frequency_hz: f32,
    /// 阻尼比
    pub damping_ratio: f32,
}

impl WeldJoint {
    pub fn new(body_a: u64, body_b: u64, world_anchor: Vec2) -> Self {
        let mut base = JointBase::new(body_a, body_b);
        base.anchor_a = world_anchor;
        base.anchor_b = world_anchor;
        Self {
            base,
            reference_angle: 0.0,
            frequency_hz: 0.0,
            damping_ratio: 0.0,
        }
    }

    pub fn with_spring(mut self, frequency: f32, damping: f32) -> Self {
        self.frequency_hz = frequency;
        self.damping_ratio = damping;
        self
    }
}

// ── 关节枚举 ──────────────────────────────────────────────────────────────────

/// 2D 关节枚举（统一存储）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Joint2d {
    Distance(DistanceJoint),
    Revolute(RevoluteJoint),
    Prismatic(PrismaticJoint),
    Weld(WeldJoint),
}

impl Joint2d {
    pub fn id(&self) -> JointId {
        match self {
            Self::Distance(j) => j.base.id,
            Self::Revolute(j) => j.base.id,
            Self::Prismatic(j) => j.base.id,
            Self::Weld(j) => j.base.id,
        }
    }

    pub fn body_a(&self) -> u64 {
        match self {
            Self::Distance(j) => j.base.body_a,
            Self::Revolute(j) => j.base.body_a,
            Self::Prismatic(j) => j.base.body_a,
            Self::Weld(j) => j.base.body_a,
        }
    }

    pub fn body_b(&self) -> u64 {
        match self {
            Self::Distance(j) => j.base.body_b,
            Self::Revolute(j) => j.base.body_b,
            Self::Prismatic(j) => j.base.body_b,
            Self::Weld(j) => j.base.body_b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revolute_joint() {
        let joint = RevoluteJoint::new(1, 2, Vec2::ZERO)
            .with_limits(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2)
            .with_motor(1.0, 10.0);

        assert!(joint.enable_limit);
        assert!(joint.enable_motor);
    }

    #[test]
    fn test_distance_joint_spring() {
        let joint = DistanceJoint::new(1, 2, 2.0).spring(100.0, 0.5);
        assert_eq!(joint.stiffness, 100.0);
        assert_eq!(joint.length, 2.0);
    }

    #[test]
    fn test_joint2d_enum() {
        let j = Joint2d::Revolute(RevoluteJoint::new(10, 20, Vec2::new(1.0, 2.0)));
        assert_eq!(j.body_a(), 10);
        assert_eq!(j.body_b(), 20);
    }
}
