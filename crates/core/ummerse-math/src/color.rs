//! RGBA 颜色类型
//!
//! 提供线性和 sRGB 两种颜色空间，以及丰富的颜色操作工具。

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 线性 RGBA 颜色（0.0 - 1.0 范围，线性光照空间）
///
/// # 颜色空间说明
/// - 所有数学运算（lerp、混合等）在线性空间进行
/// - GPU Shader 接收线性颜色；sRGB 转换由渲染管线处理
/// - 从文件/HTML 读取的颜色通常为 sRGB，需先调用 [`Color::from_srgb_hex`]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    // ── 预定义颜色常量 ───────────────────────────────────────────────────
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const ORANGE: Self = Self::new(1.0, 0.5, 0.0, 1.0);
    pub const PURPLE: Self = Self::new(0.5, 0.0, 0.5, 1.0);
    pub const PINK: Self = Self::new(1.0, 0.75, 0.8, 1.0);
    pub const GRAY: Self = Self::new(0.5, 0.5, 0.5, 1.0);
    pub const DARK_GRAY: Self = Self::new(0.25, 0.25, 0.25, 1.0);
    pub const LIGHT_GRAY: Self = Self::new(0.75, 0.75, 0.75, 1.0);

    /// 创建 RGBA 颜色
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// 从 RGB 创建（alpha = 1.0）
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// 从灰度值创建（r=g=b=gray）
    #[inline]
    pub const fn gray(value: f32) -> Self {
        Self::new(value, value, value, 1.0)
    }

    /// 从 8bit RGBA hex 值创建（如 0xFF8800FF）
    #[inline]
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Self::new(r, g, b, a)
    }

    /// 从 HTML hex 字符串创建（如 "#FF8800" 或 "#FF8800FF"）
    ///
    /// 返回 `None` 若格式无效。
    pub fn from_html(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        match s.len() {
            3 => {
                // #RGB -> #RRGGBBFF 展开
                let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()? as f32 / 255.0;
                Some(Self::new(r, g, b, 1.0))
            }
            6 => {
                let hex = u32::from_str_radix(s, 16).ok()?;
                Some(Self::from_hex((hex << 8) | 0xFF))
            }
            8 => {
                let hex = u32::from_str_radix(s, 16).ok()?;
                Some(Self::from_hex(hex))
            }
            _ => None,
        }
    }

    /// 从 HSV（色调/饱和度/明度）创建，alpha = 1.0
    ///
    /// - `h`: 0.0 ~ 360.0（度）
    /// - `s`: 0.0 ~ 1.0
    /// - `v`: 0.0 ~ 1.0
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = v - c;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        Self::new(r + m, g + m, b + m, 1.0)
    }

    // ── 颜色操作 ──────────────────────────────────────────────────────────

    /// 线性插值
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// 颜色相乘（着色运算，逐分量相乘）
    #[inline]
    #[must_use]
    pub fn multiply(self, other: Self) -> Self {
        Self {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a,
        }
    }

    /// 亮度调节（>1.0 更亮，<1.0 更暗）
    #[inline]
    #[must_use]
    pub fn lightened(self, factor: f32) -> Self {
        Self {
            r: (self.r * factor).clamp(0.0, 1.0),
            g: (self.g * factor).clamp(0.0, 1.0),
            b: (self.b * factor).clamp(0.0, 1.0),
            a: self.a,
        }
    }

    /// 设置透明度，返回新颜色
    #[inline]
    #[must_use]
    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a.clamp(0.0, 1.0);
        self
    }

    /// 转为灰度（感知亮度公式）
    #[inline]
    #[must_use]
    pub fn to_grayscale(self) -> Self {
        let lum = self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722;
        Self::new(lum, lum, lum, self.a)
    }

    /// 反色（色彩取反，不反转 alpha）
    #[inline]
    #[must_use]
    pub fn inverted(self) -> Self {
        Self::new(1.0 - self.r, 1.0 - self.g, 1.0 - self.b, self.a)
    }

    // ── 转换方法 ──────────────────────────────────────────────────────────

    /// 转换为 [f32; 4] 数组
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// 转换为 [u8; 4] 数组（线性 → sRGB 量化）
    #[inline]
    pub fn to_u8_array(self) -> [u8; 4] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8,
            (self.a.clamp(0.0, 1.0) * 255.0) as u8,
        ]
    }

    /// 转换为 HSV 元组 `(h, s, v)`，h ∈ [0, 360)
    #[must_use]
    pub fn to_hsv(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;
        let v = max;
        let s = if max > f32::EPSILON { delta / max } else { 0.0 };
        let h = if delta < f32::EPSILON {
            0.0
        } else if (max - self.r).abs() < f32::EPSILON {
            60.0 * ((self.g - self.b) / delta).rem_euclid(6.0)
        } else if (max - self.g).abs() < f32::EPSILON {
            60.0 * ((self.b - self.r) / delta + 2.0)
        } else {
            60.0 * ((self.r - self.g) / delta + 4.0)
        };
        (h, s, v)
    }

    /// 感知亮度（0.0 暗 ~ 1.0 亮）
    #[inline]
    #[must_use]
    pub fn luminance(self) -> f32 {
        self.r * 0.2126 + self.g * 0.7152 + self.b * 0.0722
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<[f32; 3]> for Color {
    fn from(arr: [f32; 3]) -> Self {
        Self::rgb(arr[0], arr[1], arr[2])
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        c.to_array()
    }
}

impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
    }
}

impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b, self.a + rhs.a)
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rgba({:.3}, {:.3}, {:.3}, {:.3})",
            self.r, self.g, self.b, self.a
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_html_6() {
        let c = Color::from_html("#FF8800").unwrap();
        assert!((c.r - 1.0).abs() < 1e-3);
        assert!((c.g - 0.533).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 1e-3);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_from_html_3() {
        let c = Color::from_html("#F80").unwrap();
        assert!((c.r - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_lerp() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hsv_roundtrip() {
        let original = Color::from_hsv(120.0, 0.8, 0.9);
        let (h, s, v) = original.to_hsv();
        let reconstructed = Color::from_hsv(h, s, v);
        assert!((original.r - reconstructed.r).abs() < 1e-5);
        assert!((original.g - reconstructed.g).abs() < 1e-5);
        assert!((original.b - reconstructed.b).abs() < 1e-5);
    }

    #[test]
    fn test_bytemuck_pod() {
        // Color 实现 Pod，可直接用于 GPU 上传
        let c = Color::RED;
        let bytes: &[u8] = bytemuck::bytes_of(&c);
        assert_eq!(bytes.len(), 16); // 4 * f32
    }
}
