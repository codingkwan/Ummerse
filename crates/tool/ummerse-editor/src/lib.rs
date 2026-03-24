//! # Ummerse Editor
//!
//! 可视化编辑器，参考 VSCode + Cline 设计模式：
//!
//! ## 架构
//! - **工作区（Workspace）**：项目文件夹管理
//! - **面板系统（Panel）**：可停靠的 UI 面板（场景树、属性、资产浏览等）
//! - **命令面板（Command Palette）**：类 VSCode 的命令搜索
//! - **AI 助手（AI Assistant）**：Cline 风格的 AI 工具调用
//! - **视口（Viewport）**：2D/3D 场景可视化渲染区
//! - **代码编辑器（Code Editor）**：脚本编辑，语法高亮

pub mod ai_assistant;
pub mod command_palette;
pub mod panel;
pub mod project;
pub mod ui;
pub mod viewport;

pub use ai_assistant::{AiAssistant, AiMessage, AiRole};
pub use command_palette::{Command, CommandPalette};
pub use panel::{EditorPanel, PanelId, PanelLayout};
pub use project::{Project, ProjectConfig, ProjectTemplate};
pub use viewport::{Viewport2d, Viewport3d, ViewportMode};

use std::path::PathBuf;
use tracing::info;

/// 编辑器版本
pub const EDITOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 编辑器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// 场景编辑模式
    Scene,
    /// 脚本编辑模式
    Script,
    /// 资产浏览器模式
    Assets,
    /// 游戏预览模式（运行游戏）
    GamePreview,
}

/// 编辑器应用主体
pub struct EditorApp {
    /// 当前项目
    pub project: Option<Project>,
    /// 当前编辑器模式
    pub mode: EditorMode,
    /// 命令面板
    pub command_palette: CommandPalette,
    /// AI 助手
    pub ai_assistant: AiAssistant,
    /// 面板布局
    pub panel_layout: PanelLayout,
}

impl std::fmt::Debug for EditorApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorApp")
            .field("project", &self.project.as_ref().map(|p| p.name()))
            .field("mode", &self.mode)
            .field("panel_count", &self.panel_layout.panels.len())
            .field("ai_message_count", &self.ai_assistant.messages.len())
            .finish_non_exhaustive()
    }
}

impl EditorApp {
    pub fn new() -> Self {
        info!("Ummerse Editor v{} initializing", EDITOR_VERSION);
        Self {
            project: None,
            mode: EditorMode::Scene,
            command_palette: CommandPalette::new(),
            ai_assistant: AiAssistant::new(),
            panel_layout: PanelLayout::default(),
        }
    }

    /// 打开项目
    pub fn open_project(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let project = Project::open(path)?;
        info!("Opened project: {}", project.config.name);
        self.project = Some(project);
        Ok(())
    }

    /// 创建新项目
    pub fn create_project(
        &mut self,
        name: &str,
        path: PathBuf,
        template: ProjectTemplate,
    ) -> anyhow::Result<()> {
        let project = Project::create(name, path, template)?;
        info!("Created project: {}", project.config.name);
        self.project = Some(project);
        Ok(())
    }

    /// 关闭项目
    pub fn close_project(&mut self) {
        if let Some(proj) = &self.project {
            info!("Closing project: {}", proj.config.name);
        }
        self.project = None;
    }

    /// 切换编辑器模式
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// 是否有活跃项目
    pub fn has_project(&self) -> bool {
        self.project.is_some()
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::new()
    }
}
