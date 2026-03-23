//! UI 系统 - 编辑器 UI 抽象层
//!
//! 基于 taffy（Flexbox/Grid 布局引擎）的 UI 框架抽象，
//! 为编辑器提供跨平台的声明式 UI 描述。
//! 实际渲染委托给 Bevy UI 或 WGPU 自绘。

use serde::{Deserialize, Serialize};

/// UI 颜色（RGBA，0.0~1.0）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl UiColor {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    // 编辑器主题色
    pub const BACKGROUND: Self = Self {
        r: 0.11,
        g: 0.11,
        b: 0.13,
        a: 1.0,
    };
    pub const SURFACE: Self = Self {
        r: 0.15,
        g: 0.15,
        b: 0.18,
        a: 1.0,
    };
    pub const BORDER: Self = Self {
        r: 0.25,
        g: 0.25,
        b: 0.28,
        a: 1.0,
    };
    pub const ACCENT: Self = Self {
        r: 0.26,
        g: 0.56,
        b: 0.97,
        a: 1.0,
    };
    pub const TEXT_PRIMARY: Self = Self {
        r: 0.95,
        g: 0.95,
        b: 0.95,
        a: 1.0,
    };
    pub const TEXT_SECONDARY: Self = Self {
        r: 0.65,
        g: 0.65,
        b: 0.65,
        a: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let a = (hex & 0xFF) as f32 / 255.0;
        Self { r, g, b, a }
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

// ── 主题系统 ──────────────────────────────────────────────────────────────────

/// 编辑器主题
#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub name: String,
    pub background: UiColor,
    pub surface: UiColor,
    pub surface_hover: UiColor,
    pub border: UiColor,
    pub accent: UiColor,
    pub accent_hover: UiColor,
    pub text_primary: UiColor,
    pub text_secondary: UiColor,
    pub text_disabled: UiColor,
    pub error: UiColor,
    pub warning: UiColor,
    pub success: UiColor,
    /// 字体大小（像素）
    pub font_size: f32,
    pub font_size_small: f32,
    pub font_size_large: f32,
    /// 圆角半径
    pub border_radius: f32,
    /// 标准间距
    pub spacing: f32,
}

impl EditorTheme {
    /// 深色主题（默认）
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            background: UiColor::BACKGROUND,
            surface: UiColor::SURFACE,
            surface_hover: UiColor::new(0.20, 0.20, 0.23, 1.0),
            border: UiColor::BORDER,
            accent: UiColor::ACCENT,
            accent_hover: UiColor::new(0.35, 0.65, 1.0, 1.0),
            text_primary: UiColor::TEXT_PRIMARY,
            text_secondary: UiColor::TEXT_SECONDARY,
            text_disabled: UiColor::new(0.4, 0.4, 0.4, 1.0),
            error: UiColor::new(0.9, 0.3, 0.3, 1.0),
            warning: UiColor::new(0.95, 0.7, 0.2, 1.0),
            success: UiColor::new(0.3, 0.85, 0.5, 1.0),
            font_size: 13.0,
            font_size_small: 11.0,
            font_size_large: 16.0,
            border_radius: 4.0,
            spacing: 8.0,
        }
    }

    /// 浅色主题
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            background: UiColor::new(0.95, 0.95, 0.96, 1.0),
            surface: UiColor::new(1.0, 1.0, 1.0, 1.0),
            surface_hover: UiColor::new(0.90, 0.90, 0.92, 1.0),
            border: UiColor::new(0.75, 0.75, 0.78, 1.0),
            accent: UiColor::new(0.1, 0.45, 0.9, 1.0),
            accent_hover: UiColor::new(0.15, 0.5, 0.95, 1.0),
            text_primary: UiColor::new(0.1, 0.1, 0.12, 1.0),
            text_secondary: UiColor::new(0.4, 0.4, 0.45, 1.0),
            text_disabled: UiColor::new(0.65, 0.65, 0.65, 1.0),
            error: UiColor::new(0.8, 0.15, 0.15, 1.0),
            warning: UiColor::new(0.85, 0.55, 0.0, 1.0),
            success: UiColor::new(0.15, 0.7, 0.35, 1.0),
            font_size: 13.0,
            font_size_small: 11.0,
            font_size_large: 16.0,
            border_radius: 4.0,
            spacing: 8.0,
        }
    }
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self::dark()
    }
}

// ── UI 组件描述符 ─────────────────────────────────────────────────────────────

/// UI 尺寸值
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UiSize {
    /// 固定像素大小
    Pixels(f32),
    /// 百分比（相对于父容器）
    Percent(f32),
    /// 自动（内容决定大小）
    Auto,
    /// Flex 填充剩余空间
    Fill,
}

/// UI 矩形内边距
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct UiInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl UiInsets {
    pub fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub fn horizontal(h: f32) -> Self {
        Self {
            left: h,
            right: h,
            ..Default::default()
        }
    }

    pub fn vertical(v: f32) -> Self {
        Self {
            top: v,
            bottom: v,
            ..Default::default()
        }
    }

    pub fn xy(x: f32, y: f32) -> Self {
        Self {
            top: y,
            right: x,
            bottom: y,
            left: x,
        }
    }
}

/// UI 弹性布局方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// UI 对齐方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

/// UI 文本截断方式
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
    Wrap,
}

// ── 状态栏 ────────────────────────────────────────────────────────────────────

/// 编辑器状态栏项
#[derive(Debug, Clone)]
pub struct StatusBarItem {
    pub id: String,
    pub text: String,
    pub tooltip: Option<String>,
    pub color: Option<UiColor>,
    pub align_right: bool,
}

impl StatusBarItem {
    pub fn left(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            tooltip: None,
            color: None,
            align_right: false,
        }
    }

    pub fn right(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            tooltip: None,
            color: None,
            align_right: true,
        }
    }

    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn with_color(mut self, color: UiColor) -> Self {
        self.color = Some(color);
        self
    }
}

/// 编辑器状态栏
pub struct StatusBar {
    items: Vec<StatusBarItem>,
}

impl StatusBar {
    pub fn new() -> Self {
        let items = vec![
            StatusBarItem::left("engine", "Ummerse v0.1.0"),
            StatusBarItem::left("project", "No Project"),
            StatusBarItem::right("fps", "-- FPS"),
            StatusBarItem::right("memory", "-- MB"),
        ];
        Self { items }
    }

    pub fn set_item(&mut self, id: &str, text: impl Into<String>) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.text = text.into();
        }
    }

    pub fn left_items(&self) -> Vec<&StatusBarItem> {
        self.items.iter().filter(|i| !i.align_right).collect()
    }

    pub fn right_items(&self) -> Vec<&StatusBarItem> {
        self.items.iter().filter(|i| i.align_right).collect()
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
