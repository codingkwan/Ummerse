//! Ummerse 编辑器可执行入口
//!
//! 基于 Bevy 渲染引擎 + Taffy 布局的 VSCode + Cline 风格游戏编辑器。
//!
//! ## 架构
//! - **Bevy App**：窗口管理、渲染、ECS 调度
//! - **EditorApp**：编辑器状态机（项目、面板、AI 助手等）
//! - **ToolRegistry**：类 Cline 的工具调用系统
//! - **PluginHost**：Wasm 插件沙箱管理

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};
use std::path::PathBuf;
use ummerse_editor::{EDITOR_VERSION, EditorApp, EditorMode};
use ummerse_plugin::host::PluginHost;
use ummerse_plugin::tool::ToolRegistry;
use ummerse_runtime::systems::UmmerseCorePlugin;

fn main() -> anyhow::Result<()> {
    // 初始化结构化日志
    init_logging();

    tracing::info!(version = EDITOR_VERSION, "Ummerse Editor starting");

    let cli = CliArgs::parse();

    // 初始化编辑器应用状态
    let mut editor_app = EditorApp::new();

    // 若提供了项目路径则立即打开
    if let Some(ref path) = cli.project {
        tracing::info!(path = %path.display(), "Opening project");
        editor_app.open_project(path.clone())?;
    }

    tracing::info!(
        commands = editor_app.command_palette.len(),
        panels = editor_app.panel_layout.visible_panels().len(),
        "Editor state initialized"
    );

    // 构建并启动 Bevy 应用
    build_bevy_app(cli, editor_app).run();
    Ok(())
}

/// 构建 Bevy 编辑器应用
fn build_bevy_app(cli: CliArgs, editor_app: EditorApp) -> App {
    let mut app = App::new();

    let (width, height) = cli.window_size.unwrap_or((1600, 900));

    // ── DefaultPlugins（窗口 + 渲染 + 输入）────────────────────────────────────
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!(
                        "Ummerse Editor v{} – {}",
                        EDITOR_VERSION,
                        editor_app
                            .project
                            .as_ref()
                            .map(|p| p.config.name.as_str())
                            .unwrap_or("No Project")
                    ),
                    resolution: WindowResolution::new(width as f32, height as f32),
                    resizable: true,
                    present_mode: if cli.vsync {
                        PresentMode::AutoVsync
                    } else {
                        PresentMode::AutoNoVsync
                    },
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(bevy::log::LogPlugin {
                // 使用 tracing-subscriber（已在 init_logging 初始化）
                level: bevy::log::Level::WARN,
                filter: "wgpu=warn,naga=warn".to_string(),
                ..Default::default()
            }),
    );

    // ── Ummerse 核心 ECS 插件 ──────────────────────────────────────────────────
    app.add_plugins(UmmerseCorePlugin);

    // ── 编辑器状态资源 ─────────────────────────────────────────────────────────
    app.insert_resource(EditorStateResource(editor_app))
        .insert_resource(EditorToolRegistryResource(ToolRegistry::new()))
        .insert_resource(EditorPluginHostResource(PluginHost::new()))
        .insert_resource(EditorConfig {
            theme: cli.theme,
            show_fps: cli.show_fps,
        });

    // ── 编辑器系统 ─────────────────────────────────────────────────────────────
    app.add_systems(Startup, editor_startup_system)
        .add_systems(Update, (editor_update_system, handle_editor_shortcuts))
        .add_systems(Last, editor_frame_end_system);

    app
}

// ── Bevy Resources ────────────────────────────────────────────────────────────

/// 编辑器应用状态 Resource
#[derive(Resource, Debug)]
pub struct EditorStateResource(pub EditorApp);

/// 工具注册表 Resource
#[derive(Resource, Debug)]
pub struct EditorToolRegistryResource(pub ToolRegistry);

/// 插件宿主 Resource
#[derive(Resource, Debug)]
pub struct EditorPluginHostResource(pub PluginHost);

/// 编辑器配置 Resource
#[derive(Resource, Debug)]
pub struct EditorConfig {
    pub theme: String,
    pub show_fps: bool,
}

// ── Bevy Systems ──────────────────────────────────────────────────────────────

/// 编辑器启动系统
fn editor_startup_system(editor: Res<EditorStateResource>, tools: Res<EditorToolRegistryResource>) {
    let app = &editor.0;

    tracing::info!(
        project = app
            .project
            .as_ref()
            .map(|p| p.config.name.as_str())
            .unwrap_or("none"),
        panels = app.panel_layout.visible_panels().len(),
        tools = tools.0.len(),
        "Editor ready"
    );
}

