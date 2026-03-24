//! # Ummerse Math
//!
//! 数学工具库，提供向量、矩阵、四元数、变换等基础数学类型。
//! 基于 [`glam`] 构建，遵循 Godot 风格的 API 设计。
//!
//! ## 主要类型
//! - [`Transform2d`] / [`Transform3d`] – 2D/3D 变换（位置、旋转、缩放）
//! - [`Aabb2d`] / [`Aabb3d`] – 轴对齐包围盒
//! - [`Rect2`] – 2D 矩形
//! - [`Color`] – RGBA 颜色（支持 Pod/Zeroable，可直接用于 GPU）
//! - [`Plane`] – 3D 平面

pub mod aabb;
pub mod color;
pub mod plane;
pub mod rect;
pub mod transform;

// ── Re-export glam 核心类型 ────────────────────────────────────────────────
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

// ── 常用数学常量 ───────────────────────────────────────────────────────────
/// 常用数学常量
pub mod consts {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
    pub const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;
    pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;
    /// 无穷小正数（float 安全除法保护）
    pub const SMALL: f32 = 1e-6;
}

// ── 插值工具函数 ──────────────────────────────────────────────────────────
/// 插值工具函数模块
pub mod lerp {
    /// 线性插值
    #[inline]
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// 反线性插值：从值求 `t`，使得 `lerp(a, b, t) == value`
    #[inline]
    pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
        if (b - a).abs() < f32::EPSILON {
            0.0
        } else {
            (value - a) / (b - a)
        }
    }

    /// 平滑步进（Ken Perlin 版本，三次多项式）
    #[inline]
    pub fn smoothstep(edge0: f32, edge1: f32, t: f32) -> f32 {
        let t = ((t - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// 更平滑的步进（五次多项式，零一、二阶导数）
    #[inline]
    pub fn smootherstep(edge0: f32, edge1: f32, t: f32) -> f32 {
        let t = ((t - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    /// 将值限制在范围内
    #[inline]
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }
}

// ── 全局工具函数 ──────────────────────────────────────────────────────────

/// 近似相等比较（默认精度 `f32::EPSILON`）
#[inline]
#[must_use]
pub fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

/// 近似相等比较（自定义精度）
#[inline]
#[must_use]
pub fn approx_eq_eps(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
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

/// 将角度规范化到 [-π, π] 范围
#[inline]
#[must_use]
pub fn normalize_angle(angle: f32) -> f32 {
    let a = angle.rem_euclid(consts::TAU);
    if a > consts::PI { a - consts::TAU } else { a }
}

/// 两个角度之间的最短差值（-π ~ π）
#[inline]
#[must_use]
pub fn angle_diff(from: f32, to: f32) -> f32 {
    normalize_angle(to - from)
}

/// 符号函数，返回 -1.0 / 0.0 / 1.0
#[inline]
#[must_use]
pub fn sign(x: f32) -> f32 {
    if x < 0.0 {
        -1.0
    } else if x > 0.0 {
        1.0
    } else {
        0.0
    }
}

/// 将值从 `[in_min, in_max]` 映射到 `[out_min, out_max]`
#[inline]
#[must_use]
pub fn remap(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = lerp::inverse_lerp(in_min, in_max, value);
    lerp::lerp(out_min, out_max, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deg_rad_roundtrip() {
        let deg = 90.0_f32;
        let rad = deg_to_rad(deg);
        assert!(approx_eq(rad_to_deg(rad), deg));
    }

    #[test]
    fn test_normalize_angle() {
        assert!(approx_eq(normalize_angle(0.0), 0.0));
        assert!(approx_eq(
            normalize_angle(consts::TAU + 1.0),
            normalize_angle(1.0)
        ));
    }

    #[test]
    fn test_remap() {
        let v = remap(0.5, 0.0, 1.0, 0.0, 100.0);
        assert!(approx_eq(v, 50.0));
    }
}
