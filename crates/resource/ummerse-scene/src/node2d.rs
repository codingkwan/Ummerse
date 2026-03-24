//! 2D 节点 - 带完整变换、父子关系和可见性继承的 2D 场景节点

use glam::Vec2;
use serde::{Deserialize, Serialize};
use ummerse_core::node::{NodeId, NodeType};
use ummerse_math::transform::Transform2d;

/// 2D 节点 - 带 2D 变换的场景节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node2d {
    pub id: NodeId,
    pub name: String,
    /// 本地变换（相对于父节点）
    pub transform: Transform2d,
    /// Z 排序索引
    pub z_index: i32,
    /// Z 是否相对于父节点（true = z_index 叠加父节点）
    pub z_as_relative: bool,
    /// 可见性（本地）
    pub visible: bool,
    /// 是否处理（enabled = false 则跳过 process/physics_process）
    pub enabled: bool,
    /// 标签列表（用于分组查找）
    pub tags: Vec<String>,
}

impl Node2d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            transform: Transform2d::IDENTITY,
            z_index: 0,
            z_as_relative: true,
            visible: true,
            enabled: true,
            tags: Vec::new(),
        }
    }

    pub fn node_type() -> NodeType {
        NodeType::Node2d
    }

    // ── 本地变换访问 ──────────────────────────────────────────────────────

    /// 本地位置
    #[inline]
    pub fn position(&self) -> Vec2 {
        self.transform.position
    }

    /// 设置本地位置
    #[inline]
    pub fn set_position(&mut self, pos: Vec2) {
        self.transform.position = pos;
    }

    /// 平移本地位置
    #[inline]
    pub fn translate(&mut self, delta: Vec2) {
        self.transform.position += delta;
    }

    /// 本地旋转（弧度）
    #[inline]
    pub fn rotation(&self) -> f32 {
        self.transform.rotation
    }

    /// 设置本地旋转（弧度）
    #[inline]
    pub fn set_rotation(&mut self, angle: f32) {
        self.transform.rotation = angle;
    }

    /// 旋转增量（弧度）
    #[inline]
    pub fn rotate(&mut self, angle: f32) {
        self.transform.rotation += angle;
    }

    /// 本地缩放
    #[inline]
    pub fn scale(&self) -> Vec2 {
        self.transform.scale
    }

    /// 设置本地缩放
    #[inline]
    pub fn set_scale(&mut self, scale: Vec2) {
        self.transform.scale = scale;
    }

    // ── 全局变换（需要父节点的全局变换）────────────────────────────────────

    /// 计算全局位置（给定父节点全局变换）
    pub fn global_position_with_parent(&self, parent_global: &Transform2d) -> Vec2 {
        parent_global
            .to_affine2()
            .transform_point2(self.transform.position)
    }

    /// 全局位置（无父节点时等于本地位置）
    pub fn global_position(&self) -> Vec2 {
        self.transform.position
    }

    /// 计算有效 Z 索引（考虑父节点叠加）
    pub fn effective_z_index(&self, parent_z: i32) -> i32 {
        if self.z_as_relative {
            parent_z + self.z_index
        } else {
            self.z_index
        }
    }

    // ── 可见性 ────────────────────────────────────────────────────────────

    /// 根据父节点继承可见性
    pub fn inherited_visibility(&self, parent_visible: bool) -> bool {
        self.visible && parent_visible
    }

    // ── 朝向辅助 ──────────────────────────────────────────────────────────

    /// 朝向目标点（设置旋转使正 Y 轴指向目标）
    pub fn look_at(&mut self, target: Vec2) {
        let dir = target - self.transform.position;
        self.transform.rotation = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;
    }

    /// 本地 X 轴方向（right vector）
    pub fn right(&self) -> Vec2 {
        let cos = self.transform.rotation.cos();
        let sin = self.transform.rotation.sin();
        Vec2::new(cos, sin)
    }

    /// 本地 Y 轴方向（up vector）
    pub fn up(&self) -> Vec2 {
        let cos = self.transform.rotation.cos();
        let sin = self.transform.rotation.sin();
        Vec2::new(-sin, cos)
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

// ── 2D 精灵节点 ───────────────────────────────────────────────────────────────

/// 2D 精灵节点（带纹理的 Node2d）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite2d {
    pub base: Node2d,
    /// 纹理资产路径
    pub texture_path: String,
    /// 着色（RGBA，与纹理相乘）
    pub color: [f32; 4],
    /// 水平翻转
    pub flip_x: bool,
    /// 垂直翻转
    pub flip_y: bool,
    /// 显示尺寸（None = 纹理原始大小）
    pub size: Option<Vec2>,
    /// UV 裁切（None = 使用整个纹理）
    pub region: Option<[f32; 4]>, // [x, y, width, height]
    /// 中心偏移（相对于本地原点）
    pub offset: Vec2,
}

