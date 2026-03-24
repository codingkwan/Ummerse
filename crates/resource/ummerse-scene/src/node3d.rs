//! 3D 节点 - 带完整变换、光照和骨骼支持的 3D 场景节点

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use ummerse_core::node::{NodeId, NodeType};
use ummerse_math::transform::Transform3d;

/// 3D 节点 - 带 3D 变换的场景节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node3d {
    pub id: NodeId,
    pub name: String,
    /// 本地变换（相对于父节点）
    pub transform: Transform3d,
    /// 是否可见
    pub visible: bool,
    /// 是否启用（false 时跳过 process/physics_process）
    pub enabled: bool,
    /// 标签列表（用于分组查找）
    pub tags: Vec<String>,
}

impl Node3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            transform: Transform3d::IDENTITY,
            visible: true,
            enabled: true,
            tags: Vec::new(),
        }
    }

    pub fn node_type() -> NodeType {
        NodeType::Node3d
    }

    // ── 本地变换访问 ──────────────────────────────────────────────────────

    /// 本地位置
    #[inline]
    pub fn position(&self) -> Vec3 {
        self.transform.position
    }

    /// 设置本地位置
    #[inline]
    pub fn set_position(&mut self, pos: Vec3) {
        self.transform.position = pos;
    }

    /// 平移本地位置
    #[inline]
    pub fn translate(&mut self, delta: Vec3) {
        self.transform.position += delta;
    }

    /// 本地旋转（四元数）
    #[inline]
    pub fn rotation(&self) -> Quat {
        self.transform.rotation
    }

    /// 设置本地旋转
    #[inline]
    pub fn set_rotation(&mut self, rot: Quat) {
        self.transform.rotation = rot;
    }

    /// 围绕轴旋转（弧度）
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.transform.rotation = Quat::from_axis_angle(axis, angle) * self.transform.rotation;
    }

    /// 本地缩放
    #[inline]
    pub fn scale(&self) -> Vec3 {
        self.transform.scale
    }

    /// 设置本地缩放
    #[inline]
    pub fn set_scale(&mut self, scale: Vec3) {
        self.transform.scale = scale;
    }

    /// 统一缩放（XYZ 相同）
    #[inline]
    pub fn set_uniform_scale(&mut self, s: f32) {
        self.transform.scale = Vec3::splat(s);
    }

    // ── 方向向量 ──────────────────────────────────────────────────────────

    /// 本地前方向（-Z 轴）
    pub fn forward(&self) -> Vec3 {
        self.transform.forward()
    }

    /// 本地后方向（+Z 轴）
    pub fn back(&self) -> Vec3 {
        -self.transform.forward()
    }

    /// 本地右方向（+X 轴）
    pub fn right(&self) -> Vec3 {
        self.transform.right()
    }

    /// 本地左方向（-X 轴）
    pub fn left(&self) -> Vec3 {
        -self.transform.right()
    }

    /// 本地上方向（+Y 轴）
    pub fn up(&self) -> Vec3 {
        self.transform.up()
    }

    /// 本地下方向（-Y 轴）
    pub fn down(&self) -> Vec3 {
        -self.transform.up()
    }

    // ── 全局变换 ──────────────────────────────────────────────────────────

    /// 全局位置（无父节点时等于本地位置）
    pub fn global_position(&self) -> Vec3 {
        self.transform.position
    }

    /// 计算有父节点时的全局位置
    pub fn global_position_with_parent(&self, parent_global: &Transform3d) -> Vec3 {
        parent_global.rotation * (parent_global.scale * self.transform.position)
            + parent_global.position
    }

    // ── 朝向辅助 ──────────────────────────────────────────────────────────

    /// 面朝目标点（保持 up 向上，使用完整 look-at 矩阵）
    pub fn look_at(&mut self, target: Vec3) {
        let dir = (target - self.transform.position).normalize_or_zero();
        if dir.length_squared() < 1e-8 {
            return;
        }
        // 避免 gimbal lock：当 dir 与 Y 轴平行时使用 Z 为 up
        let up = if dir.y.abs() > 0.999 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        // 构造完整 look-at 旋转矩阵（右手坐标系，-Z 为前方）
        let forward = -dir; // 相机前方为 -Z
        let right = up.cross(forward).normalize_or_zero();
        if right.length_squared() < 1e-8 {
            return; // up 与 forward 平行，退化情况
        }
        let corrected_up = forward.cross(right).normalize_or_zero();
        // 从旋转矩阵列向量构造四元数
        use glam::Mat3;
        let rot_mat = Mat3::from_cols(right, corrected_up, forward);
        self.transform.rotation = Quat::from_mat3(&rot_mat).normalize();
    }

    /// 面朝目标点（使用自定义上方向）
    pub fn look_at_with_up(&mut self, target: Vec3, up: Vec3) {
        let dir = (target - self.transform.position).normalize_or_zero();
        if dir.length_squared() < 1e-8 {
            return;
        }
        let forward = -dir;
        let right = up.cross(forward).normalize_or_zero();
        if right.length_squared() < 1e-8 {
            return;
        }
        let corrected_up = forward.cross(right).normalize_or_zero();
        use glam::Mat3;
        let rot_mat = Mat3::from_cols(right, corrected_up, forward);
        self.transform.rotation = Quat::from_mat3(&rot_mat).normalize();
    }

    // ── 标签操作 ──────────────────────────────────────────────────────────

    /// 添加标签
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let t = tag.into();
        if !self.tags.contains(&t) {
            self.tags.push(t);
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// 是否含有指定标签
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

// ── 3D 网格实例节点 ───────────────────────────────────────────────────────────

/// 3D 网格实例节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshInstance3d {
    pub base: Node3d,
    /// 网格资产路径（.gltf/.glb/.obj）
    pub mesh_path: String,
    /// 材质覆盖路径（None = 使用网格内嵌材质）
    pub material_path: Option<String>,
    /// 是否投射阴影
    pub cast_shadow: bool,
    /// 是否接受阴影
    pub receive_shadow: bool,
    /// LOD 组（None = 不使用 LOD）
    pub lod_group: Option<u32>,
}

