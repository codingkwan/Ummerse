//! 面板系统 - 可停靠的 UI 面板（类 VSCode/Godot 编辑器布局）

use serde::{Deserialize, Serialize};

/// 面板唯一 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelId(String);

impl PanelId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── 面板定义 ──────────────────────────────────────────────────────────────────

/// 编辑器面板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelKind {
    /// 场景树（左侧）
    SceneTree,
    /// 节点属性编辑器（右侧）
    Inspector,
    /// 文件系统浏览器（左下）
    FileSystem,
    /// 资产浏览器（底部）
    AssetBrowser,
    /// 控制台/日志输出（底部）
    Console,
    /// 2D/3D 视口（中央）
    Viewport,
    /// 脚本编辑器（中央）
    ScriptEditor,
    /// AI 助手聊天（右侧）
    AiAssistant,
    /// 插件管理器
    PluginManager,
    /// 项目设置
    ProjectSettings,
    /// 自定义面板
    Custom,
}

impl PanelKind {
    pub fn default_title(&self) -> &str {
        match self {
            Self::SceneTree => "Scene Tree",
            Self::Inspector => "Inspector",
            Self::FileSystem => "File System",
            Self::AssetBrowser => "Assets",
            Self::Console => "Console",
            Self::Viewport => "Viewport",
            Self::ScriptEditor => "Script Editor",
            Self::AiAssistant => "AI Assistant",
            Self::PluginManager => "Plugins",
            Self::ProjectSettings => "Project Settings",
            Self::Custom => "Panel",
        }
    }

    /// 默认停靠区域
    pub fn default_dock(&self) -> DockArea {
        match self {
            Self::SceneTree => DockArea::Left,
            Self::Inspector => DockArea::Right,
            Self::FileSystem => DockArea::LeftBottom,
            Self::AssetBrowser => DockArea::Bottom,
            Self::Console => DockArea::Bottom,
            Self::Viewport => DockArea::Center,
            Self::ScriptEditor => DockArea::Center,
            Self::AiAssistant => DockArea::Right,
            Self::PluginManager => DockArea::Right,
            Self::ProjectSettings => DockArea::Center,
            Self::Custom => DockArea::Right,
        }
    }
}

/// 停靠区域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockArea {
    Left,
    LeftBottom,
    Right,
    Bottom,
    Center,
    Float,
}

/// 面板状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelState {
    Visible,
    Hidden,
    Minimized,
    Maximized,
}

/// 编辑器面板
#[derive(Debug, Clone)]
pub struct EditorPanel {
    pub id: PanelId,
    pub kind: PanelKind,
    pub title: String,
    pub state: PanelState,
    pub dock: DockArea,
    /// 面板宽度（像素，None = 自动）
    pub width: Option<f32>,
    /// 面板高度（像素，None = 自动）
    pub height: Option<f32>,
}

impl EditorPanel {
    pub fn new(kind: PanelKind) -> Self {
        let id = PanelId::new(format!("{:?}", kind).to_lowercase());
        Self {
            id,
            title: kind.default_title().to_string(),
            dock: kind.default_dock(),
            kind,
            state: PanelState::Visible,
            width: None,
            height: None,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.state == PanelState::Visible || self.state == PanelState::Maximized
    }

    pub fn toggle(&mut self) {
        self.state = match self.state {
            PanelState::Visible | PanelState::Maximized => PanelState::Hidden,
            PanelState::Hidden | PanelState::Minimized => PanelState::Visible,
        };
    }
}

// ── 面板布局管理 ──────────────────────────────────────────────────────────────

/// 编辑器面板布局管理器
#[derive(Debug)]
pub struct PanelLayout {
    pub panels: Vec<EditorPanel>,
}

impl PanelLayout {
    /// 创建默认编辑器布局（类 Godot 4 布局）
    pub fn default_layout() -> Self {
        let panels = vec![
            EditorPanel::new(PanelKind::SceneTree),
            {
                let mut p = EditorPanel::new(PanelKind::FileSystem);
                p.height = Some(200.0);
                p
            },
            {
                let mut p = EditorPanel::new(PanelKind::Inspector);
                p.width = Some(280.0);
                p
            },
            {
                let mut p = EditorPanel::new(PanelKind::AiAssistant);
                p.width = Some(320.0);
                p.state = PanelState::Hidden; // 默认隐藏，按需打开
                p
            },
            EditorPanel::new(PanelKind::Viewport),
            {
                let mut p = EditorPanel::new(PanelKind::Console);
                p.height = Some(180.0);
                p
            },
            {
                let mut p = EditorPanel::new(PanelKind::AssetBrowser);
                p.height = Some(180.0);
                p
            },
        ];

        Self { panels }
    }

    /// 显示/隐藏面板
    pub fn toggle_panel(&mut self, kind: PanelKind) {
        if let Some(panel) = self.panels.iter_mut().find(|p| p.kind == kind) {
            panel.toggle();
        }
    }

    /// 获取可见面板列表
    pub fn visible_panels(&self) -> Vec<&EditorPanel> {
        self.panels.iter().filter(|p| p.is_visible()).collect()
    }

    /// 按停靠区域获取面板
    pub fn panels_in_dock(&self, dock: DockArea) -> Vec<&EditorPanel> {
        self.panels
            .iter()
            .filter(|p| p.dock == dock && p.is_visible())
            .collect()
    }

    /// 查找面板
    pub fn find(&self, kind: PanelKind) -> Option<&EditorPanel> {
        self.panels.iter().find(|p| p.kind == kind)
    }
}

impl Default for PanelLayout {
    fn default() -> Self {
        Self::default_layout()
    }
}

// ── 控制台日志 ────────────────────────────────────────────────────────────────

/// 控制台日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsoleLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 控制台消息
#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub message: String,
    pub source: Option<String>,
    pub timestamp: u64,
}

impl ConsoleMessage {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: ConsoleLevel::Info,
            message: message.into(),
            source: None,
            timestamp: current_ms(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: ConsoleLevel::Warning,
            message: message.into(),
            source: None,
            timestamp: current_ms(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: ConsoleLevel::Error,
            message: message.into(),
            source: None,
            timestamp: current_ms(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

fn current_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 控制台面板（集成日志历史）
#[derive(Debug)]
pub struct Console {
    messages: Vec<ConsoleMessage>,
    max_messages: usize,
    filter: Option<ConsoleLevel>,
}

impl Console {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 1000,
            filter: None,
        }
    }

    pub fn push(&mut self, msg: ConsoleMessage) {
        self.messages.push(msg);
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn info(&mut self, text: impl Into<String>) {
        self.push(ConsoleMessage::info(text));
    }

    pub fn warning(&mut self, text: impl Into<String>) {
        self.push(ConsoleMessage::warning(text));
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.push(ConsoleMessage::error(text));
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn set_filter(&mut self, level: Option<ConsoleLevel>) {
        self.filter = level;
    }

    pub fn visible_messages(&self) -> Vec<&ConsoleMessage> {
        if let Some(filter) = self.filter {
            self.messages.iter().filter(|m| m.level == filter).collect()
        } else {
            self.messages.iter().collect()
        }
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}