impl Sprite2d {
    pub fn new(name: impl Into<String>, texture_path: impl Into<String>) -> Self {
        Self {
            base: Node2d::new(name),
            texture_path: texture_path.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            flip_x: false,
            flip_y: false,
            size: None,
            region: None,
            offset: Vec2::ZERO,
        }
    }

    /// 设置着色颜色
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// 设置显示尺寸
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some(Vec2::new(width, height));
        self
    }
}

// ── 2D 相机节点 ───────────────────────────────────────────────────────────────

/// 2D 相机节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera2dNode {
    pub base: Node2d,
    /// 缩放（1.0 = 1:1，>1 缩小场景，<1 放大场景）
    pub zoom: f32,
    /// 是否为当前激活相机
    pub is_current: bool,
    /// 相机限制区域（世界坐标，None = 无限制）
    pub limit: Option<[f32; 4]>, // [left, top, right, bottom]
    /// 跟随平滑系数（0 = 立即，1 = 不移动）
    pub smoothing: f32,
}

impl Camera2dNode {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node2d::new(name),
            zoom: 1.0,
            is_current: false,
            limit: None,
            smoothing: 0.0,
        }
    }

    /// 设置为当前相机
    pub fn activate(&mut self) {
        self.is_current = true;
    }
}

// ── 动画精灵节点 ──────────────────────────────────────────────────────────────

/// 动画帧信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    /// 纹理 UV 区域 [x, y, width, height]（相对于 sprite sheet）
    pub region: [f32; 4],
    /// 帧持续时间（秒）
    pub duration: f32,
}

/// 动画定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAnimation {
    pub name: String,
    pub frames: Vec<AnimationFrame>,
    pub looping: bool,
    /// 帧率（frames per second，优先于每帧 duration）
    pub fps: Option<f32>,
}