impl MeshInstance3d {
    pub fn new(name: impl Into<String>, mesh_path: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            mesh_path: mesh_path.into(),
            material_path: None,
            cast_shadow: true,
            receive_shadow: true,
            lod_group: None,
        }
    }

    pub fn with_material(mut self, path: impl Into<String>) -> Self {
        self.material_path = Some(path.into());
        self
    }
}

// ── 光源节点 ──────────────────────────────────────────────────────────────────

/// 光源颜色和强度
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LightColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    /// 光强度（lux 或任意单位）
    pub intensity: f32,
}

impl LightColor {
    pub fn white(intensity: f32) -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            intensity,
        }
    }

    pub fn rgb(r: f32, g: f32, b: f32, intensity: f32) -> Self {
        Self { r, g, b, intensity }
    }

    pub fn warm_white(intensity: f32) -> Self {
        Self {
            r: 1.0,
            g: 0.95,
            b: 0.8,
            intensity,
        }
    }
}

impl Default for LightColor {
    fn default() -> Self {
        Self::white(3.0)
    }
}

/// 方向光节点（全局平行光，如太阳光）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight3d {
    pub base: Node3d,
    pub color: LightColor,
    /// 是否产生阴影
    pub cast_shadow: bool,
    /// 阴影贴图大小（像素，通常 1024/2048/4096）
    pub shadow_map_size: u32,
    /// 阴影最大距离
    pub shadow_max_distance: f32,
}

impl DirectionalLight3d {
    pub fn new(name: impl Into<String>) -> Self {
        let mut base = Node3d::new(name);
        // 默认朝下偏向前方，类似午后阳光
        base.transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4);
        Self {
            base,
            color: LightColor::warm_white(3.0),
            cast_shadow: true,
            shadow_map_size: 2048,
            shadow_max_distance: 200.0,
        }
    }
}

/// 点光源节点（全向光）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLight3d {
    pub base: Node3d,
    pub color: LightColor,
    /// 光照范围（世界单位）
    pub range: f32,
    /// 衰减系数
    pub attenuation: f32,
    pub cast_shadow: bool,
}

impl PointLight3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            color: LightColor::white(1.0),
            range: 10.0,
            attenuation: 1.0,
            cast_shadow: false,
        }
    }

    /// 在指定位置创建点光
    pub fn at(name: impl Into<String>, pos: Vec3, intensity: f32, range: f32) -> Self {
        let mut light = Self::new(name);
        light.base.set_position(pos);
        light.color.intensity = intensity;
        light.range = range;
        light
    }
}

