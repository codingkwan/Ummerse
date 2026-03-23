//! 输入系统 - 键盘、鼠标、游戏手柄输入处理
//!
//! 提供帧级的输入状态查询（按下/保持/释放），
//! 参考 Godot 4 的 Input singleton 设计。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ── 键码 ──────────────────────────────────────────────────────────────────────

/// 键盘按键码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    // 字母
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
    // 数字
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
    // 控制键
    Escape,
    Enter,
    Space,
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
    LMeta,
    RMeta,
    // 小键盘
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    NumAdd,
    NumSub,
    NumMul,
    NumDiv,
    NumEnter,
    NumDot,
    // 其他
    Semicolon,
    Comma,
    Period,
    Slash,
    Backslash,
    LeftBracket,
    RightBracket,
    Quote,
    Backtick,
    Minus,
    Equals,
    CapsLock,
    NumLock,
    ScrollLock,
    PrintScreen,
    Pause,
    Unknown(u32),
}

/// 鼠标按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
    Other(u8),
}

/// 游戏手柄按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    South, // A / Cross
    East,  // B / Circle
    West,  // X / Square
    North, // Y / Triangle
    LBumper,
    RBumper,
    LTrigger,
    RTrigger,
    Select,
    Start,
    Guide,
    LStick,
    RStick,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

/// 游戏手柄轴
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

// ── 输入动作映射 ──────────────────────────────────────────────────────────────

/// 输入绑定（可以是键盘键、鼠标键或手柄键）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
    Gamepad { id: u8, button: GamepadButton },
}

/// 输入动作（类似 Godot 的 Action）
#[derive(Debug, Clone)]
pub struct InputAction {
    pub name: String,
    pub bindings: Vec<InputBinding>,
    pub deadzone: f32,
}

impl InputAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: Vec::new(),
            deadzone: 0.2,
        }
    }

    pub fn with_key(mut self, key: KeyCode) -> Self {
        self.bindings.push(InputBinding::Key(key));
        self
    }

    pub fn with_mouse(mut self, button: MouseButton) -> Self {
        self.bindings.push(InputBinding::Mouse(button));
        self
    }
}

// ── 输入状态 ──────────────────────────────────────────────────────────────────

/// 每帧输入状态快照
#[derive(Debug, Default, Clone)]
pub struct InputState {
    // 键盘
    keys_pressed: HashSet<KeyCode>,  // 本帧刚按下
    keys_held: HashSet<KeyCode>,     // 本帧保持按下
    keys_released: HashSet<KeyCode>, // 本帧刚释放

    // 鼠标
    mouse_pressed: HashSet<MouseButton>,
    mouse_held: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    /// 当前鼠标位置（屏幕像素坐标）
    pub mouse_position: (f32, f32),
    /// 本帧鼠标移动量
    pub mouse_delta: (f32, f32),
    /// 鼠标滚轮
    pub scroll_delta: (f32, f32),

    // 手柄轴
    gamepad_axes: HashMap<(u8, GamepadAxis), f32>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    // ── 键盘查询 ───────────────────────────────────────────────────────────

    /// 键是否刚被按下（仅本帧第一帧返回 true）
    #[inline]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// 键是否持续按下
    #[inline]
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    /// 键是否刚被释放
    #[inline]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    // ── 鼠标查询 ───────────────────────────────────────────────────────────

    #[inline]
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    #[inline]
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }

    #[inline]
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_released.contains(&button)
    }

    // ── 手柄查询 ───────────────────────────────────────────────────────────

    /// 获取手柄轴值（-1.0 ~ 1.0）
    pub fn gamepad_axis(&self, id: u8, axis: GamepadAxis) -> f32 {
        *self.gamepad_axes.get(&(id, axis)).unwrap_or(&0.0)
    }
}

// ── 输入管理器 ────────────────────────────────────────────────────────────────

/// 输入管理器 - 集中处理所有输入事件
pub struct InputManager {
    /// 当前帧输入状态
    pub state: InputState,
    /// 动作映射表
    actions: HashMap<String, InputAction>,
    /// 上一帧按住的键（用于计算 just_pressed/released）
    prev_keys: HashSet<KeyCode>,
    prev_mouse: HashSet<MouseButton>,
}

impl InputManager {
    pub fn new() -> Self {
        let mut manager = Self {
            state: InputState::new(),
            actions: HashMap::new(),
            prev_keys: HashSet::new(),
            prev_mouse: HashSet::new(),
        };
        manager.register_default_actions();
        manager
    }

    /// 注册默认输入动作
    fn register_default_actions(&mut self) {
        let actions = vec![
            InputAction::new("ui_accept")
                .with_key(KeyCode::Enter)
                .with_key(KeyCode::Space),
            InputAction::new("ui_cancel").with_key(KeyCode::Escape),
            InputAction::new("ui_left").with_key(KeyCode::Left),
            InputAction::new("ui_right").with_key(KeyCode::Right),
            InputAction::new("ui_up").with_key(KeyCode::Up),
            InputAction::new("ui_down").with_key(KeyCode::Down),
            InputAction::new("move_forward")
                .with_key(KeyCode::W)
                .with_key(KeyCode::Up),
            InputAction::new("move_backward")
                .with_key(KeyCode::S)
                .with_key(KeyCode::Down),
            InputAction::new("move_left")
                .with_key(KeyCode::A)
                .with_key(KeyCode::Left),
            InputAction::new("move_right")
                .with_key(KeyCode::D)
                .with_key(KeyCode::Right),
            InputAction::new("jump").with_key(KeyCode::Space),
            InputAction::new("run").with_key(KeyCode::LShift),
        ];

        for action in actions {
            self.actions.insert(action.name.clone(), action);
        }
    }

