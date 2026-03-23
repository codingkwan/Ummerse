//! 物理世界 - 管理物理仿真（带碰撞响应）

use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::{
    body::{RigidBody2d, RigidBody3d},
    collider::{Collider2d, Collider3d},
    CollisionEvent, CollisionEventKind, PhysicsMaterial,
};

/// 碰撞响应配置
#[derive(Debug, Clone)]
pub struct CollisionResponse {
    /// 是否启用位置修正（防止穿透）
    pub position_correction: bool,
    /// 位置修正比率（0.0 ~ 1.0，通常 0.2 ~ 0.8）
    pub baumgarte_factor: f32,
    /// 穿透容差（小于此值不进行修正）
    pub slop: f32,
    /// 最大物理步数/帧（防止死亡螺旋）
    pub max_steps: u32,
}

impl Default for CollisionResponse {
    fn default() -> Self {
        Self {
            position_correction: true,
            baumgarte_factor: 0.4,
            slop: 0.01,
            max_steps: 8,
        }
    }
}

/// 2D 物理世界
pub struct PhysicsWorld2d {
    pub gravity: Vec2,
    pub response: CollisionResponse,
    bodies: HashMap<u64, RigidBody2d>,
    colliders: HashMap<u64, Collider2d>,
    materials: HashMap<u64, PhysicsMaterial>,
    next_id: u64,
    /// 上一帧碰撞事件（用于触发 enter/exit 信号）
    active_contacts: HashMap<(u64, u64), f32>, // (body_a, body_b) -> penetration
}

