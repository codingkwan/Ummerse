//! # Ummerse MCP Server
//!
//! MCP (Model Context Protocol) Server 实现，让 AI Agent（如 Cline）能够：
//! - 控制无头 Bevy 引擎中的实体（移动、创建、删除）
//! - 查询场景状态（实体列表、位置信息）
//! - 执行引擎命令
//!
//! ## 架构
//! ```text
//! ┌─────────────┐   JSON-RPC/stdio   ┌───────────────┐   channel   ┌──────────────┐
//! │  Cline/AI   │ ◄──────────────────► │  MCP Server   │ ◄─────────► │ Bevy Engine  │
//! │  (Client)   │                     │  (this crate) │            │ (headless)   │
//! └─────────────┘                     └───────────────┘            └──────────────┘
//! ```
//!
//! ## 协议
//! 使用 MCP 标准协议（JSON-RPC 2.0 over stdio）

pub mod engine_bridge;
pub mod mcp_server;
pub mod tools;