    /// 注册自定义输入动作
    pub fn register_action(&mut self, action: InputAction) {
        self.actions.insert(action.name.clone(), action);
    }

    /// 每帧开始时调用，重置单帧状态
    pub fn begin_frame(&mut self) {
        self.state.keys_pressed.clear();
        self.state.keys_released.clear();
        self.state.mouse_pressed.clear();
        self.state.mouse_released.clear();
        self.state.mouse_delta = (0.0, 0.0);
        self.state.scroll_delta = (0.0, 0.0);
    }

    /// 处理键盘按下事件
    pub fn on_key_down(&mut self, key: KeyCode) {
        if !self.prev_keys.contains(&key) {
            self.state.keys_pressed.insert(key);
        }
        self.state.keys_held.insert(key);
        self.prev_keys.insert(key);
    }

    /// 处理键盘释放事件
    pub fn on_key_up(&mut self, key: KeyCode) {
        self.state.keys_held.remove(&key);
        self.state.keys_released.insert(key);
        self.prev_keys.remove(&key);
    }

    /// 处理鼠标按下
    pub fn on_mouse_down(&mut self, button: MouseButton) {
        if !self.prev_mouse.contains(&button) {
            self.state.mouse_pressed.insert(button);
        }
        self.state.mouse_held.insert(button);
        self.prev_mouse.insert(button);
    }

    /// 处理鼠标释放
    pub fn on_mouse_up(&mut self, button: MouseButton) {
        self.state.mouse_held.remove(&button);
        self.state.mouse_released.insert(button);
        self.prev_mouse.remove(&button);
    }

    /// 处理鼠标移动
    pub fn on_mouse_move(&mut self, x: f32, y: f32, dx: f32, dy: f32) {
        self.state.mouse_position = (x, y);
        self.state.mouse_delta = (self.state.mouse_delta.0 + dx, self.state.mouse_delta.1 + dy);
    }

    /// 处理滚轮
    pub fn on_scroll(&mut self, dx: f32, dy: f32) {
        self.state.scroll_delta = (
            self.state.scroll_delta.0 + dx,
            self.state.scroll_delta.1 + dy,
        );
    }

    /// 设置手柄轴值
    pub fn on_gamepad_axis(&mut self, id: u8, axis: GamepadAxis, value: f32) {
        self.state.gamepad_axes.insert((id, axis), value);
    }

    // ── 动作查询 ───────────────────────────────────────────────────────────

    /// 动作是否刚被触发（单帧）
    pub fn is_action_just_pressed(&self, action: &str) -> bool {
        if let Some(action) = self.actions.get(action) {
            action.bindings.iter().any(|b| match b {
                InputBinding::Key(k) => self.state.is_key_just_pressed(*k),
                InputBinding::Mouse(m) => self.state.is_mouse_just_pressed(*m),
                InputBinding::Gamepad { .. } => false, // TODO
            })
        } else {
            false
        }
    }

    /// 动作是否持续触发
    pub fn is_action_pressed(&self, action: &str) -> bool {
        if let Some(action) = self.actions.get(action) {
            action.bindings.iter().any(|b| match b {
                InputBinding::Key(k) => self.state.is_key_pressed(*k),
                InputBinding::Mouse(m) => self.state.is_mouse_pressed(*m),
                InputBinding::Gamepad { .. } => false,
            })
        } else {
            false
        }
    }

    /// 动作是否刚被释放
    pub fn is_action_just_released(&self, action: &str) -> bool {
        if let Some(action) = self.actions.get(action) {
            action.bindings.iter().any(|b| match b {
                InputBinding::Key(k) => self.state.is_key_just_released(*k),
                InputBinding::Mouse(m) => self.state.is_mouse_just_released(*m),
                InputBinding::Gamepad { .. } => false,
            })
        } else {
            false
        }
    }

    /// 获取 1D 输入轴（正向 - 负向，范围 -1.0 ~ 1.0）
    pub fn get_axis(&self, negative_action: &str, positive_action: &str) -> f32 {
        let positive = if self.is_action_pressed(positive_action) {
            1.0
        } else {
            0.0
        };
        let negative = if self.is_action_pressed(negative_action) {
            1.0
        } else {
            0.0
        };
        positive - negative
    }

    /// 获取 2D 输入向量（归一化）
    pub fn get_vector(&self, left: &str, right: &str, up: &str, down: &str) -> (f32, f32) {
        let x = self.get_axis(left, right);
        let y = self.get_axis(down, up);
        // 归一化
        let len = (x * x + y * y).sqrt();
        if len > 1.0 {
            (x / len, y / len)
        } else {
            (x, y)
        }
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
    fn test_key_just_pressed() {
        let mut manager = InputManager::new();
        manager.begin_frame();
        manager.on_key_down(KeyCode::W);
        assert!(manager.state.is_key_just_pressed(KeyCode::W));
        assert!(manager.state.is_key_pressed(KeyCode::W));
        assert!(!manager.state.is_key_just_released(KeyCode::W));
    }

    #[test]
    fn test_key_held_not_just_pressed() {
        let mut manager = InputManager::new();
        // 第一帧：按下
        manager.begin_frame();
        manager.on_key_down(KeyCode::W);
        // 第二帧：保持按下
        manager.begin_frame();
        manager.on_key_down(KeyCode::W);
        assert!(!manager.state.is_key_just_pressed(KeyCode::W));
        assert!(manager.state.is_key_pressed(KeyCode::W));
    }

    #[test]
    fn test_get_axis() {
        let mut manager = InputManager::new();
        manager.begin_frame();
        manager.on_key_down(KeyCode::D); // move_right
        let axis = manager.get_axis("move_left", "move_right");
        assert!((axis - 1.0).abs() < 0.001);
    }
}
