//! Bevy ECS 系统集合 - 连接引擎各子系统
//!
//! 定义 Ummerse 引擎的 ECS 组件、系统和插件。
//! 所有组件均可通过 Bevy Query 系统高效访问。

use bevy::prelude::*;

// ── 变换组件 ──────────────────────────────────────────────────────────────────

/// Ummerse 2D 变换组件
///
/// 桥接 Ummerse 坐标系与 Bevy Transform，支持通过 Changed 检测自动同步。
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct UmmerseTransform2d {
    /// 世界坐标（像素）
    pub position: Vec2,
    /// 旋转角度（弧度，逆时针为正）
    pub rotation: f32,
    /// 缩放（1.0 = 原始大小）
    pub scale: Vec2,
}

impl UmmerseTransform2d {
    /// 创建指定位置的变换
    #[inline]
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }

    /// 创建完整变换
    #[inline]
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self { position, rotation, scale }
    }
}

/// Ummerse 3D 变换组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct UmmerseTransform3d {
    /// 世界坐标
    pub position: Vec3,
    /// 旋转四元数
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
}

impl UmmerseTransform3d {
    /// 创建指定位置的变换
    #[inline]
    pub fn at(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Default for UmmerseTransform3d {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

// ── 通用节点组件 ──────────────────────────────────────────────────────────────

/// 节点名称组件（类 Godot 的 Node.name）
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct NodeName(pub String);

impl NodeName {
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 节点可见性组件
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct NodeVisible(pub bool);

impl NodeVisible {
    #[inline]
    pub fn visible() -> Self {
        Self(true)
    }

    #[inline]
    pub fn hidden() -> Self {
        Self(false)
    }
}

// ── 渲染组件 ──────────────────────────────────────────────────────────────────

/// 2D 精灵组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SpriteComponent {
    /// 纹理资产路径
    pub texture_path: String,
    /// 显示尺寸（像素）
    pub size: Vec2,
    /// 着色（与纹理颜色相乘）
    pub color: [f32; 4],
    /// 水平翻转
    pub flip_x: bool,
    /// 垂直翻转
    pub flip_y: bool,
    /// Z 排序层级（数值越大越靠前）
    pub z_index: i32,
}

impl SpriteComponent {
    pub fn new(texture_path: impl Into<String>, size: Vec2) -> Self {
        Self {
            texture_path: texture_path.into(),
            size,
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            z_index: 0,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    pub fn with_z(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }
}

/// 3D 网格实例组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MeshInstance3dComponent {
    /// 网格资产路径（gltf/obj 等）
    pub mesh_path: String,
    /// 材质资产路径（None 使用默认材质）
    pub material_path: Option<String>,
    /// 是否投射阴影
    pub cast_shadow: bool,
    /// 是否接受阴影
    pub receive_shadow: bool,
}

impl MeshInstance3dComponent {
    pub fn new(mesh_path: impl Into<String>) -> Self {
        Self {
            mesh_path: mesh_path.into(),
            material_path: None,
            cast_shadow: true,
            receive_shadow: true,
        }
    }

    pub fn with_material(mut self, path: impl Into<String>) -> Self {
        self.material_path = Some(path.into());
        self
    }
}

// ── 物理组件 ──────────────────────────────────────────────────────────────────

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum RigidBodyType {
    /// 动态刚体（受物理模拟）
    #[default]
    Dynamic,
    /// 静态刚体（固定不动）
    Static,
    /// 运动学刚体（手动控制）
    Kinematic,
}

/// 2D 刚体组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct RigidBody2dComponent {
    /// 刚体类型
    pub body_type: RigidBodyType,
    /// 线速度（像素/秒）
    pub velocity: Vec2,
    /// 角速度（弧度/秒）
    pub angular_velocity: f32,
    /// 质量（kg）
    pub mass: f32,
    /// 重力缩放（0 = 无重力，1 = 正常，负数 = 反重力）
    pub gravity_scale: f32,
    /// 线性阻尼（0 ~ 1）
    pub linear_damping: f32,
    /// 角阻尼（0 ~ 1）
    pub angular_damping: f32,
}

impl RigidBody2dComponent {
    /// 动态刚体（受重力）
    pub fn dynamic() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            mass: 1.0,
            gravity_scale: 1.0,
            linear_damping: 0.05,
            angular_damping: 0.05,
        }
    }

    /// 静态刚体（固定不动）
    pub fn static_body() -> Self {
        Self { body_type: RigidBodyType::Static, ..Self::dynamic() }
    }

    /// 运动学刚体（手动控制）
    pub fn kinematic() -> Self {
        Self { body_type: RigidBodyType::Kinematic, ..Self::dynamic() }
    }
}

/// 2D 碰撞体形状
#[derive(Debug, Clone, Reflect)]
pub enum ColliderShape {
    /// 圆形
    Circle { radius: f32 },
    /// 矩形（宽 × 高）
    Rect { width: f32, height: f32 },
    /// 胶囊（半径 + 高度）
    Capsule { radius: f32, height: f32 },
    /// 任意多边形
    Polygon { vertices: Vec<Vec2> },
}

/// 2D 碰撞体组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Collider2dComponent {
    /// 碰撞体形状
    pub shape: ColliderShape,
    /// 是否为传感器（Trigger，不产生碰撞响应）
    pub is_sensor: bool,
    /// 相对父节点的偏移
    pub offset: Vec2,
    /// 摩擦系数（0 ~ 1）
    pub friction: f32,
    /// 弹性系数（0 ~ 1）
    pub restitution: f32,
}

