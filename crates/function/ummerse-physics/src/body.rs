//! 刚体定义

use glam::{Vec2, Vec3, Quat};
use serde::{Deserialize, Serialize};

/// 2D 刚体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody2d {
    pub id: u64,
    pub position: Vec2,
    pub rotation: f32,
    pub linear_velocity: Vec2,
    pub angular_velocity: f32,
    pub mass: f32,
    pub inertia: f32,
    pub gravity_scale: f32,
    pub is_static: bool,
    pub is_kinematic: bool,
    pub is_sleeping: bool,
}

impl RigidBody2d {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            position: Vec2::ZERO,
            rotation: 0.0,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            mass: 1.0,
            inertia: 1.0,
            gravity_scale: 1.0,
            is_static: false,
            is_kinematic: false,
            is_sleeping: false,
        }
    }

    /// 施加力（改变速度）
    pub fn apply_force(&mut self, force: Vec2, delta: f32) {
        if !self.is_static {
            self.linear_velocity += force * (delta / self.mass);
        }
    }

    /// 施加冲量（直接改变动量）
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if !self.is_static {
            self.linear_velocity += impulse / self.mass;
        }
    }

    /// 物理步进
    pub fn integrate(&mut self, gravity: Vec2, delta: f32) {
        if self.is_static || self.is_sleeping { return; }
        self.linear_velocity += gravity * self.gravity_scale * delta;
        self.position += self.linear_velocity * delta;
        self.rotation += self.angular_velocity * delta;
    }
}

/// 3D 刚体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody3d {
    pub id: u64,
    pub position: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub gravity_scale: f32,
    pub is_static: bool,
    pub is_kinematic: bool,
}

impl RigidBody3d {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            gravity_scale: 1.0,
            is_static: false,
            is_kinematic: false,
        }
    }

    pub fn apply_force(&mut self, force: Vec3, delta: f32) {
        if !self.is_static {
            self.linear_velocity += force * (delta / self.mass);
        }
    }

    pub fn integrate(&mut self, gravity: Vec3, delta: f32) {
        if self.is_static { return; }
        self.linear_velocity += gravity * self.gravity_scale * delta;
        self.position += self.linear_velocity * delta;
    }
}