/// 聚光灯节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLight3d {
    pub base: Node3d,
    pub color: LightColor,
    /// 光照范围
    pub range: f32,
    /// 内锥角（弧度，无过渡区）
    pub inner_angle: f32,
    /// 外锥角（弧度，软边缘）
    pub outer_angle: f32,
    pub cast_shadow: bool,
}

impl SpotLight3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            color: LightColor::white(2.0),
            range: 20.0,
            inner_angle: 20.0_f32.to_radians(),
            outer_angle: 30.0_f32.to_radians(),
            cast_shadow: false,
        }
    }
}

// ── 3D 相机节点 ───────────────────────────────────────────────────────────────

/// 3D 相机节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera3dNode {
    pub base: Node3d,
    /// 垂直视野角（弧度）
    pub fov_y: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 是否为当前激活相机
    pub is_current: bool,
    /// 是否使用正交投影（false = 透视）
    pub orthographic: bool,
    /// 正交宽度（orthographic = true 时使用）
    pub ortho_size: f32,
}

impl Camera3dNode {
    pub fn perspective(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            fov_y: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            is_current: false,
            orthographic: false,
            ortho_size: 10.0,
        }
    }

    pub fn orthographic_camera(name: impl Into<String>, ortho_size: f32) -> Self {
        Self {
            base: Node3d::new(name),
            fov_y: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            is_current: false,
            orthographic: true,
            ortho_size,
        }
    }

    /// 激活此相机
    pub fn activate(&mut self) {
        self.is_current = true;
    }
}

// ── 粒子系统节点 ──────────────────────────────────────────────────────────────

/// 粒子发射模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmitterShape {
    /// 点发射
    Point,
    /// 球形发射
    Sphere,
    /// 盒形发射
    Box,
    /// 圆锥形发射
    Cone,
    /// 半球发射
    Hemisphere,
}

/// 3D 粒子系统节点（CPU 粒子）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem3d {
    pub base: Node3d,
    /// 是否正在发射
    pub emitting: bool,
    /// 每秒发射粒子数
    pub emission_rate: f32,
    /// 最大粒子数
    pub max_particles: u32,
    /// 粒子生命周期（秒）
    pub lifetime: f32,
    /// 生命周期随机范围
    pub lifetime_random: f32,
    /// 初始速度
    pub initial_velocity: f32,
    /// 速度随机范围
    pub velocity_random: f32,
    /// 重力倍率
    pub gravity_scale: f32,
    /// 发射器形状
    pub emitter_shape: EmitterShape,
    /// 初始大小
    pub initial_size: f32,
    /// 结束大小
    pub end_size: f32,
    /// 初始颜色（RGBA）
    pub initial_color: [f32; 4],
    /// 结束颜色（RGBA）
    pub end_color: [f32; 4],
    /// 粒子纹理路径
    pub texture_path: Option<String>,
}

impl ParticleSystem3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            emitting: true,
            emission_rate: 20.0,
            max_particles: 200,
            lifetime: 2.0,
            lifetime_random: 0.5,
            initial_velocity: 5.0,
            velocity_random: 2.0,
            gravity_scale: 1.0,
            emitter_shape: EmitterShape::Point,
            initial_size: 0.1,
            end_size: 0.0,
            initial_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            texture_path: None,
        }
    }

    /// 火焰预设
    pub fn fire(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            emitting: true,
            emission_rate: 50.0,
            max_particles: 300,
            lifetime: 1.5,
            lifetime_random: 0.3,
            initial_velocity: 3.0,
            velocity_random: 1.0,
            gravity_scale: -0.3, // 向上飘
            emitter_shape: EmitterShape::Cone,
            initial_size: 0.2,
            end_size: 0.0,
            initial_color: [1.0, 0.6, 0.1, 1.0],
            end_color: [0.5, 0.1, 0.0, 0.0],
            texture_path: None,
        }
    }

    /// 烟雾预设
    pub fn smoke(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            emitting: true,
            emission_rate: 15.0,
            max_particles: 100,
            lifetime: 4.0,
            lifetime_random: 1.0,
            initial_velocity: 1.0,
            velocity_random: 0.5,
            gravity_scale: -0.1,
            emitter_shape: EmitterShape::Sphere,
            initial_size: 0.3,
            end_size: 1.5,
            initial_color: [0.5, 0.5, 0.5, 0.6],
            end_color: [0.8, 0.8, 0.8, 0.0],
            texture_path: None,
        }
    }
}

