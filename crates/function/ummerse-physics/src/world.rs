//! 物理世界 - 管理物理仿真（带碰撞响应）

use std::collections::HashMap;

use glam::{Vec2, Vec3};

use crate::{
    CollisionEvent, CollisionEventKind, PhysicsMaterial,
    body::{RigidBody2d, RigidBody3d},
    collider::{Collider2d, Collider3d},
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
#[allow(missing_debug_implementations)]
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

/// 3D 射线检测结果
#[derive(Debug, Clone)]
pub struct RaycastHit3d {
    pub body_id: u64,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

/// 3D 物理世界
#[allow(missing_debug_implementations)]
pub struct PhysicsWorld3d {
    pub gravity: Vec3,
    bodies: HashMap<u64, RigidBody3d>,
    colliders: HashMap<u64, Collider3d>,
    next_id: u64,
    /// 上一帧碰撞接触对（用于 Entered/Exited 事件）
    active_contacts: HashMap<(u64, u64), f32>,
}

impl PhysicsWorld3d {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.8, 0.0),
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            next_id: 0,
            active_contacts: HashMap::new(),
        }
    }

    /// 分配新的物体 ID
    pub fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_body(&mut self, body: RigidBody3d) -> u64 {
        let id = body.id;
        self.bodies.insert(id, body);
        id
    }

    pub fn add_collider(&mut self, collider: Collider3d) {
        self.colliders.insert(collider.body_id, collider);
    }

    /// 移除刚体及其碰撞体
    pub fn remove_body(&mut self, id: u64) {
        self.bodies.remove(&id);
        self.colliders.remove(&id);
    }

    pub fn get_body(&self, id: u64) -> Option<&RigidBody3d> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: u64) -> Option<&mut RigidBody3d> {
        self.bodies.get_mut(&id)
    }

    /// 施加冲量到刚体
    pub fn apply_impulse(&mut self, id: u64, impulse: Vec3) {
        if let Some(body) = self.bodies.get_mut(&id) {
            body.apply_force(impulse, 1.0);
        }
    }

    /// 推进物理步（重力积分 + 球体/AABB 碰撞检测）
    pub fn step(&mut self, delta: f32) -> Vec<CollisionEvent> {
        // 1. 积分所有非静态刚体
        let gravity = self.gravity;
        for body in self.bodies.values_mut() {
            body.integrate(gravity, delta);
        }

        // 2. 宽相 + 窄相碰撞检测（球体 vs 球体，AABB vs AABB）
        let mut events = Vec::new();
        let body_ids: Vec<u64> = self.bodies.keys().cloned().collect();

        for i in 0..body_ids.len() {
            for j in (i + 1)..body_ids.len() {
                let id_a = body_ids[i];
                let id_b = body_ids[j];

                let (col_a, col_b) =
                    match (self.colliders.get(&id_a), self.colliders.get(&id_b)) {
                        (Some(a), Some(b)) => (a, b),
                        _ => continue,
                    };

                // 碰撞层掩码检测
                if (col_a.collision_mask & col_b.collision_layer) == 0 {
                    continue;
                }

                let (body_a, body_b) =
                    match (self.bodies.get(&id_a), self.bodies.get(&id_b)) {
                        (Some(a), Some(b)) => (a, b),
                        _ => continue,
                    };

                // 两个静态体无需检测
                if body_a.is_static && body_b.is_static {
                    continue;
                }

                // 窄相碰撞检测
                if let Some((contact_point, normal, penetration)) =
                    test_3d_collision(col_a, body_a.position, col_b, body_b.position)
                {
                    // 碰撞响应（非传感器）
                    if !col_a.is_sensor && !col_b.is_sensor {
                        resolve_3d_collision(
                            &mut self.bodies,
                            id_a,
                            id_b,
                            normal,
                            penetration,
                        );
                    }

                    let pair = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };
                    let kind = if self.active_contacts.contains_key(&pair) {
                        CollisionEventKind::Persisted
                    } else {
                        CollisionEventKind::Entered
                    };
                    self.active_contacts.insert(pair, penetration);

                    events.push(CollisionEvent {
                        body_a: id_a,
                        body_b: id_b,
                        contact_point,
                        contact_normal: normal,
                        penetration,
                        kind,
                    });
                }
            }
        }

        // 3. 清除已分离的接触对（产生 Exited 事件）
        let current_pairs: std::collections::HashSet<(u64, u64)> = events
            .iter()
            .map(|e| {
                if e.body_a < e.body_b {
                    (e.body_a, e.body_b)
                } else {
                    (e.body_b, e.body_a)
                }
            })
            .collect();

        let exited_pairs: Vec<(u64, u64)> = self
            .active_contacts
            .keys()
            .filter(|pair| !current_pairs.contains(pair))
            .copied()
            .collect();

        for pair in exited_pairs {
            self.active_contacts.remove(&pair);
            events.push(CollisionEvent {
                body_a: pair.0,
                body_b: pair.1,
                contact_point: Vec3::ZERO,
                contact_normal: Vec3::Y,
                penetration: 0.0,
                kind: CollisionEventKind::Exited,
            });
        }

        events
    }

    /// 射线检测（AABB 近似，球体精确）
    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_dist: f32) -> Option<RaycastHit3d> {
        let dir_norm = direction.normalize_or_zero();
        let mut closest: Option<RaycastHit3d> = None;

        for (&id, collider) in &self.colliders {
            if let Some(body) = self.bodies.get(&id) {
                let hit = match &collider.shape {
                    crate::collider::ColliderShape3d::Sphere { radius } => {
                        sphere_raycast(origin, dir_norm, body.position + collider.offset, *radius)
                    }
                    crate::collider::ColliderShape3d::Box { half_extents } => {
                        let center = body.position + collider.offset;
                        aabb3d_raycast(origin, dir_norm, center - *half_extents, center + *half_extents)
                            .map(|(t, n)| (t, n))
                    }
                    crate::collider::ColliderShape3d::Capsule { radius, height } => {
                        // 用球体近似
                        let r = radius + height * 0.5;
                        sphere_raycast(origin, dir_norm, body.position + collider.offset, r)
                    }
                    crate::collider::ColliderShape3d::Cylinder { radius, height } => {
                        let r = radius.max(height * 0.5);
                        sphere_raycast(origin, dir_norm, body.position + collider.offset, r)
                    }
                };

                if let Some((t, normal)) = hit {
                    if t <= max_dist {
                        if closest.as_ref().map(|c| t < c.distance).unwrap_or(true) {
                            closest = Some(RaycastHit3d {
                                body_id: id,
                                point: origin + dir_norm * t,
                                normal,
                                distance: t,
                            });
                        }
                    }
                }
            }
        }

        closest
    }

    /// 获取所有刚体位置（用于渲染同步）
    pub fn body_positions(&self) -> Vec<(u64, Vec3)> {
        self.bodies
            .iter()
            .map(|(&id, b)| (id, b.position))
            .collect()
    }

    pub fn next_id(&mut self) -> u64 {
        self.alloc_id()
    }
}

