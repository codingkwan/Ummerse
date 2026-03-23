//! Ummerse 运行时可执行入口
//!
//! 使用 Bevy 作为 ECS + 窗口 + 渲染后端，
//! 运行游戏项目的主循环。

use tracing_subscriber::EnvFilter;
use ummerse_runtime::{GameAppBuilder, GameRuntime, Platform};

fn main() -> anyhow::Result<()> {
    // 初始化日志（Runtime 模式下 Bevy 的 LogPlugin 会接管，这里仅用于启动前诊断）
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("ummerse=info,bevy=warn")),
        )
        .with_target(false)
        .try_init();

    tracing::info!("=== Ummerse Runtime ===");
    tracing::info!("Platform: {}", Platform::current());
    tracing::info!("Version: {}", ummerse_core::ENGINE_VERSION);

    // 解析命令行参数
    let args = parse_args();

    if args.list_platforms {
        println!("Supported platforms: Windows, macOS, Linux, Web (WASM)");
        return Ok(());
    }

    // 构建并运行游戏
    let mut builder = GameAppBuilder::new()
        .title("Ummerse Game")
        .window_size(1280, 720)
        .vsync(true)
        .physics_fps(60)
        .show_fps(args.show_fps);

    // 从配置文件加载（若指定）
    if let Some(config_path) = &args.config {
        tracing::info!("Loading config from: {}", config_path);
        let runtime = GameRuntime::from_file(config_path)?;
        let mut app = runtime.build_bevy_app();

        // 注册演示系统
        use bevy::prelude::*;
        app.add_systems(Startup, setup_demo_scene);
        app.add_systems(Update, demo_rotate_system);
        app.run();
    } else {
        // 默认演示场景
        builder = builder
            .setup(|app| {
                use bevy::prelude::*;
                app.add_systems(Startup, setup_demo_scene);
                app.add_systems(Update, demo_rotate_system);
            });
        builder.run();
    }

    Ok(())
}

// ── 演示场景 ──────────────────────────────────────────────────────────────────

use bevy::prelude::*;

/// 设置演示场景：2D 相机 + 几个色块精灵
fn setup_demo_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 2D 正交相机
    commands.spawn(Camera2d::default());

    // 中央彩色方块
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.6, 1.0)))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        DemoRotate { speed: 1.0 },
    ));

    // 左边红色方块
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(60.0, 60.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.2, 0.2)))),
        Transform::from_xyz(-200.0, 0.0, 0.0),
        DemoRotate { speed: -1.5 },
    ));

    // 右边绿色圆形
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(40.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.9, 0.3)))),
        Transform::from_xyz(200.0, 0.0, 0.0),
        DemoRotate { speed: 2.0 },
    ));

    tracing::info!("Demo scene initialized with 3 shapes");
}

/// 旋转标记组件
#[derive(Component)]
struct DemoRotate {
    speed: f32,
}

/// 旋转演示系统
fn demo_rotate_system(time: Res<Time>, mut query: Query<(&DemoRotate, &mut Transform)>) {
    for (rotate, mut transform) in query.iter_mut() {
        transform.rotate_z(rotate.speed * time.delta_secs());
    }
}

// ── 命令行参数解析 ─────────────────────────────────────────────────────────────

struct CliArgs {
    /// 配置文件路径
    config: Option<String>,
    /// 是否显示 FPS
    show_fps: bool,
    /// 列出支持平台
    list_platforms: bool,
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut config = None;
    let mut show_fps = false;
    let mut list_platforms = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => {
                i += 1;
                if i < args.len() {
                    config = Some(args[i].clone());
                }
            }
            "--fps" => show_fps = true,
            "--list-platforms" => list_platforms = true,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                println!("ummerse-runtime {}", ummerse_core::ENGINE_VERSION);
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    CliArgs { config, show_fps, list_platforms }
}

fn print_help() {
    println!(
        r#"Ummerse Runtime v{}

USAGE:
    ummerse-runtime [OPTIONS]

OPTIONS:
    -c, --config <PATH>    Load engine config from TOML file
    --fps                  Show FPS counter
    --list-platforms       List supported target platforms
    -v, --version          Print version
    -h, --help             Print this help

EXAMPLES:
    ummerse-runtime                         # Run with defaults
    ummerse-runtime --config engine.toml    # Load custom config
"#,
        ummerse_core::ENGINE_VERSION
    );
}