impl Collider2dComponent {
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Circle { radius },
            is_sensor: false,
            offset: Vec2::ZERO,
            friction: 0.5,
            restitution: 0.0,
        }
    }

    pub fn rect(width: f32, height: f32) -> Self {
        Self {
            shape: ColliderShape::Rect { width, height },
            is_sensor: false,
            offset: Vec2::ZERO,
            friction: 0.5,
            restitution: 0.0,
        }
    }

    pub fn sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }
}

// ── 相机组件 ──────────────────────────────────────────────────────────────────

/// 2D 相机组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Camera2dComponent {
    /// 缩放倍数（>1 缩小，<1 放大）
    pub zoom: f32,
    /// 是否为活跃相机
    pub is_active: bool,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
}

impl Default for Camera2dComponent {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            is_active: true,
            near: -1000.0,
            far: 1000.0,
        }
    }
}

/// 3D 相机组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Camera3dComponent {
    /// 垂直视野角（弧度，通常 60°）
    pub fov_y: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 是否为活跃相机
    pub is_active: bool,
}

impl Default for Camera3dComponent {
    fn default() -> Self {
        Self {
            fov_y: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            is_active: true,
        }
    }
}

// ── 脚本/音频组件 ─────────────────────────────────────────────────────────────

/// 脚本组件（Wasm 脚本附加到节点）
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ScriptComponent {
    /// 脚本资产路径（.wasm 文件）
    pub script_path: String,
    /// 是否已初始化
    pub initialized: bool,
}

impl ScriptComponent {
    pub fn new(script_path: impl Into<String>) -> Self {
        Self {
            script_path: script_path.into(),
            initialized: false,
        }
    }
}

/// 音频播放器组件
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AudioPlayerComponent {
    /// 音频资产路径
    pub audio_path: String,
    /// 音量（0.0 ~ 1.0）
    pub volume: f32,
    /// 音调（1.0 = 原始）
    pub pitch: f32,
    /// 是否循环播放
    pub looping: bool,
    /// 是否正在播放
    pub playing: bool,
    /// 是否为空间音频（3D 定位）
    pub spatial: bool,
}

impl AudioPlayerComponent {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            audio_path: path.into(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            playing: false,
            spatial: false,
        }
    }

    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn spatial(mut self) -> Self {
        self.spatial = true;
        self
    }
}

// ── Bevy 系统 ─────────────────────────────────────────────────────────────────

/// 将 [`UmmerseTransform2d`] 同步到 Bevy [`Transform`]
///
/// 仅在 UmmerseTransform2d 变更时触发（Changed 过滤）。
pub fn sync_transform_2d(
    mut query: Query<(&UmmerseTransform2d, &mut Transform), Changed<UmmerseTransform2d>>,
) {
    for (u_t, mut b_t) in query.iter_mut() {
        b_t.translation = Vec3::new(u_t.position.x, u_t.position.y, 0.0);
        b_t.rotation = Quat::from_rotation_z(u_t.rotation);
        b_t.scale = Vec3::new(u_t.scale.x, u_t.scale.y, 1.0);
    }
}

