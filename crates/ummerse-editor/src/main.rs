//! Ummerse 编辑器可执行入口
//!
//! 基于 Bevy 渲染引擎 + Taffy 布局的 VSCode + Cline 风格游戏编辑器

use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use ummerse_editor::EditorApp;

fn main() -> anyhow::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    tracing::info!(
        "Ummerse Editor v{} starting",
        ummerse_editor::EDITOR_VERSION
    );

    // 解析命令行参数
    let args = parse_args();

    // 初始化编辑器
    let mut editor = EditorApp::new();

    // 如果指定了项目路径，则打开项目
    if let Some(project_path) = args.project {
        tracing::info!("Opening project: {}", project_path.display());
        editor.open_project(project_path)?;
    }

    tracing::info!("Editor initialized successfully");
    tracing::info!(
        "Panels: {} visible",
        editor.panel_layout.visible_panels().len()
    );
    tracing::info!("Commands: {} registered", editor.command_palette.len());

    // TODO: 启动 Bevy 应用窗口
    // 当前版本输出初始化信息后退出（完整 UI 需要 Bevy 集成）
    tracing::warn!(
        "Editor UI rendering is not yet implemented - requires Bevy/WGPU window integration"
    );
    tracing::info!("Editor stub initialized successfully. Ready for UI integration.");

    Ok(())
}

/// 命令行参数
struct CliArgs {
    /// 项目目录路径
    project: Option<PathBuf>,
    /// 主题名称（dark/light）
    #[allow(dead_code)]
    theme: String,
    /// 窗口尺寸
    #[allow(dead_code)]
    window_size: Option<(u32, u32)>,
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut project = None;
    let mut theme = "dark".to_string();
    let mut window_size = None;

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
                        if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            window_size = Some((w, h));
                        }
                    }
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                println!("ummerse-editor {}", ummerse_editor::EDITOR_VERSION);
                std::process::exit(0);
            }
            other => {
                // 直接作为项目路径
                if !other.starts_with('-') && project.is_none() {
                    project = Some(PathBuf::from(other));
                } else {
                    eprintln!("Unknown argument: {}", other);
                }
            }
        }
        i += 1;
    }

    CliArgs {
        project,
        theme,
        window_size,
    }
}

fn print_help() {
    println!(
        r#"Ummerse Editor v{}

USAGE:
    ummerse-editor [OPTIONS] [PROJECT_PATH]

ARGUMENTS:
    PROJECT_PATH    Path to project directory (optional)

OPTIONS:
    -p, --project <PATH>    Path to project directory
    --theme <THEME>         UI theme: dark (default) or light
    --size <WxH>            Window size (e.g., 1920x1080)
    -v, --version           Print version and exit
    -h, --help              Print this help message

EXAMPLES:
    ummerse-editor                          # Open with no project
    ummerse-editor ~/games/my_game          # Open a project
    ummerse-editor --theme light            # Use light theme
"#,
        ummerse_editor::EDITOR_VERSION
    );
}
