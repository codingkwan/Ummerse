//! 输入系统 - 键盘、鼠标、手柄输入管理
//!
//! 参考 Godot Input 单例设计，提供轮询和事件两种输入获取方式。

use ahash::AHashMap;
use serde::{Deserialize, Serialize};

// ── 键码 ──────────────────────────────────────────────────────────────────────

/// 键盘键码（参考 Godot Key 枚举）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // 字母键
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // 数字键（键盘行）
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    // 功能键
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // 特殊键
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    // 方向键
    Left,
    Right,
    Up,
    Down,
    // 修饰键
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    LSuper,
    RSuper,
    // 数字小键盘
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEnter,
    // 其他
    Slash,
    Backslash,
    Semicolon,
    Quote,
    Comma,
    Period,
    LeftBracket,
    RightBracket,
    Grave,
    Minus,
    Equal,
    CapsLock,
    NumLock,
    ScrollLock,
    PrintScreen,
    Pause,
    /// 未知键码
    Unknown(u32),
}

// ── 鼠标按键 ──────────────────────────────────────────────────────────────────

/// 鼠标按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    /// 侧键（前进/后退）
    Button4,
    Button5,
}

// ── 输入动作 ──────────────────────────────────────────────────────────────────

/// 输入动作（抽象输入映射）
///
/// 类似 Godot 的 InputAction，将具体按键映射到逻辑动作。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputAction(String);

impl InputAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for InputAction {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for InputAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 按键状态 ──────────────────────────────────────────────────────────────────

/// 按键/按钮状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    /// 当前帧按下（刚按下）
    JustPressed,
    /// 持续按住
    Held,
    /// 当前帧释放（刚释放）
    JustReleased,
    /// 未按下
    Released,
}

// ── 输入管理器 ────────────────────────────────────────────────────────────────

/// 输入管理器 - 管理键盘、鼠标输入状态
///
/// 每帧由平台层更新：
/// 1. 收集原生输入事件
/// 2. 更新状态机（JustPressed → Held → JustReleased → Released）
/// 3. 分发到输入动作映射
#[derive(Debug)]
pub struct InputManager {
    /// 键盘状态（使用 AHashMap 提升查询性能）
    keyboard: AHashMap<KeyCode, ButtonState>,
    /// 鼠标按键状态
    mouse_buttons: AHashMap<MouseButton, ButtonState>,
    /// 鼠标光标位置（像素）
    mouse_position: glam::Vec2,
    /// 本帧鼠标移动量
    mouse_delta: glam::Vec2,
    /// 鼠标滚轮增量（Y 轴）
    scroll_delta: f32,
    /// 动作映射（动作名 → 触发的键码列表）
    action_map: AHashMap<InputAction, Vec<KeyCode>>,
}

impl InputManager {
    /// 创建输入管理器
    pub fn new() -> Self {
        Self {
            keyboard: AHashMap::new(),
            mouse_buttons: AHashMap::new(),
            mouse_position: glam::Vec2::ZERO,
            mouse_delta: glam::Vec2::ZERO,
            scroll_delta: 0.0,
            action_map: AHashMap::new(),
        }
    }

    // ── 键盘查询 ──────────────────────────────────────────────────────────