/// 将 [`UmmerseTransform3d`] 同步到 Bevy [`Transform`]
pub fn sync_transform_3d(
    mut query: Query<(&UmmerseTransform3d, &mut Transform), Changed<UmmerseTransform3d>>,
) {
    for (u_t, mut b_t) in query.iter_mut() {
        b_t.translation = u_t.position;
        b_t.rotation = u_t.rotation;
        b_t.scale = u_t.scale;
    }
}

/// 简单物理积分（占位实现，正式物理接入 rapier2d）
pub fn physics_step_2d(
    time: Res<Time>,
    mut query: Query<(&RigidBody2dComponent, &mut UmmerseTransform2d)>,
) {
    let dt = time.delta_secs();
    for (rb, mut transform) in query.iter_mut() {
        if rb.body_type != RigidBodyType::Dynamic {
            continue;
        }
        // 简单重力积分（仅演示，实际需 rapier2d）
        let gravity = Vec2::new(0.0, -9.8 * rb.gravity_scale);
        transform.position += rb.velocity * dt + gravity * (dt * dt * 0.5);
    }
}

/// 初始化未初始化的脚本组件
pub fn initialize_scripts(mut query: Query<(Entity, &mut ScriptComponent)>) {
    for (entity, mut script) in query.iter_mut() {
        if !script.initialized {
            tracing::debug!(
                entity = ?entity,
                path = %script.script_path,
                "Initializing script"
            );
            // TODO: 通过 Wasm 运行时加载脚本
            script.initialized = true;
        }
    }
}

/// 场景统计调试系统（仅 debug 级别日志）
pub fn debug_scene_stats(
    nodes: Query<&NodeName>,
    sprites: Query<&SpriteComponent>,
    meshes: Query<&MeshInstance3dComponent>,
    scripts: Query<&ScriptComponent>,
) {
    tracing::trace!(
        nodes = nodes.iter().count(),
        sprites = sprites.iter().count(),
        meshes = meshes.iter().count(),
        scripts = scripts.iter().count(),
        "Scene stats"
    );
}

// ── ECS 插件 ──────────────────────────────────────────────────────────────────

/// Ummerse 核心 ECS 系统插件
///
/// 注册所有引擎组件类型到 Bevy reflect 系统，
/// 并添加变换同步、物理步进等基础系统。
pub struct UmmerseCorePlugin;

impl Plugin for UmmerseCorePlugin {
    fn build(&self, app: &mut App) {
        app
            // ── 注册组件类型到 Reflect ────────────────────────────────
            .register_type::<UmmerseTransform2d>()
            .register_type::<UmmerseTransform3d>()
            .register_type::<NodeName>()
            .register_type::<NodeVisible>()
            .register_type::<SpriteComponent>()
            .register_type::<MeshInstance3dComponent>()
            .register_type::<RigidBody2dComponent>()
            .register_type::<Collider2dComponent>()
            .register_type::<Camera2dComponent>()
            .register_type::<Camera3dComponent>()
            .register_type::<ScriptComponent>()
            .register_type::<AudioPlayerComponent>()
            // ── 添加系统 ─────────────────────────────────────────────
            .add_systems(
                Update,
                (
                    sync_transform_2d,
                    sync_transform_3d,
                    initialize_scripts,
                ).chain(),
            )
            .add_systems(
                FixedUpdate,
                physics_step_2d,
            )
            .add_systems(Last, debug_scene_stats);

        tracing::info!("UmmerseCorePlugin initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform2d_at() {
        let t = UmmerseTransform2d::at(100.0, 200.0);
        assert_eq!(t.position, Vec2::new(100.0, 200.0));
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn test_collider_circle() {
        let c = Collider2dComponent::circle(32.0);
        assert!(!c.is_sensor);
        assert!(matches!(c.shape, ColliderShape::Circle { radius } if radius == 32.0));
    }

    #[test]
    fn test_rigid_body_types() {
        let dynamic = RigidBody2dComponent::dynamic();
        assert_eq!(dynamic.body_type, RigidBodyType::Dynamic);

        let static_body = RigidBody2dComponent::static_body();
        assert_eq!(static_body.body_type, RigidBodyType::Static);
    }
}