impl Default for PhysicsWorld3d {
    fn default() -> Self {
        Self::new()
    }
}

// ── 3D 碰撞检测辅助函数 ───────────────────────────────────────────────────────

/// 3D 碰撞检测（返回接触点、法线、穿透深度）
fn test_3d_collision(
    col_a: &Collider3d,
    pos_a: Vec3,
    col_b: &Collider3d,
    pos_b: Vec3,
) -> Option<(Vec3, Vec3, f32)> {
    let ca = pos_a + col_a.offset;
    let cb = pos_b + col_b.offset;

    match (&col_a.shape, &col_b.shape) {
        // 球 vs 球
        (
            crate::collider::ColliderShape3d::Sphere { radius: ra },
            crate::collider::ColliderShape3d::Sphere { radius: rb },
        ) => {
            let diff = cb - ca;
            let dist_sq = diff.length_squared();
            let combined = ra + rb;
            if dist_sq < combined * combined {
                let dist = dist_sq.sqrt();
                let normal = if dist > 1e-6 { diff / dist } else { Vec3::Y };
                let penetration = combined - dist;
                let contact = ca + normal * *ra;
                Some((contact, normal, penetration))
            } else {
                None
            }
        }
        // AABB vs AABB
        (
            crate::collider::ColliderShape3d::Box { half_extents: he_a },
            crate::collider::ColliderShape3d::Box { half_extents: he_b },
        ) => {
            let dx = (cb.x - ca.x).abs();
            let dy = (cb.y - ca.y).abs();
            let dz = (cb.z - ca.z).abs();
            let ox = he_a.x + he_b.x - dx;
            let oy = he_a.y + he_b.y - dy;
            let oz = he_a.z + he_b.z - dz;
            if ox > 0.0 && oy > 0.0 && oz > 0.0 {
                // 最小穿透轴
                let (normal, penetration) = if ox <= oy && ox <= oz {
                    let nx = if cb.x > ca.x { 1.0_f32 } else { -1.0_f32 };
                    (Vec3::new(nx, 0.0, 0.0), ox)
                } else if oy <= ox && oy <= oz {
                    let ny = if cb.y > ca.y { 1.0_f32 } else { -1.0_f32 };
                    (Vec3::new(0.0, ny, 0.0), oy)
                } else {
                    let nz = if cb.z > ca.z { 1.0_f32 } else { -1.0_f32 };
                    (Vec3::new(0.0, 0.0, nz), oz)
                };
                Some(((ca + cb) * 0.5, normal, penetration))
            } else {
                None
            }
        }
        // 球 vs AABB
        (
            crate::collider::ColliderShape3d::Sphere { radius },
            crate::collider::ColliderShape3d::Box { half_extents },
        ) => {
            let local = ca - cb;
            let clamped = local.clamp(-*half_extents, *half_extents);
            let closest = cb + clamped;
            let diff = ca - closest;
            let dist_sq = diff.length_squared();
            if dist_sq < radius * radius {
                let dist = dist_sq.sqrt();
                let normal = if dist > 1e-6 { diff / dist } else { Vec3::Y };
                Some((closest, normal, radius - dist))
            } else {
                None
            }
        }
        // AABB vs 球（对称）
        (
            crate::collider::ColliderShape3d::Box { .. },
            crate::collider::ColliderShape3d::Sphere { .. },
        ) => test_3d_collision(col_b, pos_b, col_a, pos_a)
            .map(|(p, n, pen)| (p, -n, pen)),
        // 其他形状组合：用球体近似
        _ => {
            let ra = shape_approx_radius(&col_a.shape);
            let rb = shape_approx_radius(&col_b.shape);
            let diff = cb - ca;
            let dist_sq = diff.length_squared();
            let combined = ra + rb;
            if dist_sq < combined * combined {
                let dist = dist_sq.sqrt();
                let normal = if dist > 1e-6 { diff / dist } else { Vec3::Y };
                Some((ca + normal * ra, normal, combined - dist))
            } else {
                None
            }
        }
    }
}