impl PhysicsWorld2d {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, -980.0), // 像素/s² (约 9.8m/s²，1px=1cm)
            response: CollisionResponse::default(),
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            materials: HashMap::new(),
            next_id: 0,
            active_contacts: HashMap::new(),
        }
    }

    /// 分配新的 ID
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 添加刚体，返回其 ID
    pub fn add_body(&mut self, body: RigidBody2d) -> u64 {
        let id = body.id;
        self.bodies.insert(id, body);
        id
    }

    /// 添加碰撞体
    pub fn add_collider(&mut self, collider: Collider2d) {
        self.colliders.insert(collider.body_id, collider);
    }

    /// 添加物理材质
    pub fn set_material(&mut self, body_id: u64, material: PhysicsMaterial) {
        self.materials.insert(body_id, material);
    }

    /// 获取刚体
    pub fn get_body(&self, id: u64) -> Option<&RigidBody2d> {
        self.bodies.get(&id)
    }

    /// 获取可变刚体
    pub fn get_body_mut(&mut self, id: u64) -> Option<&mut RigidBody2d> {
        self.bodies.get_mut(&id)
    }

    /// 移除刚体及其碰撞体
    pub fn remove_body(&mut self, id: u64) {
        self.bodies.remove(&id);
        self.colliders.remove(&id);
        self.materials.remove(&id);
    }

    /// 设置重力
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    /// 应用冲量到刚体
    pub fn apply_impulse(&mut self, id: u64, impulse: Vec2) {
        if let Some(body) = self.bodies.get_mut(&id) {
            body.apply_impulse(impulse);
        }
    }

    /// 推进物理仿真一步（集成 + 碰撞检测 + 碰撞响应）
    pub fn step(&mut self, delta: f32) -> Vec<CollisionEvent> {
        // 1. 积分所有非静态刚体
        let gravity = self.gravity;
        for body in self.bodies.values_mut() {
            body.integrate(gravity, delta);
        }

        // 2. 宽相 + 窄相碰撞检测
        let mut events = Vec::new();
        let body_ids: Vec<u64> = self.bodies.keys().cloned().collect();
        let mut contacts = Vec::new();

        for i in 0..body_ids.len() {
            for j in (i + 1)..body_ids.len() {
                let id_a = body_ids[i];
                let id_b = body_ids[j];

                let (col_a, col_b) = match (self.colliders.get(&id_a), self.colliders.get(&id_b)) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };

                // 碰撞层检测
                if !col_a.can_collide_with(col_b) {
                    continue;
                }

                let (body_a, body_b) = match (self.bodies.get(&id_a), self.bodies.get(&id_b)) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };

                // 宽相 AABB 检测
                let aabb_a = col_a.aabb(body_a.position);
                let aabb_b = col_b.aabb(body_b.position);
                if !aabb_a.intersects(&aabb_b) {
                    continue;
                }

                // 窄相精确检测
                if let Some(contact) = col_a.intersects(body_a.position, col_b, body_b.position) {
                    contacts.push((id_a, id_b, contact));
                }
            }
        }

        // 3. 碰撞响应（冲量法）
        for (id_a, id_b, contact) in &contacts {
            let id_a = *id_a;
            let id_b = *id_b;

            let is_sensor_a = self
                .colliders
                .get(&id_a)
                .map(|c| c.is_sensor)
                .unwrap_or(false);
            let is_sensor_b = self
                .colliders
                .get(&id_b)
                .map(|c| c.is_sensor)
                .unwrap_or(false);

            // 传感器不产生物理响应
            if !is_sensor_a && !is_sensor_b {
                self.resolve_collision(id_a, id_b, contact);
            }

            // 判断是新碰撞还是持续碰撞
            let pair = if id_a < id_b {
                (id_a, id_b)
            } else {
                (id_b, id_a)
            };
            let kind = if self.active_contacts.contains_key(&pair) {
                CollisionEventKind::Persisted
            } else {
                CollisionEventKind::Entered
            };
            self.active_contacts.insert(pair, contact.penetration);

            events.push(CollisionEvent {
                body_a: id_a,
                body_b: id_b,
                contact_point: contact.contact_point.extend(0.0),
                contact_normal: contact.normal.extend(0.0),
                penetration: contact.penetration,
                kind,
            });
        }

        events
    }

    /// 冲量碰撞响应
    fn resolve_collision(
        &mut self,
        id_a: u64,
        id_b: u64,
        contact: &crate::collider::ContactInfo2d,
    ) {
        // 获取质量信息
        let (mass_a, inv_mass_a, is_static_a, vel_a) = if let Some(b) = self.bodies.get(&id_a) {
            let inv = if b.is_static { 0.0 } else { 1.0 / b.mass };
            (b.mass, inv, b.is_static, b.linear_velocity)
        } else {
            return;
        };

        let (mass_b, inv_mass_b, is_static_b, vel_b) = if let Some(b) = self.bodies.get(&id_b) {
            let inv = if b.is_static { 0.0 } else { 1.0 / b.mass };
            (b.mass, inv, b.is_static, b.linear_velocity)
        } else {
            return;
        };

        let _ = (mass_a, mass_b); // 抑制 unused 警告

        if is_static_a && is_static_b {
            return; // 两个静态物体不需要响应
        }

        // 获取弹性系数（取两者的平均）
        let restitution_a = self
            .materials
            .get(&id_a)
            .map(|m| m.restitution)
            .unwrap_or(0.0);
        let restitution_b = self
            .materials
            .get(&id_b)
            .map(|m| m.restitution)
            .unwrap_or(0.0);
        let restitution = (restitution_a + restitution_b) * 0.5;

        // 相对速度
        let rel_vel = vel_b - vel_a;
        let vel_along_normal = rel_vel.dot(contact.normal);

        // 如果物体正在分离，不需要响应
        if vel_along_normal > 0.0 {
            return;
        }

        // 计算冲量大小
        let j = -(1.0 + restitution) * vel_along_normal / (inv_mass_a + inv_mass_b);

        // 应用冲量
        let impulse = contact.normal * j;
        if let Some(body) = self.bodies.get_mut(&id_a) {
            if !body.is_static {
                body.linear_velocity -= impulse * inv_mass_a;
            }
        }
        if let Some(body) = self.bodies.get_mut(&id_b) {
            if !body.is_static {
                body.linear_velocity += impulse * inv_mass_b;
            }
        }

        // 位置修正（Baumgarte 稳定化）
        if self.response.position_correction {
            let correction_mag = (contact.penetration - self.response.slop).max(0.0)
                / (inv_mass_a + inv_mass_b)
                * self.response.baumgarte_factor;
            let correction = contact.normal * correction_mag;

            if let Some(body) = self.bodies.get_mut(&id_a) {
                if !body.is_static {
                    body.position -= correction * inv_mass_a;
                }
            }
            if let Some(body) = self.bodies.get_mut(&id_b) {
                if !body.is_static {
                    body.position += correction * inv_mass_b;
                }
            }
        }
    }

    /// 获取所有刚体的位置（用于渲染同步）
    pub fn body_positions(&self) -> Vec<(u64, Vec2)> {
        self.bodies
            .iter()
            .map(|(&id, b)| (id, b.position))
            .collect()
    }

    /// 射线检测（简化版 AABB 射线检测）
    pub fn raycast(&self, origin: Vec2, direction: Vec2, max_dist: f32) -> Option<RaycastHit2d> {
        let dir_norm = direction.normalize_or_zero();
        let mut closest: Option<RaycastHit2d> = None;

        for (&id, collider) in &self.colliders {
            if let Some(body) = self.bodies.get(&id) {
                let aabb = collider.aabb(body.position);
                if let Some(t) = aabb_raycast(origin, dir_norm, &aabb) {
                    if t <= max_dist {
                        if closest.as_ref().map(|c| t < c.distance).unwrap_or(true) {
                            closest = Some(RaycastHit2d {
                                body_id: id,
                                point: origin + dir_norm * t,
                                normal: Vec2::Y, // 简化
                                distance: t,
                            });
                        }
                    }
                }
            }
        }

        closest
    }

    /// 获取当前 next_id（兼容旧代码）
    pub fn next_id(&mut self) -> u64 {
        self.alloc_id()
    }
}

