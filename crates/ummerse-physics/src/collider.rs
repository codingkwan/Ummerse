//! 碰撞体定义

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use ummerse_math::aabb::Aabb2d;

/// 2D 碰撞形状
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape2d {
    /// 圆形
    Circle { radius: f32 },
    /// 轴对齐矩形
    Rect { half_width: f32, half_height: f32 },
    /// 多边形（凸）
    Polygon { vertices: Vec<Vec2> },
    /// 胶囊体
    Capsule { radius: f32, height: f32 },
}

/// 3D 碰撞形状
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape3d {
    /// 球体
    Sphere { radius: f32 },
    /// 轴对齐盒体
    Box { half_extents: Vec3 },
    /// 胶囊体
    Capsule { radius: f32, height: f32 },
    /// 圆柱体
    Cylinder { radius: f32, height: f32 },
}

/// 2D 碰撞体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider2d {
    /// 所属刚体 ID
    pub body_id: u64,
    /// 碰撞形状
    pub shape: ColliderShape2d,
    /// 相对于刚体中心的偏移
    pub offset: Vec2,
    /// 是否为传感器（不产生物理响应，只检测重叠）
    pub is_sensor: bool,
    /// 碰撞层（位标志）
    pub collision_layer: u32,
    /// 碰撞掩码（与哪些层发生碰撞）
    pub collision_mask: u32,
}

impl Collider2d {
    pub fn circle(body_id: u64, radius: f32) -> Self {
        Self {
            body_id,
            shape: ColliderShape2d::Circle { radius },
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_layer: 1,
            collision_mask: 1,
        }
    }

    pub fn rect(body_id: u64, half_width: f32, half_height: f32) -> Self {
        Self {
            body_id,
            shape: ColliderShape2d::Rect {
                half_width,
                half_height,
            },
            offset: Vec2::ZERO,
            is_sensor: false,
            collision_layer: 1,
            collision_mask: 1,
        }
    }

    /// 计算世界空间 AABB（用于宽相碰撞检测）
    pub fn aabb(&self, body_position: Vec2) -> Aabb2d {
        let center = body_position + self.offset;
        match &self.shape {
            ColliderShape2d::Circle { radius } => Aabb2d {
                min: center - Vec2::splat(*radius),
                max: center + Vec2::splat(*radius),
            },
            ColliderShape2d::Rect {
                half_width,
                half_height,
            } => Aabb2d {
                min: center - Vec2::new(*half_width, *half_height),
                max: center + Vec2::new(*half_width, *half_height),
            },
            ColliderShape2d::Capsule { radius, height } => Aabb2d {
                min: center - Vec2::new(*radius, height * 0.5 + radius),
                max: center + Vec2::new(*radius, height * 0.5 + radius),
            },
            ColliderShape2d::Polygon { vertices } => {
                if vertices.is_empty() {
                    return Aabb2d {
                        min: center,
                        max: center,
                    };
                }
                let mut min = vertices[0] + center;
                let mut max = vertices[0] + center;
                for v in &vertices[1..] {
                    let wv = *v + center;
                    min = min.min(wv);
                    max = max.max(wv);
                }
                Aabb2d { min, max }
            }
        }
    }

