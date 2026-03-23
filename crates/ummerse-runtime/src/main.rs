//! Ummerse 游戏运行时可执行入口
//!
//! 用法：
//!   ummerse-runtime [--config path/to/engine.toml] [--scene path/to/main.uscn]

use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};
use ummerse_runtime::GameRuntime;

fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .init();

    // 解析命令行参数
    let args = parse_args();

    tracing::info!(
        "Ummerse Runtime v{} starting on {}",
        env!("CARGO_PKG_VERSION"),
        ummerse_runtime::platform::Platform::detect()
    );

    // 加载运行时配置
    let runtime = if let Some(config_path) = &args.config {
        tracing::info!("Loading config from: {}", config_path.display());
        GameRuntime::from_file(config_path.to_str().unwrap())?
    } else {
        tracing::info!("Using default engine config");
        GameRuntime::default()
    };

    // 启动运行时
    runtime.run();

    Ok(())
}

/// 命令行参数
struct CliArgs {
    /// 配置文件路径
    config: Option<PathBuf>,
    /// 初始场景路径
    scene: Option<PathBuf>,
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = std::env::args().collect();
    let mut config = None;
    let mut scene = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => {
                i += 1;
                if i < args.len() {
                    config = Some(PathBuf::from(&args[i]));
                }
            }
            "--scene" | "-s" => {
                i += 1;
                if i < args.len() {
                    scene = Some(PathBuf::from(&args[i]));
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                println!("ummerse-runtime {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
            }
        }
        i += 1;
    }

    CliArgs { config, scene }
}

fn print_help() {
    println!(
        r#"Ummerse Runtime v{}

USAGE:
    ummerse-runtime [OPTIONS]

OPTIONS:
    -c, --config <PATH>    Path to engine configuration file (TOML)
    -s, --scene  <PATH>    Path to initial scene file (.uscn)
    -v, --version          Print version and exit
    -h, --help             Print this help message
"#,
        env!("CARGO_PKG_VERSION")
    );
}
