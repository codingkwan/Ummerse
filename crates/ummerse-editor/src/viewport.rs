//! 视口（Viewport）- 2D/3D 场景可视化

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 视口模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewportMode {
    /// 2D 模式
    Mode2d,
    /// 3D 模式
    Mode3d,
    /// 分屏（2D + 3D）
    Split,
}

/// 视口状态（渲染目标尺寸和相机信息）
#[derive(Debug, Clone)]
pub struct ViewportState {
    pub width: u32,
    pub height: u32,
    pub mode: ViewportMode,
    /// 是否显示网格
    pub show_grid: bool,
    /// 是否显示碰撞体
    pub show_colliders: bool,
    /// 是否显示骨骼
    pub show_bones: bool,
    /// 是否显示 gizmos
    pub show_gizmos: bool,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            mode: ViewportMode::Mode2d,
            show_grid: true,
            show_colliders: false,
            show_bones: false,
            show_gizmos: true,
        }
    }
}

// ── 2D 视口 ───────────────────────────────────────────────────────────────────

/// 2D 编辑器视口
pub struct Viewport2d {
    pub state: ViewportState,
    /// 相机位置（像素坐标）
    pub camera_pos: Vec2,
    /// 缩放（1.0 = 100%）
    pub zoom: f32,
    /// 网格大小（像素）
    pub grid_size: f32,
    /// 吸附到网格
    pub snap_to_grid: bool,
    /// 选中的节点 ID 列表
    pub selected_nodes: Vec<String>,
    /// 拖拽操作状态
    pub drag_state: DragState,
}

impl Viewport2d {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            state: ViewportState {
                width,
                height,
                mode: ViewportMode::Mode2d,
                ..Default::default()
            },
            camera_pos: Vec2::ZERO,
            zoom: 1.0,
            grid_size: 32.0,
            snap_to_grid: false,
            selected_nodes: Vec::new(),
            drag_state: DragState::None,
        }
    }

    /// 屏幕坐标 → 世界坐标
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let center = Vec2::new(self.state.width as f32, self.state.height as f32) * 0.5;
        (screen_pos - center) / self.zoom + self.camera_pos
    }

    /// 世界坐标 → 屏幕坐标
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let center = Vec2::new(self.state.width as f32, self.state.height as f32) * 0.5;
        (world_pos - self.camera_pos) * self.zoom + center
    }

    /// 缩放（以屏幕中心为锚点）
    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom = (self.zoom * (1.0 + delta * 0.1)).clamp(0.05, 20.0);
    }

    /// 移动相机
    pub fn pan(&mut self, delta: Vec2) {
        self.camera_pos -= delta / self.zoom;
    }

    /// 将位置吸附到网格
    pub fn snap(&self, pos: Vec2) -> Vec2 {
        if self.snap_to_grid {
            (pos / self.grid_size).round() * self.grid_size
        } else {
            pos
        }
    }

    /// 重置视图
    pub fn reset_view(&mut self) {
        self.camera_pos = Vec2::ZERO;
        self.zoom = 1.0;
    }

    /// 选择节点
    pub fn select_node(&mut self, node_id: String, add_to_selection: bool) {
        if !add_to_selection {
            self.selected_nodes.clear();
        }
        if !self.selected_nodes.contains(&node_id) {
            self.selected_nodes.push(node_id);
        }
    }

    /// 清除选择
    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }
}

impl Default for Viewport2d {
    fn default() -> Self {
        Self::new(1280, 720)
    }
}

// ── 3D 视口 ───────────────────────────────────────────────────────────────────

/// 3D 编辑器视口
pub struct Viewport3d {
    pub state: ViewportState,
    /// 相机位置（世界空间）
    pub camera_pos: Vec3,
    /// 相机旋转（偏航/俯仰角，弧度）
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    /// 是否在轨道相机模式
    pub orbit_mode: bool,
    /// 轨道焦点
    pub orbit_target: Vec3,
    /// 轨道距离
    pub orbit_distance: f32,
    /// 相机移动速度
    pub move_speed: f32,
    /// 选中的节点 ID
    pub selected_nodes: Vec<String>,
    /// 视口模式（透视/正交/顶/前/右）
    pub camera_mode: CameraMode3d,
}

