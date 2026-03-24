//! 项目管理 - 创建、打开和配置游戏项目

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// 项目配置（存储在 project.toml 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// 项目名称
    pub name: String,
    /// 项目版本
    pub version: String,
    /// 项目描述
    pub description: String,
    /// 引擎版本要求
    pub engine_version: String,
    /// 初始场景路径（相对于项目根）
    pub main_scene: Option<String>,
    /// 导出设置
    pub export: ExportConfig,
    /// 自定义设置
    pub custom: serde_json::Value,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            version: "0.1.0".to_string(),
            description: "A new Ummerse game project".to_string(),
            engine_version: format!("^{}", env!("CARGO_PKG_VERSION")),
            main_scene: Some("scenes/main.uscn".to_string()),
            export: ExportConfig::default(),
            custom: serde_json::Value::Object(Default::default()),
        }
    }
}

/// 导出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub windows: bool,
    pub macos: bool,
    pub linux: bool,
    pub web: bool,
    pub android: bool,
    pub ios: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            windows: true,
            macos: true,
            linux: true,
            web: true,
            android: false,
            ios: false,
        }
    }
}

/// 项目模板
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTemplate {
    /// 空项目
    Empty,
    /// 2D 游戏模板
    Game2d,
    /// 3D 游戏模板
    Game3d,
    /// 平台跳跃游戏（2D）
    Platformer2d,
    /// 顶视角游戏（2D）
    TopDown2d,
    /// 第一人称（3D）
    FirstPerson3d,
}

impl ProjectTemplate {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Empty => "Empty Project",
            Self::Game2d => "2D Game",
            Self::Game3d => "3D Game",
            Self::Platformer2d => "2D Platformer",
            Self::TopDown2d => "2D Top-Down",
            Self::FirstPerson3d => "3D First Person",
        }
    }
}

/// 游戏项目
#[derive(Debug)]
pub struct Project {
    /// 项目根目录
    pub root: PathBuf,
    /// 项目配置
    pub config: ProjectConfig,
}

impl Project {
    /// 打开已有项目
    pub fn open(root: PathBuf) -> Result<Self> {
        let config_path = root.join("project.toml");
        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read {}", config_path.display()))?;
            toml::from_str::<ProjectConfig>(&content)
                .with_context(|| "Failed to parse project.toml")?
        } else {
            // 兼容没有 project.toml 的目录
            let name = root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            ProjectConfig {
                name,
                ..Default::default()
            }
        };

        Ok(Self { root, config })
    }

    /// 创建新项目
    pub fn create(name: &str, root: PathBuf, template: ProjectTemplate) -> Result<Self> {
        // 创建目录结构
        std::fs::create_dir_all(&root)?;

        let config = ProjectConfig {
            name: name.to_string(),
            ..Default::default()
        };

        let project = Self { root, config };

        // 创建标准目录结构
        project.create_directory_structure()?;

        // 根据模板生成初始文件
        project.apply_template(template)?;

        // 保存配置
        project.save_config()?;

        info!(
            "Created project '{}' from template '{}'",
            name,
            template.display_name()
        );
        Ok(project)
    }

    /// 创建标准目录结构
    fn create_directory_structure(&self) -> Result<()> {
        let dirs = [
            "assets",
            "assets/textures",
            "assets/audio",
            "assets/fonts",
            "assets/shaders",
            "assets/meshes",
            "scenes",
            "scripts",
            "plugins",
        ];
        for dir in dirs {
            std::fs::create_dir_all(self.root.join(dir))?;
        }
        Ok(())
    }

    /// 应用项目模板
    fn apply_template(&self, template: ProjectTemplate) -> Result<()> {
        match template {
            ProjectTemplate::Empty => {
                // 创建空的主场景
                let scene_content = r#"{
  "name": "Main",
  "version": 1,
  "nodes": [],
  "root": null,
  "metadata": {
    "description": "",
    "author": "",
    "tags": [],
    "asset_dependencies": []
  }
}"#;
                std::fs::write(self.root.join("scenes/main.uscn"), scene_content)?;
            }
            ProjectTemplate::Game2d
            | ProjectTemplate::Platformer2d
            | ProjectTemplate::TopDown2d => {
                self.apply_2d_template(template)?;
            }
            ProjectTemplate::Game3d | ProjectTemplate::FirstPerson3d => {
                self.apply_3d_template(template)?;
            }
        }
        Ok(())
    }

    fn apply_2d_template(&self, _template: ProjectTemplate) -> Result<()> {
        // 创建 2D 主场景
        let scene_content = serde_json::json!({
            "name": "Main",
            "version": 1,
            "nodes": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "name": "Root",
                    "node_type": "Node2d",
                    "enabled": true,
                    "visible": true,
                    "tags": [],
                    "parent": null,
                    "children": ["00000000-0000-0000-0000-000000000002"],
                    "properties": {}
                },
                {
                    "id": "00000000-0000-0000-0000-000000000002",
                    "name": "Camera2d",
                    "node_type": "Camera2d",
                    "enabled": true,
                    "visible": true,
                    "tags": [],
                    "parent": "00000000-0000-0000-0000-000000000001",
                    "children": [],
                    "properties": { "is_current": true, "zoom": 1.0 }
                }
            ],
            "root": "00000000-0000-0000-0000-000000000001",
            "metadata": {
                "description": "Main 2D scene",
                "author": "",
                "tags": ["2d"],
                "asset_dependencies": []
            }
        });
        std::fs::write(
            self.root.join("scenes/main.uscn"),
            serde_json::to_string_pretty(&scene_content)?,
        )?;

        // 创建示例脚本
        let script = r#"// 玩家控制器示例脚本（Wasm 脚本存根）