impl SpriteAnimation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frames: Vec::new(),
            looping: true,
            fps: Some(12.0),
        }
    }

    pub fn frame_duration(&self) -> f32 {
        if let Some(fps) = self.fps {
            1.0 / fps.max(0.001)
        } else if let Some(first) = self.frames.first() {
            first.duration
        } else {
            1.0 / 12.0
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

/// 动画精灵节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedSprite2d {
    pub base: Node2d,
    pub texture_path: String,
    pub color: [f32; 4],
    /// 动画集合（名称 → 动画）
    pub animations: Vec<SpriteAnimation>,
    /// 当前动画名称
    pub current_animation: Option<String>,
    /// 当前帧索引
    pub current_frame: usize,
    /// 当前帧已过去的时间
    pub frame_timer: f32,
    /// 是否正在播放
    pub playing: bool,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl AnimatedSprite2d {
    pub fn new(name: impl Into<String>, texture_path: impl Into<String>) -> Self {
        Self {
            base: Node2d::new(name),
            texture_path: texture_path.into(),
            color: [1.0, 1.0, 1.0, 1.0],
            animations: Vec::new(),
            current_animation: None,
            current_frame: 0,
            frame_timer: 0.0,
            playing: false,
            flip_x: false,
            flip_y: false,
        }
    }

    /// 添加动画
    pub fn add_animation(&mut self, anim: SpriteAnimation) {
        self.animations.push(anim);
    }

    /// 播放指定动画
    pub fn play(&mut self, name: &str) {
        if self.current_animation.as_deref() != Some(name) {
            self.current_animation = Some(name.to_string());
            self.current_frame = 0;
            self.frame_timer = 0.0;
        }
        self.playing = true;
    }

    /// 停止动画
    pub fn stop(&mut self) {
        self.playing = false;
    }

    /// 更新动画（每帧调用）
    pub fn update(&mut self, delta: f32) {
        if !self.playing {
            return;
        }
        let anim = self
            .current_animation
            .as_ref()
            .and_then(|name| self.animations.iter().find(|a| &a.name == name));

        if let Some(anim) = anim {
            let frame_dur = anim.frame_duration();
            self.frame_timer += delta;
            while self.frame_timer >= frame_dur {
                self.frame_timer -= frame_dur;
                self.current_frame += 1;
                if self.current_frame >= anim.frame_count() {
                    if anim.looping {
                        self.current_frame = 0;
                    } else {
                        self.current_frame = anim.frame_count().saturating_sub(1);
                        self.playing = false;
                        break;
                    }
                }
            }
        }
    }

    /// 当前帧的 UV 区域（若无动画则返回全纹理）
    pub fn current_region(&self) -> Option<[f32; 4]> {
        let name = self.current_animation.as_ref()?;
        let anim = self.animations.iter().find(|a| &a.name == name)?;
        anim.frames.get(self.current_frame).map(|f| f.region)
    }
}

// ── TileMap 节点 ───────────────────────────────────────────────────────────────

/// TileSet 中的单个 Tile 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDef {
    /// Tile ID（在 TileSet 中的索引）
    pub id: u32,
    /// 纹理 UV 区域 [x, y, width, height]（像素坐标）
    pub region: [f32; 4],
    /// 是否可通行（false = 固体碰撞）
    pub passable: bool,
    /// 自定义属性（JSON）
    pub properties: serde_json::Value,
}

impl TileDef {
    pub fn new(id: u32, region: [f32; 4]) -> Self {
        Self {
            id,
            region,
            passable: true,
            properties: serde_json::Value::Null,
        }
    }
}

/// TileSet - Tile 定义集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileSet {
    /// TileSet 名称
    pub name: String,
    /// Tile 图集纹理路径
    pub texture_path: String,
    /// Tile 宽度（像素）
    pub tile_width: u32,
    /// Tile 高度（像素）
    pub tile_height: u32,
    /// 所有 Tile 定义
    pub tiles: Vec<TileDef>,
}

impl TileSet {
    pub fn new(
        name: impl Into<String>,
        texture_path: impl Into<String>,
        tile_width: u32,
        tile_height: u32,
    ) -> Self {
        Self {
            name: name.into(),
            texture_path: texture_path.into(),
            tile_width,
            tile_height,
            tiles: Vec::new(),
        }
    }

    /// 自动从 sprite sheet 生成 Tile 定义（行 × 列）
    pub fn from_sprite_sheet(
        name: impl Into<String>,
        texture_path: impl Into<String>,
        tile_width: u32,
        tile_height: u32,
        columns: u32,
        rows: u32,
    ) -> Self {
        let mut tileset = Self::new(name, texture_path, tile_width, tile_height);
        let tw = tile_width as f32;
        let th = tile_height as f32;
        for row in 0..rows {
            for col in 0..columns {
                let id = row * columns + col;
                let region = [col as f32 * tw, row as f32 * th, tw, th];
                tileset.tiles.push(TileDef::new(id, region));
            }
        }
        tileset
    }

    /// 根据 ID 获取 Tile 定义
    pub fn get(&self, id: u32) -> Option<&TileDef> {
        self.tiles.iter().find(|t| t.id == id)
    }
}

/// TileMap 节点 - 基于 Tile 的地图渲染
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMap {
    pub base: Node2d,
    /// 关联的 TileSet
    pub tileset: Option<TileSet>,
    /// 地图数据：(cell_x, cell_y) -> tile_id
    pub cells: std::collections::HashMap<(i32, i32), u32>,
    /// 单元格宽度（像素）
    pub cell_width: u32,
    /// 单元格高度（像素）
    pub cell_height: u32,
    /// 是否显示网格（编辑器用）
    pub show_grid: bool,
}