// ── 3D 刚体节点 ───────────────────────────────────────────────────────────────

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType3d {
    /// 动态（受物理模拟）
    Dynamic,
    /// 静态（固定不动）
    Static,
    /// 运动学（手动控制）
    Kinematic,
}

/// 3D 刚体节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody3dNode {
    pub base: Node3d,
    pub body_type: RigidBodyType3d,
    pub mass: f32,
    pub gravity_scale: f32,
    /// 线速度（世界空间，m/s）
    pub linear_velocity: Vec3,
    /// 角速度（世界空间，rad/s）
    pub angular_velocity: Vec3,
    /// 线性阻尼
    pub linear_damping: f32,
    /// 角阻尼
    pub angular_damping: f32,
    /// 是否可旋转（false = 锁定旋转，适合角色）
    pub can_rotate: bool,
}

impl RigidBody3dNode {
    pub fn dynamic(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            body_type: RigidBodyType3d::Dynamic,
            mass: 1.0,
            gravity_scale: 1.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            linear_damping: 0.05,
            angular_damping: 0.05,
            can_rotate: true,
        }
    }

    pub fn static_body(name: impl Into<String>) -> Self {
        Self {
            body_type: RigidBodyType3d::Static,
            ..Self::dynamic(name)
        }
    }

    pub fn kinematic(name: impl Into<String>) -> Self {
        Self {
            body_type: RigidBodyType3d::Kinematic,
            ..Self::dynamic(name)
        }
    }

    /// 施加冲量（世界空间）
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == RigidBodyType3d::Dynamic && self.mass > 0.0 {
            self.linear_velocity += impulse / self.mass;
        }
    }
}

// ── 3D 角色控制器节点 ─────────────────────────────────────────────────────────

/// 3D 角色控制器节点（类 Godot CharacterBody3D）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBody3d {
    pub base: Node3d,
    /// 当前速度（世界空间，m/s）
    pub velocity: Vec3,
    /// 上方向参考（通常为 Vec3::Y）
    pub up_direction: Vec3,
    /// 是否在地面上
    pub is_on_floor: bool,
    /// 是否在墙壁上
    pub is_on_wall: bool,
    /// 是否在天花板上
    pub is_on_ceiling: bool,
    /// 地面法线
    pub floor_normal: Vec3,
    /// 最大可攀爬坡度角（弧度）
    pub floor_max_angle: f32,
    /// 碰撞层
    pub collision_layer: u32,
    /// 碰撞掩码
    pub collision_mask: u32,
    /// 重力缩放（1.0 = 正常）
    pub gravity_scale: f32,
}

impl CharacterBody3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            velocity: Vec3::ZERO,
            up_direction: Vec3::Y,
            is_on_floor: false,
            is_on_wall: false,
            is_on_ceiling: false,
            floor_normal: Vec3::Y,
            floor_max_angle: 46.0_f32.to_radians(),
            collision_layer: 1,
            collision_mask: 1,
            gravity_scale: 1.0,
        }
    }

    /// 移动并与地形碰撞（简化积分，完整版需物理引擎）
    pub fn move_and_slide(&mut self, gravity: Vec3, delta: f32) {
        if !self.is_on_floor {
            self.velocity += gravity * self.gravity_scale * delta;
        }
        self.base.translate(self.velocity * delta);
    }

    /// 跳跃（仅在地面时生效）
    pub fn jump(&mut self, jump_velocity: f32) -> bool {
        if self.is_on_floor {
            self.velocity.y = jump_velocity;
            self.is_on_floor = false;
            true
        } else {
            false
        }
    }

    /// 设置水平移动速度（保留 Y 速度）
    pub fn set_horizontal_velocity(&mut self, horizontal: Vec3) {
        self.velocity.x = horizontal.x;
        self.velocity.z = horizontal.z;
    }

    /// 获取水平移动速度
    pub fn horizontal_velocity(&self) -> Vec3 {
        Vec3::new(self.velocity.x, 0.0, self.velocity.z)
    }
}

