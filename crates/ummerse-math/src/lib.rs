//! # Ummerse Math
//!
//! 数学工具库，提供向量、矩阵、四元数、变换等基础数学类型。
//! 基于 [`glam`] 构建，遵循 Godot 风格的 API 设计。
//!
//! ## 主要类型
//! - [`Transform2d`] / [`Transform3d`] – 2D/3D 变换（位置、旋转、缩放）
//! - [`Aabb2d`] / [`Aabb3d`] – 轴对齐包围盒
//! - [`Rect2`] – 2D 矩形
//! - [`Color`] – RGBA 颜色
//! - [`Plane`] – 3D 平面

pub mod aabb;
pub mod color;
pub mod plane;
pub mod rect;
pub mod transform;

// Re-export glam 核心类型
pub use glam::{
    ivec2, ivec3, ivec4, uvec2, uvec3, uvec4, vec2, vec3, vec4, BVec2, BVec3, BVec4, DMat2, DMat3,
    DMat4, DQuat, DVec2, DVec3, DVec4, EulerRot, IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, Quat,
    UVec2, UVec3, UVec4, Vec2, Vec3, Vec3A, Vec4,
};

pub use aabb::{Aabb2d, Aabb3d};
pub use color::Color;
pub use plane::Plane;
pub use rect::Rect2;
pub use transform::{Transform2d, Transform3d};

/// 常用数学常量
pub mod consts {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
    pub const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;
    pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;
}

/// 插值工具函数
pub mod lerp {
    /// 线性插值
    #[inline]
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// 反线性插值 - 从值求 t
    #[inline]
    pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
        if (b - a).abs() < f32::EPSILON {
            0.0
        } else {
            (value - a) / (b - a)
        }
    }

    /// 平滑步进
    #[inline]
    pub fn smoothstep(a: f32, b: f32, t: f32) -> f32 {
        let t = ((t - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// 将值限制在范围内
    #[inline]
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }
}

/// 近似相等比较
#[inline]
#[must_use]
pub fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

/// 角度转弧度
#[inline]
#[must_use]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * consts::DEG_TO_RAD
}

/// 弧度转角度
#[inline]
#[must_use]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * consts::RAD_TO_DEG
}