/// 获取形状的近似球半径（用于宽相和非精确碰撞）
fn shape_approx_radius(shape: &crate::collider::ColliderShape3d) -> f32 {
    match shape {
        crate::collider::ColliderShape3d::Sphere { radius } => *radius,
        crate::collider::ColliderShape3d::Box { half_extents } => half_extents.length(),
        crate::collider::ColliderShape3d::Capsule { radius, height } => radius + height * 0.5,
        crate::collider::ColliderShape3d::Cylinder { radius, height } => {
            radius.max(height * 0.5)
        }
    }
}

/// 3D 冲量碰撞响应
fn resolve_3d_collision(
    bodies: &mut HashMap<u64, RigidBody3d>,
    id_a: u64,
    id_b: u64,
    normal: Vec3,
    penetration: f32,
) {
    let (inv_mass_a, static_a, vel_a) = match bodies.get(&id_a) {
        Some(b) => (
            if b.is_static { 0.0 } else { 1.0 / b.mass },
            b.is_static,
            b.linear_velocity,
        ),
        None => return,
    };
    let (inv_mass_b, static_b, vel_b) = match bodies.get(&id_b) {
        Some(b) => (
            if b.is_static { 0.0 } else { 1.0 / b.mass },
            b.is_static,
            b.linear_velocity,
        ),
        None => return,
    };

    if static_a && static_b {
        return;
    }

    let rel_vel = vel_b - vel_a;
    let vel_along = rel_vel.dot(normal);
    if vel_along > 0.0 {
        return; // 正在分离
    }

    let restitution = 0.3_f32;
    let j = -(1.0 + restitution) * vel_along / (inv_mass_a + inv_mass_b + 1e-8);
    let impulse = normal * j;

    if let Some(body) = bodies.get_mut(&id_a) {
        if !body.is_static {
            body.linear_velocity -= impulse * inv_mass_a;
        }
    }
    if let Some(body) = bodies.get_mut(&id_b) {
        if !body.is_static {
            body.linear_velocity += impulse * inv_mass_b;
        }
    }

    // Baumgarte 位置修正
    let correction_mag =
        (penetration - 0.01_f32).max(0.0) / (inv_mass_a + inv_mass_b + 1e-8) * 0.4;
    let correction = normal * correction_mag;
    if let Some(body) = bodies.get_mut(&id_a) {
        if !body.is_static {
            body.position -= correction * inv_mass_a;
        }
    }
    if let Some(body) = bodies.get_mut(&id_b) {
        if !body.is_static {
            body.position += correction * inv_mass_b;
        }
    }
}