// ── 3D 静态物体节点 ───────────────────────────────────────────────────────────

/// 3D 静态物体节点（不受物理影响的静态碰撞体）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticBody3d {
    pub base: Node3d,
    /// 碰撞层
    pub collision_layer: u32,
    /// 碰撞掩码
    pub collision_mask: u32,
    /// 物理材质摩擦系数
    pub friction: f32,
    /// 物理材质弹性
    pub restitution: f32,
    /// 平台运动速度（传送带/移动平台）
    pub constant_linear_velocity: Vec3,
    /// 平台旋转速度
    pub constant_angular_velocity: Vec3,
}

impl StaticBody3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            collision_layer: 1,
            collision_mask: 1,
            friction: 0.6,
            restitution: 0.0,
            constant_linear_velocity: Vec3::ZERO,
            constant_angular_velocity: Vec3::ZERO,
        }
    }

    /// 创建移动平台预设
    pub fn moving_platform(name: impl Into<String>, velocity: Vec3) -> Self {
        let mut body = Self::new(name);
        body.constant_linear_velocity = velocity;
        body
    }
}

// ── 3D 碰撞形状节点 ───────────────────────────────────────────────────────────

/// 3D 碰撞形状定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape3dDef {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Capsule { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
    ConvexHull { mesh_path: String },
    ConcaveMesh { mesh_path: String },
}

/// 3D 碰撞形状节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionShape3dNode {
    pub base: Node3d,
    pub shape: Shape3dDef,
    pub disabled: bool,
}

impl CollisionShape3dNode {
    pub fn sphere(name: impl Into<String>, radius: f32) -> Self {
        Self {
            base: Node3d::new(name),
            shape: Shape3dDef::Sphere { radius },
            disabled: false,
        }
    }

    pub fn box_shape(name: impl Into<String>, half_extents: Vec3) -> Self {
        Self {
            base: Node3d::new(name),
            shape: Shape3dDef::Box { half_extents },
            disabled: false,
        }
    }

    pub fn capsule(name: impl Into<String>, radius: f32, height: f32) -> Self {
        Self {
            base: Node3d::new(name),
            shape: Shape3dDef::Capsule { radius, height },
            disabled: false,
        }
    }
}

// ── 3D Area 节点 ──────────────────────────────────────────────────────────────

/// 3D 区域节点（传感器/触发区域）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area3d {
    pub base: Node3d,
    pub collision_layer: u32,
    pub collision_mask: u32,
    pub monitoring: bool,
    pub monitorable: bool,
    /// 重力方向覆盖
    pub gravity_dir: Vec3,
    /// 重力强度（-1 = 使用世界重力）
    pub gravity: f32,
    /// 线性阻尼覆盖（-1 = 不覆盖）
    pub linear_damp: f32,
    /// 角阻尼覆盖（-1 = 不覆盖）
    pub angular_damp: f32,
    /// 区域内重叠物体（运行时）
    #[serde(skip)]
    pub overlapping_bodies: Vec<u64>,
}

impl Area3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            collision_layer: 1,
            collision_mask: 1,
            monitoring: true,
            monitorable: true,
            gravity_dir: Vec3::NEG_Y,
            gravity: -1.0,
            linear_damp: -1.0,
            angular_damp: -1.0,
            overlapping_bodies: Vec::new(),
        }
    }

    pub fn has_body(&self, id: u64) -> bool {
        self.overlapping_bodies.contains(&id)
    }
}

// ── 3D 射线检测节点 ───────────────────────────────────────────────────────────

/// 3D 射线检测节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RayCast3dNode {
    pub base: Node3d,
    /// 射线终点（相对于本地坐标）
    pub target_position: Vec3,
    pub enabled: bool,
    pub collision_mask: u32,
    pub exclude_parent: bool,
    /// 上帧检测结果
    pub is_colliding: bool,
    pub collision_point: Vec3,
    pub collision_normal: Vec3,
    pub collider_id: Option<u64>,
}

impl RayCast3dNode {
    pub fn new(name: impl Into<String>, target: Vec3) -> Self {
        Self {
            base: Node3d::new(name),
            target_position: target,
            enabled: true,
            collision_mask: u32::MAX,
            exclude_parent: true,
            is_colliding: false,
            collision_point: Vec3::ZERO,
            collision_normal: Vec3::ZERO,
            collider_id: None,
        }
    }
}