impl TileMap {
    pub fn new(name: impl Into<String>, cell_width: u32, cell_height: u32) -> Self {
        Self {
            base: Node2d::new(name),
            tileset: None,
            cells: std::collections::HashMap::new(),
            cell_width,
            cell_height,
            show_grid: false,
        }
    }

    /// 设置指定格子的 Tile
    pub fn set_cell(&mut self, x: i32, y: i32, tile_id: u32) {
        self.cells.insert((x, y), tile_id);
    }

    /// 清除指定格子
    pub fn clear_cell(&mut self, x: i32, y: i32) {
        self.cells.remove(&(x, y));
    }

    /// 获取指定格子的 Tile ID
    pub fn get_cell(&self, x: i32, y: i32) -> Option<u32> {
        self.cells.get(&(x, y)).copied()
    }

    /// 将世界坐标转换为格子坐标
    pub fn world_to_cell(&self, world_pos: Vec2) -> (i32, i32) {
        let origin = self.base.position();
        let local = world_pos - origin;
        let cx = (local.x / self.cell_width as f32).floor() as i32;
        let cy = (local.y / self.cell_height as f32).floor() as i32;
        (cx, cy)
    }

    /// 将格子坐标转换为世界坐标（格子左上角）
    pub fn cell_to_world(&self, cx: i32, cy: i32) -> Vec2 {
        let origin = self.base.position();
        origin + Vec2::new(cx as f32 * self.cell_width as f32, cy as f32 * self.cell_height as f32)
    }

    /// 地图尺寸（最大 - 最小格子范围）
    pub fn bounds(&self) -> Option<((i32, i32), (i32, i32))> {
        if self.cells.is_empty() {
            return None;
        }
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for &(x, y) in self.cells.keys() {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        Some(((min_x, min_y), (max_x, max_y)))
    }

    /// 填充矩形区域
    pub fn fill_rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, tile_id: u32) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.cells.insert((x, y), tile_id);
            }
        }
    }
}

// ── 2D Area 节点 ──────────────────────────────────────────────────────────────

/// 2D 区域节点（传感器区域，用于检测重叠）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Area2d {
    pub base: Node2d,
    /// 碰撞层
    pub collision_layer: u32,
    /// 碰撞掩码
    pub collision_mask: u32,
    /// 是否监控进入/离开事件
    pub monitoring: bool,
    /// 是否可被监控
    pub monitorable: bool,
    /// 区域内重叠的物体 ID 列表（运行时状态）
    #[serde(skip)]
    pub overlapping_bodies: Vec<u64>,
    /// 重力方向（用于区域内重力覆盖）
    pub gravity_dir: Vec2,
    /// 重力倍率（1.0 = 使用世界重力）
    pub gravity_scale: f32,
    /// 线性阻尼覆盖（-1 = 不覆盖）
    pub linear_damp: f32,
}

impl Area2d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node2d::new(name),
            collision_layer: 1,
            collision_mask: 1,
            monitoring: true,
            monitorable: true,
            overlapping_bodies: Vec::new(),
            gravity_dir: Vec2::new(0.0, -1.0),
            gravity_scale: 1.0,
            linear_damp: -1.0,
        }
    }

    /// 创建触发器区域（只检测重叠不影响物理）
    pub fn trigger(name: impl Into<String>) -> Self {
        let mut area = Self::new(name);
        area.monitorable = false;
        area
    }

    /// 判断某物体是否在区域内
    pub fn has_body(&self, body_id: u64) -> bool {
        self.overlapping_bodies.contains(&body_id)
    }

    /// 添加重叠物体（运行时调用）
    pub fn add_overlap(&mut self, body_id: u64) {
        if !self.overlapping_bodies.contains(&body_id) {
            self.overlapping_bodies.push(body_id);
        }
    }

    /// 移除重叠物体（运行时调用）
    pub fn remove_overlap(&mut self, body_id: u64) {
        self.overlapping_bodies.retain(|&id| id != body_id);
    }
}

// ── 2D 角色控制器节点 ─────────────────────────────────────────────────────────

/// 角色控制器运动模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterMotionMode {
    /// 浮动模式（3D 风格，无上限）
    Floating,
    /// 朝向地面（自动吸附地面，适合平台跳跃）
    Grounded,
}

