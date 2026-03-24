//! 命令面板 - 类 VSCode 的命令搜索和执行

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 命令 ID 类型别名
pub type CommandId = String;

/// 编辑器命令
#[derive(Debug, Clone)]
pub struct Command {
    pub id: CommandId,
    pub title: String,
    pub description: String,
    pub category: CommandCategory,
    pub keybinding: Option<String>,
    /// 执行命令的回调（函数指针）
    handler: fn() -> CommandResult,
}

impl Command {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        category: CommandCategory,
        handler: fn() -> CommandResult,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            category,
            keybinding: None,
            handler,
        }
    }

    pub fn with_keybinding(mut self, keybinding: impl Into<String>) -> Self {
        self.keybinding = Some(keybinding.into());
        self
    }

    /// 执行命令
    pub fn execute(&self) -> CommandResult {
        (self.handler)()
    }
}

/// 命令结果
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// 成功，可选的输出消息
    Success(Option<String>),
    /// 失败，带错误信息
    Error(String),
    /// 命令需要更多输入（比如弹出对话框）
    NeedsInput(String),
}

/// 命令分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandCategory {
    File,
    Edit,
    Scene,
    Asset,
    Script,
    Build,
    Plugin,
    View,
    Help,
}

impl CommandCategory {
    pub fn display_name(&self) -> &str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::Scene => "Scene",
            Self::Asset => "Assets",
            Self::Script => "Script",
            Self::Build => "Build",
            Self::Plugin => "Plugin",
            Self::View => "View",
            Self::Help => "Help",
        }
    }
}

// ── 命令面板 ──────────────────────────────────────────────────────────────────

/// 命令面板 - 管理和搜索所有命令
#[derive(Debug)]
pub struct CommandPalette {
    commands: HashMap<CommandId, Command>,
    /// 最近使用的命令（最多 10 条）
    recent: Vec<CommandId>,
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut palette = Self {
            commands: HashMap::new(),
            recent: Vec::new(),
        };
        // 注册内置命令
        palette.register_builtin_commands();
        palette
    }

    /// 注册命令
    pub fn register(&mut self, command: Command) {
        self.commands.insert(command.id.clone(), command);
    }

    /// 执行命令
    pub fn execute(&mut self, id: &str) -> CommandResult {
        if let Some(cmd) = self.commands.get(id) {
            // 更新最近使用
            self.recent.retain(|r| r != id);
            self.recent.insert(0, id.to_string());
            self.recent.truncate(10);

            cmd.execute()
        } else {
            CommandResult::Error(format!("Command '{}' not found", id))
        }
    }

    /// 模糊搜索命令
    pub fn search(&self, query: &str) -> Vec<&Command> {
        if query.is_empty() {
            // 返回最近使用的命令
            let mut results: Vec<&Command> = self
                .recent
                .iter()
                .filter_map(|id| self.commands.get(id))
                .collect();
            // 补充一些常用命令
            if results.len() < 5 {
                for cmd in self.commands.values() {
                    if !results.iter().any(|c| c.id == cmd.id) {
                        results.push(cmd);
                        if results.len() >= 10 {
                            break;
                        }
                    }
                }
            }
            return results;
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<(&Command, usize)> = self
            .commands
            .values()
            .filter_map(|cmd| {
                let title_lower = cmd.title.to_lowercase();
                let id_lower = cmd.id.to_lowercase();
                // 计算匹配分数
                let score = if title_lower == query_lower {
                    100
                } else if title_lower.starts_with(&query_lower) {
                    80
                } else if title_lower.contains(&query_lower) {
                    60
                } else if id_lower.contains(&query_lower) {
                    40
                } else if cmd.description.to_lowercase().contains(&query_lower) {
                    20
                } else {
                    return None;
                };
                Some((cmd, score))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(cmd, _)| cmd).take(20).collect()
    }

    /// 获取命令
    pub fn get(&self, id: &str) -> Option<&Command> {
        self.commands.get(id)
    }

    /// 所有命令数量
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// 注册内置命令
    fn register_builtin_commands(&mut self) {
        let cmds = vec![
            Command::new(
                "file.new_project",
                "New Project",
                "Create a new game project",
                CommandCategory::File,
                || CommandResult::NeedsInput("new_project_dialog".to_string()),
            )
            .with_keybinding("Ctrl+Shift+N"),
            Command::new(
                "file.open_project",
                "Open Project",
                "Open an existing game project",
                CommandCategory::File,
                || CommandResult::NeedsInput("open_project_dialog".to_string()),
            )
            .with_keybinding("Ctrl+Shift+O"),
            Command::new(
                "file.save",
                "Save",
                "Save the current file",
                CommandCategory::File,
                || CommandResult::Success(Some("Saved".to_string())),
            )
            .with_keybinding("Ctrl+S"),
            Command::new(
                "file.save_all",
                "Save All",
                "Save all modified files",
                CommandCategory::File,
                || CommandResult::Success(Some("All files saved".to_string())),
            )
            .with_keybinding("Ctrl+Shift+S"),
            Command::new(
                "scene.new_scene",
                "New Scene",
                "Create a new empty scene",
                CommandCategory::Scene,
                || CommandResult::NeedsInput("new_scene_dialog".to_string()),
            ),
            Command::new(
                "scene.run",
                "Run Scene",
                "Run the current scene in game preview",
                CommandCategory::Scene,
                || CommandResult::Success(None),
            )
            .with_keybinding("F5"),
            Command::new(
                "scene.stop",
                "Stop",
                "Stop game preview",
                CommandCategory::Scene,
                || CommandResult::Success(None),
            )
            .with_keybinding("Escape"),
            Command::new(
                "build.export_web",
                "Export for Web",
                "Build and export the game for WebAssembly",
                CommandCategory::Build,
                || CommandResult::NeedsInput("export_web_dialog".to_string()),
            ),
            Command::new(
                "build.export_desktop",
                "Export for Desktop",
                "Build and export the game for desktop platforms",
                CommandCategory::Build,
                || CommandResult::NeedsInput("export_desktop_dialog".to_string()),
            ),
            Command::new(
                "view.toggle_ai",
                "Toggle AI Assistant",
                "Show or hide the AI assistant panel",
                CommandCategory::View,
                || CommandResult::Success(None),
            )
            .with_keybinding("Ctrl+Shift+A"),
            Command::new(
                "view.toggle_scene_tree",
                "Toggle Scene Tree",
                "Show or hide the scene tree panel",
                CommandCategory::View,
                || CommandResult::Success(None),
            ),
            Command::new(
                "help.about",
                "About Ummerse",
                "Show information about the Ummerse game engine",
                CommandCategory::Help,
                || {
                    CommandResult::Success(Some(format!(
                        "Ummerse Engine v{}\nA modern 2D/3D game engine",
                        env!("CARGO_PKG_VERSION")
                    )))
                },
            ),
        ];

        for cmd in cmds {
            self.commands.insert(cmd.id.clone(), cmd);
        }
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}