// ── 3D 环境天空盒 ─────────────────────────────────────────────────────────────

/// 天空盒类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkyBackground {
    /// 纯色天空
    Color { color: [f32; 4] },
    /// 纯色渐变（顶部/底部）
    Gradient {
        top_color: [f32; 4],
        bottom_color: [f32; 4],
        curve: f32,
    },
    /// HDR 全景天空贴图
    Panorama { texture_path: String },
    /// 程序天空（大气散射）
    Procedural {
        /// 太阳方向
        sun_dir: Vec3,
        /// 瑞利散射强度
        rayleigh: f32,
        /// 米氏散射强度
        mie: f32,
        /// 大气厚度
        sky_top_color: [f32; 4],
        sky_horizon_color: [f32; 4],
    },
}

impl Default for SkyBackground {
    fn default() -> Self {
        Self::Procedural {
            sun_dir: Vec3::new(0.2, 0.8, 0.3).normalize(),
            rayleigh: 0.035,
            mie: 0.003,
            sky_top_color: [0.2, 0.4, 0.8, 1.0],
            sky_horizon_color: [0.6, 0.8, 1.0, 1.0],
        }
    }
}

/// 色调映射模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TonemapMode {
    /// 线性（无映射）
    Linear,
    /// ACES Film（Filmic 效果）
    Aces,
    /// Filmic（软高光）
    Filmic,
    /// AgX（Blender 4.0 默认）
    Agx,
}

/// 3D 世界环境节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEnvironment3d {
    pub base: Node3d,
    /// 天空背景
    pub sky: SkyBackground,
    /// 环境光颜色
    pub ambient_light_color: [f32; 4],
    /// 环境光强度
    pub ambient_light_energy: f32,
    /// 曝光值（EV100）
    pub tonemap_exposure: f32,
    /// 色调映射模式
    pub tonemap_mode: TonemapMode,
    /// 雾效是否启用
    pub fog_enabled: bool,
    /// 雾效颜色
    pub fog_color: [f32; 4],
    /// 雾效起始距离
    pub fog_near: f32,
    /// 雾效最远距离
    pub fog_far: f32,
    /// 雾效密度
    pub fog_density: f32,
}

impl WorldEnvironment3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            sky: SkyBackground::default(),
            ambient_light_color: [0.2, 0.2, 0.25, 1.0],
            ambient_light_energy: 0.5,
            tonemap_exposure: 1.0,
            tonemap_mode: TonemapMode::Aces,
            fog_enabled: false,
            fog_color: [0.7, 0.7, 0.7, 1.0],
            fog_near: 10.0,
            fog_far: 200.0,
            fog_density: 0.01,
        }
    }

    /// 白天户外预设
    pub fn outdoor_day(name: impl Into<String>) -> Self {
        Self::new(name)
    }

    /// 夜晚预设
    pub fn night(name: impl Into<String>) -> Self {
        let mut env = Self::new(name);
        env.sky = SkyBackground::Color {
            color: [0.02, 0.02, 0.05, 1.0],
        };
        env.ambient_light_color = [0.05, 0.05, 0.1, 1.0];
        env.ambient_light_energy = 0.1;
        env
    }

    /// 带雾霾预设
    pub fn foggy(name: impl Into<String>) -> Self {
        let mut env = Self::new(name);
        env.fog_enabled = true;
        env.fog_near = 5.0;
        env.fog_far = 80.0;
        env.fog_density = 0.05;
        env
    }
}

// ── 骨骼动画 ──────────────────────────────────────────────────────────────────

/// 骨骼关节（骨头）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    pub name: String,
    /// 骨头索引
    pub index: u32,
    /// 父骨头索引（None = 根骨头）
    pub parent: Option<u32>,
    /// 静息姿势（局部变换）
    pub rest_position: Vec3,
    pub rest_rotation: Quat,
    pub rest_scale: Vec3,
}

impl Bone {
    pub fn new(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index,
            parent: None,
            rest_position: Vec3::ZERO,
            rest_rotation: Quat::IDENTITY,
            rest_scale: Vec3::ONE,
        }
    }
}

/// 骨骼帧关键帧（单个骨头）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneKeyframe {
    /// 时间点（秒）
    pub time: f32,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

