//! 2D 和 3D 变换类型

use glam::{Affine2, Affine3A, Mat3, Mat4, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

// ── 2D Transform ─────────────────────────────────────────────────────────────

/// 2D 变换（位置 + 旋转 + 缩放）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2d {
    pub position: Vec2,
    /// 旋转角度（弧度）
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2d {
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    #[inline]
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self { position, rotation, scale }
    }

    #[inline]
    pub fn from_position(position: Vec2) -> Self {
        Self { position, ..Self::IDENTITY }
    }

    /// 转换为仿射矩阵
    pub fn to_affine2(&self) -> Affine2 {
        Affine2::from_scale_angle_translation(self.scale, self.rotation, self.position)
    }

    /// 转换为 Mat3（用于着色器）
    pub fn to_mat3(&self) -> Mat3 {
        Mat3::from_scale_angle_translation(self.scale, self.rotation, self.position)
    }

    /// 变换一个点
    #[inline]
    pub fn transform_point(&self, point: Vec2) -> Vec2 {
        self.to_affine2().transform_point2(point)
    }

    /// 变换一个向量（不含位移）
    #[inline]
    pub fn transform_vector(&self, vector: Vec2) -> Vec2 {
        self.to_affine2().transform_vector2(vector)
    }

    /// 组合两个变换（self 作用于 child 之后）
    #[inline]
    pub fn mul_transform(&self, child: &Self) -> Self {
        let combined = self.to_affine2() * child.to_affine2();
        // 从仿射矩阵提取 TRS
        let position = combined.translation;
        let (scale_x, scale_y) = (
            combined.matrix2.col(0).length(),
            combined.matrix2.col(1).length(),
        );
        let rotation = combined.matrix2.col(0).y.atan2(combined.matrix2.col(0).x);
        Self {
            position,
            rotation,
            scale: Vec2::new(scale_x, scale_y),
        }
    }

    /// 求逆变换
    #[inline]
    pub fn inverse(&self) -> Self {
        let inv = self.to_affine2().inverse();
        let position = inv.translation;
        let (scale_x, scale_y) = (
            inv.matrix2.col(0).length(),
            inv.matrix2.col(1).length(),
        );
        let rotation = inv.matrix2.col(0).y.atan2(inv.matrix2.col(0).x);
        Self {
            position,
            rotation,
            scale: Vec2::new(scale_x, scale_y),
        }
    }

    /// 向前方向（旋转角度对应的单位向量）
    #[inline]
    pub fn forward(&self) -> Vec2 {
        Vec2::new(self.rotation.cos(), self.rotation.sin())
    }

    /// 向右方向（旋转 +90°）
    #[inline]
    pub fn right(&self) -> Vec2 {
        let angle = self.rotation + std::f32::consts::FRAC_PI_2;
        Vec2::new(angle.cos(), angle.sin())
    }
}

impl Default for Transform2d {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// ── 3D Transform ─────────────────────────────────────────────────────────────

/// 3D 变换（位置 + 四元数旋转 + 缩放）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3d {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform3d {
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    #[inline]
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { position, rotation, scale }
    }

    #[inline]
    pub fn from_position(position: Vec3) -> Self {
        Self { position, ..Self::IDENTITY }
    }

    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self { rotation, ..Self::IDENTITY }
    }

    /// 朝向目标点（look-at）
    pub fn looking_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (target - eye).normalize();
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        let rotation = Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward));
        Self {
            position: eye,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// 转换为仿射矩阵 Affine3A
    pub fn to_affine3a(&self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// 转换为 Mat4（用于着色器 uniform）
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// 变换一个点
    #[inline]
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.to_affine3a().transform_point3(point)
    }

    /// 变换一个向量（不含位移）
    #[inline]
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.to_affine3a().transform_vector3(vector)
    }

    /// 组合变换
    #[inline]
    pub fn mul_transform(&self, child: &Self) -> Self {
        Self {
            position: self.transform_point(child.position),
            rotation: self.rotation * child.rotation,
            scale: self.scale * child.scale,
        }
    }

    /// 求逆变换
    #[inline]
    pub fn inverse(&self) -> Self {
        let inv_scale = Vec3::ONE / self.scale;
        let inv_rotation = self.rotation.inverse();
        let inv_position = inv_rotation * (inv_scale * -self.position);
        Self {
            position: inv_position,
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    /// 前方向（-Z 轴）
    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// 后方向（+Z 轴）
    #[inline]
    pub fn back(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    /// 右方向（+X 轴）
    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// 左方向（-X 轴）
    #[inline]
    pub fn left(&self) -> Vec3 {
        self.rotation * Vec3::NEG_X
    }

    /// 上方向（+Y 轴）
    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// 下方向（-Y 轴）
    #[inline]
    pub fn down(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Y
    }
}

impl Default for Transform3d {
    fn default() -> Self {
        Self::IDENTITY
    }
}