/// 2D 角色控制器节点（类 Godot CharacterBody2D）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBody2d {
    pub base: Node2d,
    /// 当前速度（每帧由游戏逻辑设置）
    pub velocity: Vec2,
    /// 最大步高（可爬过的台阶高度）
    pub floor_max_angle: f32,
    /// 地面碰撞法线（仅在 is_on_floor 时有效）
    pub floor_normal: Vec2,
    /// 是否在地面上
    pub is_on_floor: bool,
    /// 是否在墙壁上
    pub is_on_wall: bool,
    /// 是否在天花板上
    pub is_on_ceiling: bool,
    /// 运动模式
    pub motion_mode: CharacterMotionMode,
    /// 上方向（地面法线参考方向，通常为 Vec2::Y）
    pub up_direction: Vec2,
    /// 碰撞层
    pub collision_layer: u32,
    /// 碰撞掩码
    pub collision_mask: u32,
    /// 地面速度（在地面移动时的参考速度）
    pub floor_velocity: Vec2,
}

impl CharacterBody2d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node2d::new(name),
            velocity: Vec2::ZERO,
            floor_max_angle: 46.0_f32.to_radians(),
            floor_normal: Vec2::Y,
            is_on_floor: false,
            is_on_wall: false,
            is_on_ceiling: false,
            motion_mode: CharacterMotionMode::Grounded,
            up_direction: Vec2::Y,
            collision_layer: 1,
            collision_mask: 1,
            floor_velocity: Vec2::ZERO,
        }
    }

    /// 移动并与地形碰撞（简化实现，完整版需物理引擎支持）
    pub fn move_and_slide(&mut self, gravity: Vec2, delta: f32) {
        // 应用重力（仅在空中时）
        if !self.is_on_floor {
            self.velocity += gravity * delta;
        }
        // 简单位置积分（实际碰撞响应需物理引擎）
        self.base.translate(self.velocity * delta);
    }

    /// 模拟跳跃（仅在地面时生效）
    pub fn jump(&mut self, jump_velocity: f32) -> bool {
        if self.is_on_floor {
            self.velocity.y = jump_velocity;
            self.is_on_floor = false;
            true
        } else {
            false
        }
    }
}

// ── 2D 碰撞形状节点 ───────────────────────────────────────────────────────────

/// 2D 碰撞形状（挂接到物理节点下）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shape2dDef {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
    Capsule { radius: f32, height: f32 },
    Segment { from: Vec2, to: Vec2 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionShape2dNode {
    pub base: Node2d,
    pub shape: Shape2dDef,
    /// 是否禁用此碰撞形状
    pub disabled: bool,
    /// 是否仅为单面碰撞（仅对线段有效）
    pub one_way: bool,
}

impl CollisionShape2dNode {
    pub fn circle(name: impl Into<String>, radius: f32) -> Self {
        Self {
            base: Node2d::new(name),
            shape: Shape2dDef::Circle { radius },
            disabled: false,
            one_way: false,
        }
    }

    pub fn rect(name: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            base: Node2d::new(name),
            shape: Shape2dDef::Rect { width, height },
            disabled: false,
            one_way: false,
        }
    }

    pub fn capsule(name: impl Into<String>, radius: f32, height: f32) -> Self {
        Self {
            base: Node2d::new(name),
            shape: Shape2dDef::Capsule { radius, height },
            disabled: false,
            one_way: false,
        }
    }
}

// ── 2D 射线节点 ───────────────────────────────────────────────────────────────

/// 2D 射线检测节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RayCast2dNode {
    pub base: Node2d,
    /// 射线终点（相对于节点本地坐标）
    pub target_position: Vec2,
    /// 是否已启用
    pub enabled: bool,
    /// 碰撞掩码
    pub collision_mask: u32,
    /// 是否排除父节点的碰撞体
    pub exclude_parent: bool,
    /// 上帧检测结果
    pub is_colliding: bool,
    /// 碰撞点（世界坐标）
    pub collision_point: Vec2,
    /// 碰撞法线
    pub collision_normal: Vec2,
    /// 命中对象 ID
    pub collider_id: Option<u64>,
}