/// 骨骼动画轨道（每个骨头一条轨道）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneTrack {
    /// 骨头名称
    pub bone_name: String,
    /// 关键帧列表（按时间排序）
    pub keyframes: Vec<BoneKeyframe>,
}

impl BoneTrack {
    pub fn new(bone_name: impl Into<String>) -> Self {
        Self {
            bone_name: bone_name.into(),
            keyframes: Vec::new(),
        }
    }

    /// 在指定时间采样（线性插值）
    pub fn sample(&self, time: f32) -> Option<(Vec3, Quat, Vec3)> {
        if self.keyframes.is_empty() {
            return None;
        }
        // 找到前后关键帧
        let after_idx = self.keyframes.partition_point(|k| k.time <= time);
        if after_idx == 0 {
            let k = &self.keyframes[0];
            return Some((k.position, k.rotation, k.scale));
        }
        if after_idx >= self.keyframes.len() {
            let k = &self.keyframes[self.keyframes.len() - 1];
            return Some((k.position, k.rotation, k.scale));
        }
        let k0 = &self.keyframes[after_idx - 1];
        let k1 = &self.keyframes[after_idx];
        let t = if (k1.time - k0.time).abs() < 1e-6 {
            0.0
        } else {
            (time - k0.time) / (k1.time - k0.time)
        };
        let pos = k0.position.lerp(k1.position, t);
        let rot = k0.rotation.slerp(k1.rotation, t);
        let scale = k0.scale.lerp(k1.scale, t);
        Some((pos, rot, scale))
    }
}

/// 骨骼动画剪辑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletalAnimation {
    pub name: String,
    /// 总时长（秒）
    pub duration: f32,
    /// 是否循环
    pub looping: bool,
    /// 播放速率（1.0 = 正常速度）
    pub playback_rate: f32,
    /// 各骨头的动画轨道
    pub tracks: Vec<BoneTrack>,
}

impl SkeletalAnimation {
    pub fn new(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            looping: true,
            playback_rate: 1.0,
            tracks: Vec::new(),
        }
    }

    /// 在指定时间采样所有骨头状态
    pub fn sample_all(&self, time: f32) -> Vec<(&str, Vec3, Quat, Vec3)> {
        self.tracks
            .iter()
            .filter_map(|track| {
                track
                    .sample(time)
                    .map(|(pos, rot, scale)| (track.bone_name.as_str(), pos, rot, scale))
            })
            .collect()
    }
}

/// 骨骼网格节点（带骨骼动画的 MeshInstance3d）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletalMesh3d {
    pub base: Node3d,
    /// 网格资产路径
    pub mesh_path: String,
    /// 材质路径
    pub material_path: Option<String>,
    /// 骨骼定义
    pub skeleton: Vec<Bone>,
    /// 动画剪辑集合
    pub animations: Vec<SkeletalAnimation>,
    /// 当前播放的动画名称
    pub current_animation: Option<String>,
    /// 当前播放时间
    pub current_time: f32,
    /// 是否正在播放
    pub playing: bool,
    /// 是否投射阴影
    pub cast_shadow: bool,
}

impl SkeletalMesh3d {
    pub fn new(name: impl Into<String>, mesh_path: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            mesh_path: mesh_path.into(),
            material_path: None,
            skeleton: Vec::new(),
            animations: Vec::new(),
            current_animation: None,
            current_time: 0.0,
            playing: false,
            cast_shadow: true,
        }
    }

    /// 播放指定动画
    pub fn play(&mut self, name: &str) {
        if self.current_animation.as_deref() != Some(name) {
            self.current_animation = Some(name.to_string());
            self.current_time = 0.0;
        }
        self.playing = true;
    }

    /// 停止动画
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// 每帧更新动画时间
    pub fn update(&mut self, delta: f32) {
        if !self.playing {
            return;
        }
        let anim = self
            .current_animation
            .as_ref()
            .and_then(|name| self.animations.iter().find(|a| &a.name == name));

        if let Some(anim) = anim {
            self.current_time += delta * anim.playback_rate;
            if self.current_time > anim.duration {
                if anim.looping {
                    self.current_time %= anim.duration.max(0.0001);
                } else {
                    self.current_time = anim.duration;
                    self.playing = false;
                }
            }
        }
    }

    /// 获取当前帧的骨骼姿势（用于 GPU skinning）
    pub fn current_pose(&self) -> Vec<(&str, Vec3, Quat, Vec3)> {
        let Some(name) = &self.current_animation else {
            return Vec::new();
        };
        let Some(anim) = self.animations.iter().find(|a| &a.name == name) else {
            return Vec::new();
        };
        anim.sample_all(self.current_time)
    }
}