/// 3D 视口相机模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode3d {
    /// 透视相机（自由漫游）
    Perspective,
    /// 正交相机
    Orthographic,
    /// 顶视图（+Y）
    Top,
    /// 底视图（-Y）
    Bottom,
    /// 前视图（-Z）
    Front,
    /// 后视图（+Z）
    Back,
    /// 左视图（-X）
    Left,
    /// 右视图（+X）
    Right,
}

impl Viewport3d {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            state: ViewportState {
                width,
                height,
                mode: ViewportMode::Mode3d,
                ..Default::default()
            },
            camera_pos: Vec3::new(0.0, 5.0, 10.0),
            camera_yaw: 0.0,
            camera_pitch: -0.4,
            orbit_mode: true,
            orbit_target: Vec3::ZERO,
            orbit_distance: 10.0,
            move_speed: 5.0,
            selected_nodes: Vec::new(),
            camera_mode: CameraMode3d::Perspective,
        }
    }

    /// 轨道相机旋转
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.camera_yaw += delta_yaw;
        self.camera_pitch = (self.camera_pitch + delta_pitch).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
        self.update_camera_pos_from_orbit();
    }

    /// 轨道缩放（调整轨道距离）
    pub fn zoom(&mut self, delta: f32) {
        self.orbit_distance = (self.orbit_distance - delta).clamp(0.5, 1000.0);
        self.update_camera_pos_from_orbit();
    }

    /// 从轨道参数更新相机位置
    fn update_camera_pos_from_orbit(&mut self) {
        let x = self.orbit_distance * self.camera_yaw.sin() * self.camera_pitch.cos();
        let y = self.orbit_distance * self.camera_pitch.sin();
        let z = self.orbit_distance * self.camera_yaw.cos() * self.camera_pitch.cos();
        self.camera_pos = self.orbit_target + Vec3::new(x, y, z);
    }

    /// 前进/后退（飞行模式）
    pub fn fly_forward(&mut self, amount: f32) {
        let forward = self.forward_vector();
        self.camera_pos += forward * amount;
        if self.orbit_mode {
            self.orbit_target += forward * amount;
        }
    }

    /// 相机前方向向量
    pub fn forward_vector(&self) -> Vec3 {
        Vec3::new(
            self.camera_yaw.sin() * self.camera_pitch.cos(),
            self.camera_pitch.sin(),
            -self.camera_yaw.cos() * self.camera_pitch.cos(),
        )
        .normalize_or_zero()
    }

    /// 相机右方向向量
    pub fn right_vector(&self) -> Vec3 {
        let forward = self.forward_vector();
        forward.cross(Vec3::Y).normalize_or_zero()
    }

    /// 重置视图（聚焦到原点）
    pub fn reset_view(&mut self) {
        self.orbit_target = Vec3::ZERO;
        self.orbit_distance = 10.0;
        self.camera_yaw = 0.0;
        self.camera_pitch = -0.4;
        self.update_camera_pos_from_orbit();
    }

    /// 聚焦到选中节点
    pub fn focus_selection(&mut self, target: Vec3) {
        self.orbit_target = target;
        self.update_camera_pos_from_orbit();
    }
}

impl Default for Viewport3d {
    fn default() -> Self {
        Self::new(1280, 720)
    }
}

// ── 拖拽操作 ──────────────────────────────────────────────────────────────────

/// 拖拽操作状态
#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    None,
    /// 正在拖动相机（平移）
    Panning {
        start: Vec2,
    },
    /// 正在拖动节点
    MovingNode {
        node_id: String,
        start: Vec2,
        offset: Vec2,
    },
    /// 框选
    BoxSelect {
        start: Vec2,
        current: Vec2,
    },
}