/// 编辑器每帧更新系统
fn editor_update_system(editor: ResMut<EditorStateResource>, time: Res<Time>) {
    // 每 5 秒打印一次诊断（仅 trace 级别）
    if (time.elapsed_secs() % 5.0) < time.delta_secs() {
        tracing::trace!(
            frame     = time.elapsed_secs(),
            mode      = ?editor.0.mode,
            has_proj  = editor.0.has_project(),
            "Editor tick"
        );
    }
}

/// 编辑器快捷键处理系统
fn handle_editor_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor: ResMut<EditorStateResource>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Ctrl+Shift+A: 切换 AI 助手面板
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyA) {
        use ummerse_editor::panel::PanelKind;
        editor.0.panel_layout.toggle_panel(PanelKind::AiAssistant);
        tracing::debug!("Toggled AI assistant panel");
    }

    // Ctrl+Shift+P: 打开命令面板
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyP) {
        tracing::debug!("Command palette triggered (TODO: open overlay)");
    }

    // F5: 运行场景预览
    if keyboard.just_pressed(KeyCode::F5) {
        editor.0.set_mode(EditorMode::GamePreview);
        tracing::info!("Entering game preview mode");
    }

    // Escape: 停止预览
    if keyboard.just_pressed(KeyCode::Escape) && editor.0.mode == EditorMode::GamePreview {
        editor.0.set_mode(EditorMode::Scene);
        tracing::info!("Exiting game preview mode");
    }
}

/// 帧末处理（清理临时状态）
fn editor_frame_end_system() {
    // TODO: 刷新 UI 脏标记、提交 undo 历史等
}

// ── 日志初始化 ────────────────────────────────────────────────────────────────

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};

    // 若已初始化则跳过（避免 Bevy LogPlugin 冲突）
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,wgpu=warn,naga=warn,bevy_render=info,bevy_ecs=warn")
    });

    // 使用 try_init，防止重复初始化 panic
    let _ = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(false)
        .compact()
        .try_init();
}

// ── CLI 参数解析 ──────────────────────────────────────────────────────────────

/// 命令行参数
struct CliArgs {
    project: Option<PathBuf>,
    theme: String,
    window_size: Option<(u32, u32)>,
    vsync: bool,
    show_fps: bool,
}

impl CliArgs {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut project = None;
        let mut theme = "dark".to_string();
        let mut window_size = None;
        let mut vsync = true;
        let mut show_fps = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--project" | "-p" => {
                    i += 1;
                    if i < args.len() {
                        project = Some(PathBuf::from(&args[i]));
                    }
                }
                "--theme" => {
                    i += 1;
                    if i < args.len() {
                        theme = args[i].clone();
                    }
                }
                "--size" => {
                    i += 1;
                    if i < args.len() {
                        let parts: Vec<&str> = args[i].split('x').collect();
                        if parts.len() == 2 {
                            if let (Ok(w), Ok(h)) =
                                (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                            {
                                window_size = Some((w, h));
                            }
                        }
                    }
                }
                "--no-vsync" => vsync = false,
                "--show-fps" => show_fps = true,
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                "--version" | "-v" => {
                    println!("ummerse-editor {EDITOR_VERSION}");
                    std::process::exit(0);
                }
                other => {
                    if !other.starts_with('-') && project.is_none() {
                        project = Some(PathBuf::from(other));
                    } else {
                        eprintln!("Unknown argument: {other}");
                    }
                }
            }
            i += 1;
        }

        Self {
            project,
            theme,
            window_size,
            vsync,
            show_fps,
        }
    }

    fn print_help() {
        println!(
            r#"Ummerse Editor v{EDITOR_VERSION}

USAGE:
    ummerse-editor [OPTIONS] [PROJECT_PATH]

ARGUMENTS:
    PROJECT_PATH         Path to project directory (optional)

OPTIONS:
    -p, --project <PATH>   Path to project directory
    --theme <THEME>        UI theme: dark (default) | light
    --size <WxH>           Window size, e.g. 1920x1080
    --no-vsync             Disable vertical sync
    --show-fps             Show FPS overlay
    -v, --version          Print version and exit
    -h, --help             Print this help

EXAMPLES:
    ummerse-editor
    ummerse-editor ~/games/my_game
    ummerse-editor --theme light --size 1920x1080
"#
        );
    }
}