// ── 动画播放器节点 ────────────────────────────────────────────────────────────

/// 通用动画播放器节点（类 Godot AnimationPlayer）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPlayer3d {
    pub base: Node3d,
    /// 动画库（名称 → 动画资产路径）
    pub animations: std::collections::HashMap<String, String>,
    /// 当前播放的动画
    pub current_animation: Option<String>,
    /// 当前播放时间
    pub current_time: f32,
    /// 是否正在播放
    pub playing: bool,
    /// 播放速率
    pub playback_rate: f32,
    /// 是否自动播放（进入场景时自动播放 autoplay 动画）
    pub autoplay: Option<String>,
}

impl AnimationPlayer3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            animations: std::collections::HashMap::new(),
            current_animation: None,
            current_time: 0.0,
            playing: false,
            playback_rate: 1.0,
            autoplay: None,
        }
    }

    pub fn add_animation(&mut self, name: impl Into<String>, path: impl Into<String>) {
        self.animations.insert(name.into(), path.into());
    }

    pub fn play(&mut self, name: &str) {
        if self.animations.contains_key(name) {
            self.current_animation = Some(name.to_string());
            self.current_time = 0.0;
            self.playing = true;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }
}

// ── 3D 导航网格代理节点 ───────────────────────────────────────────────────────

/// 3D 导航代理节点（用于路径寻路）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationAgent3d {
    pub base: Node3d,
    /// 目标位置
    pub target_position: Vec3,
    /// 最大移动速度（m/s）
    pub max_speed: f32,
    /// 最大加速度（m/s²）
    pub max_acceleration: f32,
    /// 到达目标的容差距离
    pub path_desired_distance: f32,
    /// 目标到达容差
    pub target_desired_distance: f32,
    /// 是否已到达目标
    pub is_navigation_finished: bool,
    /// 当前路径（世界坐标点列表）
    #[serde(skip)]
    pub current_path: Vec<Vec3>,
}

impl NavigationAgent3d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node3d::new(name),
            target_position: Vec3::ZERO,
            max_speed: 5.0,
            max_acceleration: 20.0,
            path_desired_distance: 1.0,
            target_desired_distance: 0.5,
            is_navigation_finished: false,
            current_path: Vec::new(),
        }
    }

    /// 获取下一个路径点
    pub fn next_path_position(&self) -> Option<Vec3> {
        self.current_path.first().copied()
    }

    /// 标记当前路径点已到达
    pub fn advance_path(&mut self) {
        if !self.current_path.is_empty() {
            self.current_path.remove(0);
            self.is_navigation_finished = self.current_path.is_empty();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node3d_translate() {
        let mut node = Node3d::new("test");
        node.translate(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(node.position(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_node3d_rotate_axis() {
        let mut node = Node3d::new("test");
        node.rotate_axis(Vec3::Y, std::f32::consts::FRAC_PI_2);
        // 绕 Y 轴旋转 90° 后 right 应近似 (0, 0, -1)
        let right = node.right();
        assert!(
            (right.z + 1.0).abs() < 1e-5,
            "right.z should be ~-1, got {}",
            right.z
        );
    }

    #[test]
    fn test_directional_light_default() {
        let light = DirectionalLight3d::new("sun");
        assert!(light.cast_shadow);
        assert_eq!(light.shadow_map_size, 2048);
    }

    #[test]
    fn test_particle_system_fire() {
        let fire = ParticleSystem3d::fire("fire");
        assert!(fire.emitting);
        assert!(fire.gravity_scale < 0.0, "Fire particles should float up");
    }

    #[test]
    fn test_rigid_body_impulse() {
        let mut body = RigidBody3dNode::dynamic("box");
        body.apply_impulse(Vec3::new(10.0, 0.0, 0.0));
        // mass = 1.0, impulse / mass = 10.0
        assert!((body.linear_velocity.x - 10.0).abs() < 1e-6);
    }
}