    /// 精确碰撞检测（窄相）
    pub fn intersects(
        &self,
        pos_a: Vec2,
        other: &Collider2d,
        pos_b: Vec2,
    ) -> Option<ContactInfo2d> {
        match (&self.shape, &other.shape) {
            (ColliderShape2d::Circle { radius: ra }, ColliderShape2d::Circle { radius: rb }) => {
                let ca = pos_a + self.offset;
                let cb = pos_b + other.offset;
                let diff = cb - ca;
                let dist_sq = diff.length_squared();
                let combined = ra + rb;
                if dist_sq < combined * combined {
                    let dist = dist_sq.sqrt();
                    let normal = if dist > 1e-6 { diff / dist } else { Vec2::Y };
                    let penetration = combined - dist;
                    Some(ContactInfo2d {
                        normal,
                        penetration,
                        contact_point: ca + normal * *ra,
                    })
                } else {
                    None
                }
            }
            (
                ColliderShape2d::Rect {
                    half_width: hw_a,
                    half_height: hh_a,
                },
                ColliderShape2d::Rect {
                    half_width: hw_b,
                    half_height: hh_b,
                },
            ) => {
                let ca = pos_a + self.offset;
                let cb = pos_b + other.offset;
                let dx = (cb.x - ca.x).abs();
                let dy = (cb.y - ca.y).abs();
                let overlap_x = hw_a + hw_b - dx;
                let overlap_y = hh_a + hh_b - dy;
                if overlap_x > 0.0 && overlap_y > 0.0 {
                    // 选择最小穿透轴
                    let (normal, penetration) = if overlap_x < overlap_y {
                        let nx = if cb.x > ca.x { 1.0 } else { -1.0 };
                        (Vec2::new(nx, 0.0), overlap_x)
                    } else {
                        let ny = if cb.y > ca.y { 1.0 } else { -1.0 };
                        (Vec2::new(0.0, ny), overlap_y)
                    };
                    Some(ContactInfo2d {
                        normal,
                        penetration,
                        contact_point: (ca + cb) * 0.5,
                    })
                } else {
                    None
                }
            }
            (
                ColliderShape2d::Circle { radius },
                ColliderShape2d::Rect {
                    half_width,
                    half_height,
                },
            ) => {
                let circle_pos = pos_a + self.offset;
                let rect_pos = pos_b + other.offset;
                // 将圆心变换到矩形局部空间
                let local = circle_pos - rect_pos;
                let clamped = Vec2::new(
                    local.x.clamp(-half_width, *half_width),
                    local.y.clamp(-half_height, *half_height),
                );
                let closest = rect_pos + clamped;
                let diff = circle_pos - closest;
                let dist_sq = diff.length_squared();
                if dist_sq < radius * radius {
                    let dist = dist_sq.sqrt();
                    let normal = if dist > 1e-6 { diff / dist } else { Vec2::Y };
                    Some(ContactInfo2d {
                        normal,
                        penetration: radius - dist,
                        contact_point: closest,
                    })
                } else {
                    None
                }
            }
            // 矩形 vs 圆形（对称处理）
            (ColliderShape2d::Rect { .. }, ColliderShape2d::Circle { .. }) => {
                other.intersects(pos_b, self, pos_a).map(|mut c| {
                    c.normal = -c.normal;
                    c
                })
            }
            _ => None, // 其他形状组合暂不支持
        }
    }

    /// 碰撞层掩码检测
    pub fn can_collide_with(&self, other: &Collider2d) -> bool {
        (self.collision_mask & other.collision_layer) != 0
    }
}

/// 2D 碰撞接触信息
#[derive(Debug, Clone)]
pub struct ContactInfo2d {
    /// 碰撞法线（从 A 指向 B）
    pub normal: Vec2,
    /// 穿透深度
    pub penetration: f32,
    /// 接触点（世界坐标）
    pub contact_point: Vec2,
}

/// 3D 碰撞体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider3d {
    pub body_id: u64,
    pub shape: ColliderShape3d,
    pub offset: Vec3,
    pub is_sensor: bool,
    pub collision_layer: u32,
    pub collision_mask: u32,
}

impl Collider3d {
    pub fn sphere(body_id: u64, radius: f32) -> Self {
        Self {
            body_id,
            shape: ColliderShape3d::Sphere { radius },
            offset: Vec3::ZERO,
            is_sensor: false,
            collision_layer: 1,
            collision_mask: 1,
        }
    }

    pub fn box_collider(body_id: u64, half_extents: Vec3) -> Self {
        Self {
            body_id,
            shape: ColliderShape3d::Box { half_extents },
            offset: Vec3::ZERO,
            is_sensor: false,
            collision_layer: 1,
            collision_mask: 1,
        }
    }
}