impl RayCast2dNode {
    pub fn new(name: impl Into<String>, target: Vec2) -> Self {
        Self {
            base: Node2d::new(name),
            target_position: target,
            enabled: true,
            collision_mask: u32::MAX,
            exclude_parent: true,
            is_colliding: false,
            collision_point: Vec2::ZERO,
            collision_normal: Vec2::ZERO,
            collider_id: None,
        }
    }
}

// ── 2D 粒子系统 ───────────────────────────────────────────────────────────────

/// 2D CPU 粒子实例（运行时状态）
#[derive(Debug, Clone)]
pub struct Particle2d {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub elapsed: f32,
    pub size: f32,
    pub color: [f32; 4],
    pub rotation: f32,
    pub angular_velocity: f32,
}

/// 2D 粒子系统节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuParticles2d {
    pub base: Node2d,
    /// 是否正在发射
    pub emitting: bool,
    /// 每秒发射粒子数
    pub emission_rate: f32,
    /// 最大粒子数
    pub max_particles: u32,
    /// 粒子生命周期（秒）
    pub lifetime: f32,
    /// 生命周期随机范围（±lifetime_random * lifetime）
    pub lifetime_random: f32,
    /// 初始速度
    pub initial_velocity: f32,
    /// 速度随机范围
    pub velocity_random: f32,
    /// 发射方向角度（弧度，相对于节点朝向）
    pub spread: f32,
    /// 重力（像素/s²）
    pub gravity: Vec2,
    /// 初始大小（像素）
    pub initial_size: f32,
    /// 结束大小（像素）
    pub end_size: f32,
    /// 初始颜色
    pub initial_color: [f32; 4],
    /// 结束颜色
    pub end_color: [f32; 4],
    /// 粒子纹理路径
    pub texture_path: Option<String>,
    /// 运行时粒子实例（跳过序列化）
    #[serde(skip)]
    pub particles: Vec<Particle2d>,
    /// 发射计时器（累积时间）
    #[serde(skip)]
    emission_timer: f32,
}

impl CpuParticles2d {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: Node2d::new(name),
            emitting: true,
            emission_rate: 20.0,
            max_particles: 100,
            lifetime: 2.0,
            lifetime_random: 0.5,
            initial_velocity: 50.0,
            velocity_random: 20.0,
            spread: std::f32::consts::FRAC_PI_4,
            gravity: Vec2::new(0.0, 98.0), // 2D 重力向下（Y 向下）
            initial_size: 8.0,
            end_size: 0.0,
            initial_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            texture_path: None,
            particles: Vec::new(),
            emission_timer: 0.0,
        }
    }

    /// 每帧更新粒子系统
    pub fn update(&mut self, delta: f32) {
        // 更新存活粒子
        self.particles.retain_mut(|p| {
            p.elapsed += delta;
            let t = p.elapsed / p.lifetime;
            p.velocity += self.gravity * delta;
            p.position += p.velocity * delta;
            p.rotation += p.angular_velocity * delta;
            // 线性插值大小和颜色
            p.size = self.initial_size + (self.end_size - self.initial_size) * t;
            for i in 0..4 {
                p.color[i] = self.initial_color[i]
                    + (self.end_color[i] - self.initial_color[i]) * t;
            }
            p.elapsed < p.lifetime
        });

        // 发射新粒子
        if self.emitting && (self.particles.len() as u32) < self.max_particles {
            self.emission_timer += delta;
            let interval = 1.0 / self.emission_rate.max(0.001);
            while self.emission_timer >= interval
                && (self.particles.len() as u32) < self.max_particles
            {
                self.emission_timer -= interval;
                self.emit_one();
            }
        }
    }

    fn emit_one(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        // 简单伪随机（生产环境应使用 rand crate）
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(12345)
            .wrapping_add(self.particles.len() as u32);
        let rng = |s: u32| -> f32 {
            let v = s.wrapping_mul(1664525).wrapping_add(1013904223);
            ((v >> 16) as f32 / 65536.0) * 2.0 - 1.0
        };
        let angle = rng(seed) * self.spread;
        let speed = self.initial_velocity * (1.0 + rng(seed.wrapping_add(1)) * self.velocity_random / self.initial_velocity.max(1.0));
        let life_var = self.lifetime * (1.0 + rng(seed.wrapping_add(2)) * self.lifetime_random);
        let dir = Vec2::new(angle.cos(), angle.sin());
        self.particles.push(Particle2d {
            position: self.base.position(),
            velocity: dir * speed,
            lifetime: life_var.max(0.01),
            elapsed: 0.0,
            size: self.initial_size,
            color: self.initial_color,
            rotation: 0.0,
            angular_velocity: rng(seed.wrapping_add(3)) * 2.0,
        });
    }
}