/// 简化的 AABB 射线检测（slab method）
fn aabb_raycast(origin: Vec2, dir: Vec2, aabb: &ummerse_math::aabb::Aabb2d) -> Option<f32> {
    let inv_dir = Vec2::new(
        if dir.x.abs() > 1e-8 {
            1.0 / dir.x
        } else {
            f32::INFINITY
        },
        if dir.y.abs() > 1e-8 {
            1.0 / dir.y
        } else {
            f32::INFINITY
        },
    );
    let t1 = (aabb.min - origin) * inv_dir;
    let t2 = (aabb.max - origin) * inv_dir;
    let t_min = t1.min(t2);
    let t_max = t1.max(t2);
    let t_enter = t_min.x.max(t_min.y);
    let t_exit = t_max.x.min(t_max.y);
    if t_enter <= t_exit && t_exit >= 0.0 {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

/// 2D 射线检测结果
#[derive(Debug, Clone)]
pub struct RaycastHit2d {
    pub body_id: u64,
    pub point: Vec2,
    pub normal: Vec2,
    pub distance: f32,
}

impl Default for PhysicsWorld2d {
    fn default() -> Self {
        Self::new()
    }
}

// ── 3D 物理世界 ────────────────────────────────────────────────────────────────

/// 3D 物理世界
pub struct PhysicsWorld3d {
    pub gravity: Vec3,
    bodies: HashMap<u64, RigidBody3d>,
    colliders: HashMap<u64, Collider3d>,
    next_id: u64,
}

impl PhysicsWorld3d {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.8, 0.0),
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_body(&mut self, body: RigidBody3d) -> u64 {
        let id = body.id;
        self.bodies.insert(id, body);
        id
    }

    pub fn add_collider(&mut self, collider: Collider3d) {
        self.colliders.insert(collider.body_id, collider);
    }

    pub fn get_body(&self, id: u64) -> Option<&RigidBody3d> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: u64) -> Option<&mut RigidBody3d> {
        self.bodies.get_mut(&id)
    }

    pub fn step(&mut self, delta: f32) -> Vec<CollisionEvent> {
        let gravity = self.gravity;
        for body in self.bodies.values_mut() {
            body.integrate(gravity, delta);
        }
        // TODO: 3D 碰撞检测（需要 GJK/EPA 算法）
        Vec::new()
    }

    pub fn body_positions(&self) -> Vec<(u64, Vec3)> {
        self.bodies
            .iter()
            .map(|(&id, b)| (id, b.position))
            .collect()
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl Default for PhysicsWorld3d {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::RigidBody2d;
    use crate::collider::Collider2d;

    #[test]
    fn test_gravity_integration() {
        let mut world = PhysicsWorld2d::new();
        world.gravity = Vec2::new(0.0, -9.8);
        let mut body = RigidBody2d::new(world.alloc_id());
        body.position = Vec2::new(0.0, 100.0);
        let id = world.add_body(body);
        world.step(1.0 / 60.0);
        let body = world.get_body(id).unwrap();
        // 重力应该使 y 速度减小（向下）
        assert!(body.linear_velocity.y < 0.0);
    }

    #[test]
    fn test_circle_collision() {
        let mut world = PhysicsWorld2d::new();
        world.gravity = Vec2::ZERO; // 关闭重力

        let id_a = world.alloc_id();
        let mut body_a = RigidBody2d::new(id_a);
        body_a.position = Vec2::new(0.0, 0.0);
        body_a.linear_velocity = Vec2::new(10.0, 0.0);
        world.add_body(body_a);
        world.add_collider(Collider2d::circle(id_a, 10.0));

        let id_b = world.alloc_id();
        let mut body_b = RigidBody2d::new(id_b);
        body_b.position = Vec2::new(15.0, 0.0); // 重叠，距离 15 < r_a+r_b = 20
        world.add_body(body_b);
        world.add_collider(Collider2d::circle(id_b, 10.0));

        let events = world.step(1.0 / 60.0);
        assert!(!events.is_empty(), "碰撞事件应该被检测到");
    }
}
