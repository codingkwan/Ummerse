# Ummerse MCP Server

> **AI Agent ↔ 游戏引擎** 的桥接层 —— 让 Cline 能够直接操控 Ummerse 引擎中的实体。

## 架构总览

```text
┌─────────────┐  JSON-RPC 2.0  ┌────────────────────┐  Arc<Mutex<>>  ┌────────────────┐
│  Cline/AI   │ ◄────stdio────► │    McpServer       │ ◄────────────► │  EngineBridge  │
│  (Client)   │                 │  (ummerse-mcp bin) │                │  (场景状态)     │
└─────────────┘                 └────────────────────┘                └────────────────┘
```

## 快速开始

### 1. 编译 MCP Server

```sh
# 在 Ummerse 项目根目录下
cargo build --release -p ummerse-mcp
# 二进制位于: target/release/ummerse-mcp.exe（Windows）
#              target/release/ummerse-mcp（Linux/macOS）
```

### 2. 调试运行（手动测试）

```sh
cargo run -p ummerse-mcp
```

然后在 stdin 输入 JSON-RPC 请求（每行一条）：

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_scene","arguments":{}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"move_block","arguments":{"name":"MainBlock","dx":50}}}
```

### 3. 配置到 VS Code Cline 插件

打开 VS Code，进入 **Cline** 扩展设置 → **MCP Servers** → 点击 **Edit Config**，添加：

```json
{
  "mcpServers": {
    "ummerse": {
      "command": "cargo",
      "args": ["run", "--release", "-p", "ummerse-mcp"],
      "cwd": "C:\\Users\\你的用户名\\coding\\Ummerse",
      "env": {
        "RUST_LOG": "ummerse_mcp=info"
      }
    }
  }
}
```

**或者**，如果已经编译好二进制（推荐，启动更快）：

```json
{
  "mcpServers": {
    "ummerse": {
      "command": "C:\\Users\\你的用户名\\coding\\Ummerse\\target\\release\\ummerse-mcp.exe",
      "args": [],
      "env": {
        "RUST_LOG": "ummerse_mcp=info"
      }
    }
  }
}
```

### 4. 与 Cline 对话示例

配置完成后，在 Cline 聊天框中输入：

```
让 MainBlock 向右移动 100 像素
```

```
在坐标 (300, 200) 生成一个名为 "Enemy" 的方块
```

```
显示当前场景中所有实体的位置
```

```
隐藏 GreenCircle
```

---

## 可用工具（AI 工具列表）

| 工具名 | 功能 | 必填参数 | 可选参数 |
|--------|------|----------|----------|
| `move_block` | 相对移动实体（增量） | `name` | `dx`, `dy` |
| `set_position` | 设置实体绝对坐标 | `name`, `x`, `y` | — |
| `spawn_entity` | 生成新实体 | `name` | `kind`, `x`, `y` |
| `despawn_entity` | 删除实体 | `name` | — |
| `get_scene` | 获取完整场景快照 | — | — |
| `get_entity` | 获取单个实体详情 | `name` | — |
| `set_property` | 设置实体属性 | `name`, `property`, `value` | — |
| `list_entities` | 列出所有实体摘要 | — | — |

### `kind` 可选值（spawn_entity）

| 值 | 说明 |
|----|------|
| `block` | 矩形方块（默认） |
| `circle` | 圆形 |
| `player` | 玩家角色 |
| `camera` | 相机节点 |

### `property` 可选值（set_property）

| 值 | 类型 | 说明 |
|----|------|------|
| `visible` | `bool` | 是否可见 |
| `rotation` | `number` | 旋转角度（弧度） |
| `scale_x` | `number` | X 轴缩放 |
| `scale_y` | `number` | Y 轴缩放 |
| 任意字符串 | `any` | 自定义属性 |

---

## 演示场景

服务器启动时会自动创建一个演示场景，包含 3 个实体：

| 名称 | 类型 | 初始位置 |
|------|------|----------|
| `MainBlock` | Block | (0, 0) |
| `RedBlock` | Block | (-200, 0) |
| `GreenCircle` | Circle | (200, 0) |

---

## MCP 协议说明

本服务器实现 [MCP 2024-11-05](https://spec.modelcontextprotocol.io/) 规范，支持：

- ✅ `initialize` / `notifications/initialized` 握手
- ✅ `tools/list` 工具发现
- ✅ `tools/call` 工具调用
- ✅ `ping` 心跳
- ✅ `resources/list` / `prompts/list`（空列表，预留扩展）

传输层：**JSON-RPC 2.0 over stdio**（每条消息一行 JSON + `\n`）

日志输出到 **stderr**，不污染 JSON-RPC 的 **stdout** 流。

---

## 与真实 Bevy 引擎集成

目前 `EngineBridge` 使用内存中的 `SceneState` 模拟引擎状态（无 Bevy 依赖，轻量可测试）。

若要接入真实 Bevy 引擎，可通过以下方式替换：

```rust
// 方案 1：跨线程 channel
let (tx, rx) = tokio::sync::mpsc::channel::<EngineCommand>(100);
// MCP Server 发送命令 → Bevy World 在 Update 系统中处理

// 方案 2：Bevy Resource + Mutex
// 在 Bevy World 中插入 Arc<Mutex<SceneState>> 作为 Resource
// MCP Server 和 Bevy System 共享同一个 Arc
```

---

## 开发

```sh
# 运行所有测试
cargo test -p ummerse-mcp

# 查看测试输出
cargo test -p ummerse-mcp -- --nocapture

# 检查代码（不编译二进制，更快）
cargo check -p ummerse-mcp
```
