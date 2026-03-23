//! 相机系统

use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 相机投影方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraProjection {
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov_y_radians: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
}

impl CameraProjection {
    /// 正交投影（2D）
    pub fn orthographic_2d(width: f32, height: f32) -> Self {
        Self::Orthographic {
            left: -width * 0.5,
            right: width * 0.5,
            bottom: -height * 0.5,
            top: height * 0.5,
            near: -1000.0,
            far: 1000.0,
        }
    }

    /// 透视投影（3D）
    pub fn perspective(fov_y_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov_y_radians: fov_y_degrees.to_radians(),
            aspect_ratio,
            near,
            far,
        }
    }

    /// 转换为投影矩阵
    pub fn to_matrix(&self) -> Mat4 {
        match self {
            Self::Orthographic { left, right, bottom, top, near, far } => {
                Mat4::orthographic_rh(*left, *right, *bottom, *top, *near, *far)
            }
            Self::Perspective { fov_y_radians, aspect_ratio, near, far } => {
                Mat4::perspective_rh(*fov_y_radians, *aspect_ratio, *near, *far)
            }
        }
    }
}

/// 2D 相机
#[derive(Debug, Clone)]
pub struct Camera2d {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Camera2d {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            viewport_width,
            viewport_height,
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        let w = self.viewport_width / self.zoom;
        let h = self.viewport_height / self.zoom;
        Mat4::orthographic_rh(-w * 0.5, w * 0.5, -h * 0.5, h * 0.5, -1000.0, 1000.0)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_rotation_z(-self.rotation)
            * Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0))
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// 3D 相机
#[derive(Debug, Clone)]
pub struct Camera3d {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
}

impl Camera3d {
    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y_degrees: 60.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio,
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fov_y_degrees.to_radians(),
            self.aspect_ratio,
            self.near,
            self.far,
        )
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}
