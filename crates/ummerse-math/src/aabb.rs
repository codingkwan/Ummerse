//! 轴对齐包围盒（AABB）

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 2D 轴对齐包围盒
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb2d {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2d {
    pub const ZERO: Self = Self {
        min: Vec2::ZERO,
        max: Vec2::ZERO,
    };

    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_center_half_size(center: Vec2, half_size: Vec2) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn half_size(&self) -> Vec2 {
        (self.max - self.min) * 0.5
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.y >= self.min.y
            && point.x <= self.max.x
            && point.y <= self.max.y
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    #[inline]
    pub fn expand_to_point(&self, point: Vec2) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }
}

impl Default for Aabb2d {
    fn default() -> Self {
        Self::ZERO
    }
}

// ── 3D AABB ──────────────────────────────────────────────────────────────────

/// 3D 轴对齐包围盒
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb3d {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb3d {
    pub const ZERO: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_center_half_size(center: Vec3, half_size: Vec3) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn half_size(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    #[inline]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    #[inline]
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.y >= self.min.y
            && point.z >= self.min.z
            && point.x <= self.max.x
            && point.y <= self.max.y
            && point.z <= self.max.z
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    #[inline]
    pub fn expand_to_point(&self, point: Vec3) -> Self {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    /// 计算射线与 AABB 的交点（slab method）
    pub fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let inv_dir = Vec3::ONE / direction;
        let t1 = (self.min - origin) * inv_dir;
        let t2 = (self.max - origin) * inv_dir;
        let t_min = t1.min(t2);
        let t_max = t1.max(t2);
        let t_enter = t_min.x.max(t_min.y).max(t_min.z);
        let t_exit = t_max.x.min(t_max.y).min(t_max.z);
        if t_exit >= t_enter && t_exit >= 0.0 {
            Some(t_enter.max(0.0))
        } else {
            None
        }
    }
}

impl Default for Aabb3d {
    fn default() -> Self {
        Self::ZERO
    }
}