/// 球体射线检测（返回 (t, 法线)）
fn sphere_raycast(
    origin: Vec3,
    dir: Vec3,
    center: Vec3,
    radius: f32,
) -> Option<(f32, Vec3)> {
    let oc = origin - center;
    let b = oc.dot(dir);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return None;
    }
    let sqrt_d = discriminant.sqrt();
    let t = -b - sqrt_d;
    let t = if t >= 0.0 { t } else { -b + sqrt_d };
    if t >= 0.0 {
        let hit_point = origin + dir * t;
        let normal = (hit_point - center).normalize_or_zero();
        Some((t, normal))
    } else {
        None
    }
}

/// AABB3D 射线检测（slab method，返回 (t, 法线)）
fn aabb3d_raycast(
    origin: Vec3,
    dir: Vec3,
    aabb_min: Vec3,
    aabb_max: Vec3,
) -> Option<(f32, Vec3)> {
    let inv_dir = Vec3::new(
        if dir.x.abs() > 1e-8 { 1.0 / dir.x } else { f32::INFINITY },
        if dir.y.abs() > 1e-8 { 1.0 / dir.y } else { f32::INFINITY },
        if dir.z.abs() > 1e-8 { 1.0 / dir.z } else { f32::INFINITY },
    );
    let t1 = (aabb_min - origin) * inv_dir;
    let t2 = (aabb_max - origin) * inv_dir;
    let t_min_v = t1.min(t2);
    let t_max_v = t1.max(t2);
    let t_enter = t_min_v.x.max(t_min_v.y).max(t_min_v.z);
    let t_exit = t_max_v.x.min(t_max_v.y).min(t_max_v.z);

    if t_enter <= t_exit && t_exit >= 0.0 {
        let t = t_enter.max(0.0);
        // 计算命中面法线
        let center = (aabb_min + aabb_max) * 0.5;
        let hit = origin + dir * t;
        let local = (hit - center) / ((aabb_max - aabb_min) * 0.5);
        let abs = local.abs();
        let normal = if abs.x >= abs.y && abs.x >= abs.z {
            Vec3::new(local.x.signum(), 0.0, 0.0)
        } else if abs.y >= abs.x && abs.y >= abs.z {
            Vec3::new(0.0, local.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, local.z.signum())
        };
        Some((t, normal))
    } else {
        None
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
