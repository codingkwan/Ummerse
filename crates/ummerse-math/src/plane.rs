//! 3D 平面类型

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// 3D 平面（法线 + 距离表示：n·x = d）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Plane {
    /// 单位法向量
    pub normal: Vec3,
    /// 平面到原点的有符号距离
    pub d: f32,
}

impl Plane {
    pub const XY: Self = Self { normal: Vec3::Z, d: 0.0 };
    pub const XZ: Self = Self { normal: Vec3::Y, d: 0.0 };
    pub const YZ: Self = Self { normal: Vec3::X, d: 0.0 };

    /// 从法线和距离创建（法线需已归一化）
    #[inline]
    pub fn new(normal: Vec3, d: f32) -> Self {
        Self { normal, d }
    }

    /// 从法线和平面上一点创建
    #[inline]
    pub fn from_normal_point(normal: Vec3, point: Vec3) -> Self {
        let normal = normal.normalize();
        let d = normal.dot(point);
        Self { normal, d }
    }

    /// 从平面上三点创建（逆时针为正面）
    pub fn from_three_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = (b - a).cross(c - a).normalize();
        Self::from_normal_point(normal, a)
    }

    /// 点到平面的有符号距离（正表示法线侧）
    #[inline]
    pub fn signed_distance(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.d
    }

    /// 点是否在平面正侧（法线方向）
    #[inline]
    pub fn is_point_over(&self, point: Vec3) -> bool {
        self.signed_distance(point) > 0.0
    }

    /// 将点投影到平面上
    #[inline]
    pub fn project_point(&self, point: Vec3) -> Vec3 {
        point - self.normal * self.signed_distance(point)
    }

    /// 射线与平面的交点
    pub fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let denom = self.normal.dot(direction);
        if denom.abs() < f32::EPSILON {
            return None; // 平行
        }
        let t = (self.d - self.normal.dot(origin)) / denom;
        if t >= 0.0 {
            Some(origin + direction * t)
        } else {
            None
        }
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self::XZ
    }
}