// ── 测试 ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node2d_look_at() {
        let mut node = Node2d::new("test");
        node.set_position(Vec2::new(0.0, 0.0));
        node.look_at(Vec2::new(0.0, 1.0));
        // look_at(0,1) 从 (0,0) 向上，角度应为 0（正 Y 轴）
        assert!((node.rotation()).abs() < 1e-5);
    }

    #[test]
    fn test_node2d_translate() {
        let mut node = Node2d::new("test");
        node.translate(Vec2::new(10.0, 5.0));
        assert_eq!(node.position(), Vec2::new(10.0, 5.0));
    }

    #[test]
    fn test_sprite2d_creation() {
        let sprite = Sprite2d::new("player", "assets/textures/player.png");
        assert_eq!(sprite.texture_path, "assets/textures/player.png");
        assert_eq!(sprite.color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_animated_sprite_update() {
        let mut sprite = AnimatedSprite2d::new("hero", "assets/hero.png");
        let mut anim = SpriteAnimation::new("run");
        anim.fps = Some(10.0);
        for i in 0..4 {
            anim.frames.push(AnimationFrame {
                region: [i as f32 * 32.0, 0.0, 32.0, 32.0],
                duration: 0.1,
            });
        }
        sprite.add_animation(anim);
        sprite.play("run");
        assert!(sprite.playing);

        // 推进超过一帧时间（0.1s @ 10fps）
        sprite.update(0.15);
        assert_eq!(sprite.current_frame, 1);
    }

    #[test]
    fn test_tilemap_cells() {
        let mut map = TileMap::new("map", 32, 32);
        map.set_cell(0, 0, 1);
        map.set_cell(1, 0, 2);
        assert_eq!(map.get_cell(0, 0), Some(1));
        assert_eq!(map.get_cell(1, 0), Some(2));
        assert_eq!(map.get_cell(0, 1), None);
        map.clear_cell(0, 0);
        assert_eq!(map.get_cell(0, 0), None);
    }

    #[test]
    fn test_tilemap_world_to_cell() {
        let map = TileMap::new("map", 32, 32);
        let (cx, cy) = map.world_to_cell(Vec2::new(64.0, 32.0));
        assert_eq!(cx, 2);
        assert_eq!(cy, 1);
    }

    #[test]
    fn test_character_body2d_jump() {
        let mut body = CharacterBody2d::new("player");
        body.is_on_floor = true;
        let jumped = body.jump(500.0);
        assert!(jumped);
        assert_eq!(body.velocity.y, 500.0);
        assert!(!body.is_on_floor);
    }

    #[test]
    fn test_area2d_overlap_tracking() {
        let mut area = Area2d::new("trigger");
        area.add_overlap(42);
        area.add_overlap(99);
        assert!(area.has_body(42));
        assert!(area.has_body(99));
        area.remove_overlap(42);
        assert!(!area.has_body(42));
        assert_eq!(area.overlapping_bodies.len(), 1);
    }

    #[test]
    fn test_tileset_from_sprite_sheet() {
        let ts = TileSet::from_sprite_sheet("ts", "tileset.png", 16, 16, 4, 4);
        assert_eq!(ts.tiles.len(), 16);
        assert_eq!(ts.tiles[5].region, [16.0, 16.0, 16.0, 16.0]);
    }

    #[test]
    fn test_cpu_particles_update() {
        let mut ps = CpuParticles2d::new("sparks");
        ps.emission_rate = 60.0;
        ps.update(0.1);
        // 经过 0.1s @ 60/s 应该发射至少 6 个粒子
        assert!(!ps.particles.is_empty());
    }
}
