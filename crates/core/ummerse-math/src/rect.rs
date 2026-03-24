//! 2D 矩形类型

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// 2D 轴对齐矩形（左上角 + 尺寸）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect2 {
    /// 左上角位置
    pub position: Vec2,
    /// 宽高
    pub size: Vec2,
}

impl Rect2 {
    pub const ZERO: Self = Self {
        position: Vec2::ZERO,
        size: Vec2::ZERO,
    };

    /// 从位置和尺寸创建
    #[inline]
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(w, h),
        }
    }

    /// 从两个角点创建
    #[inline]
    pub fn from_corners(min: Vec2, max: Vec2) -> Self {
        Self {
            position: min,
            size: max - min,
        }
    }

    /// 矩形右下角
    #[inline]
    pub fn end(&self) -> Vec2 {
        self.position + self.size
    }

    /// 矩形中心
    #[inline]
    pub fn center(&self) -> Vec2 {
        self.position + self.size * 0.5
    }

    /// 面积
    #[inline]
    pub fn area(&self) -> f32 {
        self.size.x * self.size.y
    }

    /// 点是否在矩形内（含边界）
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        let end = self.end();
        point.x >= self.position.x
            && point.y >= self.position.y
            && point.x <= end.x
            && point.y <= end.y
    }

    /// 与另一矩形是否相交
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        let a_end = self.end();
        let b_end = other.end();
        self.position.x < b_end.x
            && a_end.x > other.position.x
            && self.position.y < b_end.y
            && a_end.y > other.position.y
    }

    /// 求交集矩形
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min = self.position.max(other.position);
        let max = self.end().min(other.end());
        if max.x > min.x && max.y > min.y {
            Some(Self::from_corners(min, max))
        } else {
            None
        }
    }

    /// 合并两个矩形（包围盒）
    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        let min = self.position.min(other.position);
        let max = self.end().max(other.end());
        Self::from_corners(min, max)
    }

    /// 扩展以包含一个点
    #[inline]
    pub fn expand_to(&self, point: Vec2) -> Self {
        let min = self.position.min(point);
        let max = self.end().max(point);
        Self::from_corners(min, max)
    }

    /// 向四周膨胀
    #[inline]
    pub fn grow(&self, amount: f32) -> Self {
        Self {
            position: self.position - Vec2::splat(amount),
            size: self.size + Vec2::splat(amount * 2.0),
        }
    }
}

impl Default for Rect2 {
    fn default() -> Self {
        Self::ZERO
    }
}
