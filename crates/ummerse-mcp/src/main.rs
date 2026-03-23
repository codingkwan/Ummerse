//! Ummerse MCP Server 可执行入口
//!
//! 启动一个基于 stdio 的 MCP Server，将 Ummerse 引擎功能暴露给
//! Cline 等 AI Agent，实现"AI 直接操控游戏引擎"的核心原型。
//!
//! ## 使用方式
//!
//! ### 1. 直接运行（调试模式）
//! ```sh
//! cargo run -p ummerse-mcp
//! # 然后通过 stdin 手动发送 JSON-RPC 请求：
//! # {"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
//! ```
//!
//! ### 2. 配置到 VS Code Cline 插件
//! 在 Cline MCP Settings 中添加：
//! ```json
//! {
//!   "ummerse": {
//!     "command": "cargo",
//!     "args": ["run", "--release", "-p", "ummerse-mcp"],
//!     "cwd": "/path/to/Ummerse"
//!   }
//! }
//! ```
//!
//! ### 3. 编译后直接使用二进制
//! ```sh
//! cargo build --release -p ummerse-mcp
//! # 二进制位于 target/release/ummerse-mcp
//! ```
//!
//! ## 架构
//! ```text
//! ┌─────────────┐  JSON-RPC/stdio  ┌──────────────────┐  Arc<Mutex<>>  ┌───────────────┐
//! │  Cline/AI   │ ◄──────────────► │  McpServer       │ ◄────────────► │  EngineBridge │
//! │  (Client)   │                  │  (this binary)   │                │  (scene state)│
//! └─────────────┘                  └──────────────────┘                └───────────────┘
//! ```

use tracing_subscriber::EnvFilter;
use ummerse_mcp::{engine_bridge::EngineBridge, mcp_server::McpServer};

fn main() -> anyhow::Result<()> {
    // ── 初始化日志（输出到 stderr，避免污染 stdout 的 JSON-RPC 流）─────────
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("ummerse_mcp=info")),
        )
        .with_writer(std::io::stderr) // 关键：日志写到 stderr，JSON-RPC 走 stdout
        .with_target(false)
        .with_ansi(false) // 禁用 ANSI 颜色码（避免 MCP 客户端解析混乱）
        .compact()
        .init();

    // ── 打印启动横幅到 stderr ────────────────────────────────────────────────
    eprintln!("╔══════════════════════════════════════════════════════════╗");
    eprintln!("║          Ummerse MCP Server - AI Engine Bridge           ║");
    eprintln!("║  Protocol: MCP 2024-11-05 (JSON-RPC 2.0 over stdio)     ║");
    eprintln!("╠══════════════════════════════════════════════════════════╣");
    eprintln!("║  Available Tools:                                        ║");
    eprintln!("║    • move_block     - 相对移动实体                       ║");
    eprintln!("║    • set_position   - 设置实体绝对坐标                   ║");
    eprintln!("║    • spawn_entity   - 生成新实体                         ║");
    eprintln!("║    • despawn_entity - 删除实体                           ║");
    eprintln!("║    • get_scene      - 获取场景快照                       ║");
    eprintln!("║    • get_entity     - 获取实体详情                       ║");
    eprintln!("║    • set_property   - 设置实体属性                       ║");
    eprintln!("║    • list_entities  - 列出所有实体                       ║");
    eprintln!("╚══════════════════════════════════════════════════════════╝");
    eprintln!();

    // ── 创建引擎桥接（带演示场景：MainBlock, RedBlock, GreenCircle）─────────
    let bridge = EngineBridge::new_with_demo();
    tracing::info!("Engine bridge initialized with demo scene (3 entities)");

    // ── 创建并启动 MCP Server ────────────────────────────────────────────────
    let server = McpServer::new(bridge);
    server.run_stdio()?;

    Ok(())
}
