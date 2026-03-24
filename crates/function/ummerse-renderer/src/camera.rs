//! 相机系统 - 2D 正交相机 + 3D 透视/正交相机

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};

// ── 相机 Uniform（传入 GPU 的数据结构）───────────────────────────────────────

/// 2D 相机 Uniform（发送到 GPU）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform2d {
    /// 正交投影 × 视图矩阵（列主序）
    pub view_proj: [[f32; 4]; 4],
}

/// 3D 相机 Uniform（发送到 GPU）
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct CameraUniform3d {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    /// 世界空间相机位置
    pub position: [f32; 3],
    pub _pad: f32,
}

// ── 相机投影类型 ──────────────────────────────────────────────────────────────

/// 相机投影方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraProjection {
    /// 透视投影（3D）
    Perspective {
        /// 垂直视野角（弧度）
        fov_y: f32,
        /// 近裁剪面
        near: f32,
        /// 远裁剪面
        far: f32,
    },
    /// 正交投影（2D/正交 3D）
    Orthographic {
        /// 视口宽度（世界单位）
        width: f32,
        /// 视口高度（世界单位）
        height: f32,
        /// 近裁剪面
        near: f32,
        /// 远裁剪面
        far: f32,
    },
}

impl CameraProjection {
    /// 计算投影矩阵
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        match *self {
            Self::Perspective { fov_y, near, far } => {
                Mat4::perspective_rh(fov_y, aspect, near, far)
            }
            Self::Orthographic { width, height, near, far } => {
                let half_w = width * 0.5;
                let half_h = height * 0.5;
                Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, near, far)
            }
        }
    }
}

// ── 2D 相机 ──────────────────────────────────────────────────────────────────

/// 2D 正交相机
#[derive(Debug, Clone)]
pub struct Camera2d {
    /// 相机世界位置
    pub position: Vec2,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// 缩放（1.0 = 1 像素 = 1 世界单位）
    pub zoom: f32,
    /// 视口宽度（像素）
    pub viewport_width: f32,
    /// 视口高度（像素）
    pub viewport_height: f32,
    /// 近/远裁剪面
    pub near: f32,
    pub far: f32,
}

impl Camera2d {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            zoom: 1.0,
            viewport_width,
            viewport_height,
            near: -1000.0,
            far: 1000.0,
        }
    }

    /// 计算视图矩阵（相机变换的逆）
    pub fn view_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0));
        let rotation = Mat4::from_rotation_z(-self.rotation);
        let scale = Mat4::from_scale(Vec3::splat(self.zoom));
        scale * rotation * translation
    }

    /// 计算正交投影矩阵
    pub fn projection_matrix(&self) -> Mat4 {
        let half_w = self.viewport_width * 0.5 / self.zoom;
        let half_h = self.viewport_height * 0.5 / self.zoom;
        Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, self.near, self.far)
    }

    /// 计算 view-projection 矩阵
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// 生成 GPU uniform 数据
    pub fn to_uniform(&self) -> CameraUniform2d {
        CameraUniform2d {
            view_proj: self.view_projection_matrix().to_cols_array_2d(),
        }
    }

    /// 屏幕坐标 → 世界坐标（Y 轴向上）
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let ndc = Vec2::new(
            2.0 * screen_pos.x / self.viewport_width - 1.0,
            1.0 - 2.0 * screen_pos.y / self.viewport_height,
        );
        let vp_inv = self.view_projection_matrix().inverse();
        let world = vp_inv.transform_point3(Vec3::new(ndc.x, ndc.y, 0.0));
        Vec2::new(world.x, world.y)
    }

    /// 世界坐标 → 屏幕坐标
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let vp = self.view_projection_matrix();
        let clip = vp.transform_point3(Vec3::new(world_pos.x, world_pos.y, 0.0));
        Vec2::new(
            (clip.x + 1.0) * 0.5 * self.viewport_width,
            (1.0 - clip.y) * 0.5 * self.viewport_height,
        )
    }

    /// 更新视口尺寸
    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }
}

impl Default for Camera2d {
    fn default() -> Self {
        Self::new(1280.0, 720.0)
    }
}

// ── 3D 相机 ──────────────────────────────────────────────────────────────────

/// 3D 相机
#[derive(Debug, Clone)]
pub struct Camera3d {
    /// 相机世界位置
    pub position: Vec3,
    /// 相机目标点（look-at 目标）
    pub target: Vec3,
    /// 上方向（通常为 Y 轴）
    pub up: Vec3,
    /// 投影方式
    pub projection: CameraProjection,
    /// 视口宽高比
    pub aspect: f32,
}

impl Camera3d {
    /// 创建透视相机
    pub fn perspective(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: CameraProjection::Perspective {
                fov_y: 60.0_f32.to_radians(),
                near: 0.1,
                far: 1000.0,
            },
            aspect,
        }
    }

    /// 创建正交相机
    pub fn orthographic(width: f32, height: f32, aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 100.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: CameraProjection::Orthographic {
                width,
                height,
                near: 0.1,
                far: 1000.0,
            },
            aspect,
        }
    }

    /// 计算视图矩阵
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// 计算投影矩阵
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.projection_matrix(self.aspect)
    }

    /// 计算 view-projection 矩阵
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// 生成 GPU uniform 数据
    pub fn to_uniform(&self) -> CameraUniform3d {
        CameraUniform3d {
            view_proj: self.view_projection_matrix().to_cols_array_2d(),
            view: self.view_matrix().to_cols_array_2d(),
            position: self.position.to_array(),
            _pad: 0.0,
        }
    }

    /// 更新宽高比（窗口 resize 时调用）
    pub fn update_aspect(&mut self, viewport_width: f32, viewport_height: f32) {
        self.aspect = viewport_width / viewport_height.max(1.0);
        // 同时更新正交相机的宽高
        if let CameraProjection::Orthographic {
            width: ref mut ortho_w,
            height: ref mut ortho_h,
            ..
        } = self.projection
        {
            *ortho_w = viewport_width;
            *ortho_h = viewport_height;
        }
    }

    /// 屏幕坐标 → 世界空间射线（用于拾取）
    pub fn screen_to_ray(
        &self,
        screen_pos: Vec2,
        viewport_width: f32,
        viewport_height: f32,
    ) -> (Vec3, Vec3) {
        let ndc = Vec2::new(
            2.0 * screen_pos.x / viewport_width - 1.0,
            1.0 - 2.0 * screen_pos.y / viewport_height,
        );
        let vp_inv = self.view_projection_matrix().inverse();
        let near = vp_inv.transform_point3(Vec3::new(ndc.x, ndc.y, -1.0));
        let far = vp_inv.transform_point3(Vec3::new(ndc.x, ndc.y, 1.0));
        let direction = (far - near).normalize_or_zero();
        (near, direction)
    }

    /// 判断点是否在视锥内（用于 CPU 剔除）
    pub fn contains_point(&self, point: Vec3) -> bool {
        let vp = self.view_projection_matrix();
        let clip = vp.transform_point3(point);
        clip.x >= -1.0
            && clip.x <= 1.0
            && clip.y >= -1.0
            && clip.y <= 1.0
            && clip.z >= -1.0
            && clip.z <= 1.0
    }
}

impl Default for Camera3d {
    fn default() -> Self {
        Self::perspective(16.0 / 9.0)
    }
}