    /// 键是否被按住（含 JustPressed）
    #[inline]
    #[must_use]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        matches!(
            self.keyboard.get(&key),
            Some(ButtonState::JustPressed | ButtonState::Held)
        )
    }

    /// 键是否在本帧刚刚按下
    #[inline]
    #[must_use]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        matches!(self.keyboard.get(&key), Some(ButtonState::JustPressed))
    }

    /// 键是否在本帧刚刚释放
    #[inline]
    #[must_use]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        matches!(self.keyboard.get(&key), Some(ButtonState::JustReleased))
    }

    // ── 鼠标查询 ──────────────────────────────────────────────────────────

    /// 鼠标按键是否被按住
    #[inline]
    #[must_use]
    pub fn is_mouse_button_pressed(&self, btn: MouseButton) -> bool {
        matches!(
            self.mouse_buttons.get(&btn),
            Some(ButtonState::JustPressed | ButtonState::Held)
        )
    }

    /// 鼠标按键是否在本帧刚刚按下
    #[inline]
    #[must_use]
    pub fn is_mouse_button_just_pressed(&self, btn: MouseButton) -> bool {
        matches!(self.mouse_buttons.get(&btn), Some(ButtonState::JustPressed))
    }

    /// 鼠标按键是否在本帧刚刚释放
    #[inline]
    #[must_use]
    pub fn is_mouse_button_just_released(&self, btn: MouseButton) -> bool {
        matches!(
            self.mouse_buttons.get(&btn),
            Some(ButtonState::JustReleased)
        )
    }

    /// 鼠标当前位置（像素）
    #[inline]
    pub fn mouse_position(&self) -> glam::Vec2 {
        self.mouse_position
    }

    /// 本帧鼠标移动量
    #[inline]
    pub fn mouse_delta(&self) -> glam::Vec2 {
        self.mouse_delta
    }

    /// 本帧滚轮增量
    #[inline]
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    // ── 动作映射 ──────────────────────────────────────────────────────────

    /// 注册输入动作映射
    pub fn register_action(&mut self, action: InputAction, keys: Vec<KeyCode>) {
        self.action_map.insert(action, keys);
    }

    /// 动作是否被触发（持续）
    #[must_use]
    pub fn is_action_pressed(&self, action: &InputAction) -> bool {
        self.action_map
            .get(action)
            .map(|keys| keys.iter().any(|&k| self.is_key_pressed(k)))
            .unwrap_or(false)
    }

    /// 动作是否在本帧刚触发
    #[must_use]
    pub fn is_action_just_pressed(&self, action: &InputAction) -> bool {
        self.action_map
            .get(action)
            .map(|keys| keys.iter().any(|&k| self.is_key_just_pressed(k)))
            .unwrap_or(false)
    }

    /// 动作是否在本帧刚释放
    #[must_use]
    pub fn is_action_just_released(&self, action: &InputAction) -> bool {
        self.action_map
            .get(action)
            .map(|keys| keys.iter().any(|&k| self.is_key_just_released(k)))
            .unwrap_or(false)
    }

    /// 获取轴输入值（-1.0 / 0.0 / 1.0）
    ///
    /// 正方向键按下返回 1.0，负方向键按下返回 -1.0。
    #[inline]
    pub fn get_axis(&self, negative: KeyCode, positive: KeyCode) -> f32 {
        let pos = f32::from(self.is_key_pressed(positive));
        let neg = f32::from(self.is_key_pressed(negative));
        pos - neg
    }

    /// 获取 2D 向量输入（方向键/WASD）
    pub fn get_vector(
        &self,
        left: KeyCode,
        right: KeyCode,
        up: KeyCode,
        down: KeyCode,
    ) -> glam::Vec2 {
        let x = self.get_axis(left, right);
        let y = self.get_axis(down, up); // Y 轴向上
        glam::Vec2::new(x, y).normalize_or_zero()
    }

    // ── 状态更新（由平台层调用）──────────────────────────────────────────

    /// 通知键按下事件
    pub fn press_key(&mut self, key: KeyCode) {
        let state = self.keyboard.entry(key).or_insert(ButtonState::Released);
        if matches!(state, ButtonState::Released | ButtonState::JustReleased) {
            *state = ButtonState::JustPressed;
        }
    }

    /// 通知键释放事件
    pub fn release_key(&mut self, key: KeyCode) {
        self.keyboard.insert(key, ButtonState::JustReleased);
    }

    /// 通知鼠标按键按下
    pub fn press_mouse_button(&mut self, btn: MouseButton) {
        let state = self
            .mouse_buttons
            .entry(btn)
            .or_insert(ButtonState::Released);
        if matches!(state, ButtonState::Released | ButtonState::JustReleased) {
            *state = ButtonState::JustPressed;
        }
    }

    /// 通知鼠标按键释放
    pub fn release_mouse_button(&mut self, btn: MouseButton) {
        self.mouse_buttons.insert(btn, ButtonState::JustReleased);
    }

    /// 通知鼠标移动
    pub fn move_mouse(&mut self, position: glam::Vec2, delta: glam::Vec2) {
        self.mouse_position = position;
        self.mouse_delta += delta; // 累加（帧内多次移动）
    }

    /// 通知鼠标滚轮
    pub fn scroll_mouse(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// 帧结束时推进状态机（JustPressed→Held，JustReleased→Released）
    ///
    /// 必须在每帧末调用一次。
    pub fn flush(&mut self) {
        for state in self.keyboard.values_mut() {
            match state {
                ButtonState::JustPressed => *state = ButtonState::Held,
                ButtonState::JustReleased => *state = ButtonState::Released,
                _ => {}
            }
        }
        for state in self.mouse_buttons.values_mut() {
            match state {
                ButtonState::JustPressed => *state = ButtonState::Held,
                ButtonState::JustReleased => *state = ButtonState::Released,
                _ => {}
            }
        }
        // 重置逐帧累积量
        self.mouse_delta = glam::Vec2::ZERO;
        self.scroll_delta = 0.0;
    }

    /// 获取所有当前按下的键
    #[must_use]
    pub fn pressed_keys(&self) -> Vec<KeyCode> {
        self.keyboard
            .iter()
            .filter(|(_, s)| matches!(s, ButtonState::JustPressed | ButtonState::Held))
            .map(|(&k, _)| k)
            .collect()
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_transitions() {
        let mut input = InputManager::new();

        input.press_key(KeyCode::Space);
        assert!(input.is_key_just_pressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));

        input.flush();
        assert!(!input.is_key_just_pressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));

        input.release_key(KeyCode::Space);
        assert!(input.is_key_just_released(KeyCode::Space));
        assert!(!input.is_key_pressed(KeyCode::Space));

        input.flush();
        assert!(!input.is_key_just_released(KeyCode::Space));
    }

    #[test]
    fn test_get_axis() {
        let mut input = InputManager::new();
        input.press_key(KeyCode::D);
        assert!((input.get_axis(KeyCode::A, KeyCode::D) - 1.0).abs() < f32::EPSILON);

        input.press_key(KeyCode::A);
        assert!(input.get_axis(KeyCode::A, KeyCode::D).abs() < f32::EPSILON);
    }

    #[test]
    fn test_action_mapping() {
        let mut input = InputManager::new();
        input.register_action("jump".into(), vec![KeyCode::Space, KeyCode::W]);

        input.press_key(KeyCode::Space);
        assert!(input.is_action_just_pressed(&"jump".into()));
        assert!(input.is_action_pressed(&"jump".into()));
    }
}