// 需要编译为 .wasm 后挂载到节点

export fn ready() {
    engine_log("info", "Player ready!");
}

export fn process(delta: f32) {
    // 每帧逻辑
}
"#;
        std::fs::write(self.root.join("scripts/player.ws"), script)?;
        Ok(())
    }

    fn apply_3d_template(&self, _template: ProjectTemplate) -> Result<()> {
        let scene_content = serde_json::json!({
            "name": "Main",
            "version": 1,
            "nodes": [
                {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "name": "Root",
                    "node_type": "Node3d",
                    "enabled": true,
                    "visible": true,
                    "tags": [],
                    "parent": null,
                    "children": ["00000000-0000-0000-0000-000000000002", "00000000-0000-0000-0000-000000000003"],
                    "properties": {}
                },
                {
                    "id": "00000000-0000-0000-0000-000000000002",
                    "name": "Camera3d",
                    "node_type": "Camera3d",
                    "enabled": true,
                    "visible": true,
                    "tags": [],
                    "parent": "00000000-0000-0000-0000-000000000001",
                    "children": [],
                    "properties": { "fov": 60.0, "near": 0.1, "far": 1000.0 }
                },
                {
                    "id": "00000000-0000-0000-0000-000000000003",
                    "name": "DirectionalLight",
                    "node_type": "DirectionalLight3d",
                    "enabled": true,
                    "visible": true,
                    "tags": [],
                    "parent": "00000000-0000-0000-0000-000000000001",
                    "children": [],
                    "properties": { "color": [1.0, 0.95, 0.8, 1.0], "intensity": 3.0 }
                }
            ],
            "root": "00000000-0000-0000-0000-000000000001",
            "metadata": {
                "description": "Main 3D scene",
                "author": "",
                "tags": ["3d"],
                "asset_dependencies": []
            }
        });
        std::fs::write(
            self.root.join("scenes/main.uscn"),
            serde_json::to_string_pretty(&scene_content)?,
        )?;
        Ok(())
    }

    /// 保存项目配置
    pub fn save_config(&self) -> Result<()> {
        let toml_str =
            toml::to_string_pretty(&self.config).context("Failed to serialize project config")?;
        std::fs::write(self.root.join("project.toml"), toml_str)?;
        Ok(())
    }

    /// 项目名称
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// 绝对路径
    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    /// 主场景路径
    pub fn main_scene_path(&self) -> Option<PathBuf> {
        self.config.main_scene.as_ref().map(|p| self.root.join(p))
    }

    /// 资产目录
    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }

    /// 场景目录
    pub fn scenes_dir(&self) -> PathBuf {
        self.root.join("scenes")
    }

    /// 脚本目录
    pub fn scripts_dir(&self) -> PathBuf {
        self.root.join("scripts")
    }

    /// 插件目录
    pub fn plugins_dir(&self) -> PathBuf {
        self.root.join("plugins")
    }
}
