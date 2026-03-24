# Ummerse Engine — 技术栈架构蓝图

> **AI 驱动型游戏引擎** · Rust Workspace · 12 Crates · 五层架构

---

## 总览图

```
╔══════════════════════════════════════════════════════════════════════════════════════════════╗
║                              🤖  Cline / AI Agent  (VSCode + MCP)                           ║
╚══════════════════════════════════════════════════════════════════════════════════════════════╝
                                           │  MCP JSON-RPC (stdio)
                                           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  🛠️  TOOL LAYER  —  ummerse-editor · ummerse-mcp · ummerse-plugin                           │
│                                                                                               │
│  ┌──────────────────────┐  ┌───────────────────────┐  ┌──────────────────────────────────┐  │
│  │   ummerse-editor     │  │    ummerse-mcp         │  │        ummerse-plugin            │  │
│  │  ─────────────────   │  │  ──────────────────    │  │  ──────────────────────────────  │  │
│  │  bevy (UI/Viewport)  │  │  tokio (stdio/tcp)     │  │  wasmtime (Wasm 沙箱)            │  │
│  │  taffy (Layout)      │  │  serde_json (JSON-RPC) │  │  serde_json (Plugin Manifest)    │  │
│  │  notify (热更新触发) │  │  uuid (Entity ID)      │  │  tokio (异步加载插件)            │  │
│  │  walkdir (项目树)    │  │  tracing (调用追踪)    │  │  dashmap (插件注册表)            │  │
│  └──────────────────────┘  └───────────────────────┘  └──────────────────────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────────────┬─────────────┘
                                     │                                           │
                                     ▼                                           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  ⚙️  FUNCTION LAYER  —  ummerse-renderer · ummerse-physics · ummerse-audio · ummerse-script  │
│                                                                                               │
│  ┌─────────────────────┐  ┌──────────────────────┐  ┌─────────────┐  ┌───────────────────┐  │
│  │  ummerse-renderer   │  │  ummerse-physics      │  │ummerse-audio│  │  ummerse-script   │  │
│  │  ────────────────   │  │  ─────────────────    │  │─────────────│  │  ──────────────   │  │
│  │  wgpu (GPU 管线)    │  │  [rapier2d/3d 预留]   │  │[kira 预留]  │  │  wasmtime (脚本)  │  │
│  │  naga (WGSL 编译)   │  │  glam (碰撞数学)      │  │glam (空间)  │  │  tokio (异步执行) │  │
│  │  image (纹理解码)   │  │  serde (物理快照)     │  │serde (配置) │  │  dashmap (符号表) │  │
│  │  bytemuck (GPU缓冲) │  │  tracing (日志)       │  │tracing(日志)│  │  serde_json (API) │  │
│  │  glam (MVP 矩阵)    │  │                       │  │             │  │  uuid (脚本实例)   │  │
│  └─────────────────────┘  └──────────────────────┘  └─────────────┘  └───────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────────────┬─────────────┘
                                     │                                           │
                                     ▼                                           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  📦  RESOURCE LAYER  —  ummerse-asset · ummerse-scene                                        │
│                                                                                               │
│  ┌──────────────────────────────────────────┐  ┌──────────────────────────────────────────┐  │
│  │           ummerse-asset                  │  │            ummerse-scene                  │  │
│  │  ─────────────────────────────────────   │  │  ─────────────────────────────────────   │  │
│  │  serde + serde_json + ron  (序列化 ↔ AI) │  │  serde + serde_json + ron  (场景持久化)  │  │
│  │  tokio          (异步加载 Texture/Audio)  │  │  bevy_ecs      (ECS 组件存储)            │  │
│  │  notify         (文件系统热重载监听)      │  │  indexmap      (有序节点字典)            │  │
│  │  image          (PNG/JPG → RGBA 像素)    │  │  smallvec      (子节点紧凑数组)          │  │
│  │  walkdir        (遍历资产目录)           │  │  uuid          (节点唯一 ID)             │  │
│  │  dashmap        (异步句柄缓存池)         │  │  glam          (本地变换)                │  │
│  │  bytemuck       (零拷贝 GPU 上传)        │  │                                          │  │
│  └──────────────────────────────────────────┘  └──────────────────────────────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────────────┬─────────────┘
                                     │                                           │
                                     ▼                                           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  🧠  CORE LAYER  —  ummerse-core · ummerse-math                                              │
│                                                                                               │
│  ┌──────────────────────────────────────────┐  ┌──────────────────────────────────────────┐  │
│  │              ummerse-core                │  │             ummerse-math                  │  │
│  │  ─────────────────────────────────────   │  │  ─────────────────────────────────────   │  │
│  │  bevy_ecs + bevy_app  (ECS 世界 & 调度)  │  │  glam     (Vec2/Vec3/Mat4/Quat)          │  │
│  │  bevy_reflect         (运行时类型反射)   │  │  bytemuck (数学类型 → GPU 字节)          │  │
│  │  serde + ron          (组件 ↔ AI 文本)   │  │  serde    (数学类型序列化)               │  │
│  │  dashmap              (资源并发注册表)   │  │                                          │  │
│  │  parking_lot          (低开销读写锁)     │  │  提供：Transform · AABB · Rect           │  │
│  │  bitflags             (标志位状态机)     │  │         Color · Plane                    │  │
│  │  smallvec             (紧凑事件队列)     │  │                                          │  │
│  │  indexmap + ahash     (确定性有序 Map)   │  │                                          │  │
│  │  uuid                 (Entity 唯一标识)  │  │                                          │  │
│  └──────────────────────────────────────────┘  └──────────────────────────────────────────┘  │
└────────────────────────────────────┬───────────────────────────────────────────┬─────────────┘
                                     │                                           │
                                     ▼                                           ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  🧱  PLATFORM LAYER  —  ummerse-runtime  (集成层：将所有 crate 串联并驱动主循环)             │
│                                                                                               │
│  ┌──────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                             ummerse-runtime                                          │    │
│  │  ─────────────────────────────────────────────────────────────────────────────────   │    │
│  │  bevy (bevy_winit)     ── 跨平台窗口创建 + 鼠标/键盘/窗口事件捕获                   │    │
│  │  tokio (rt-multi-thread) ── 后台异步运行时：MCP 网络请求 + 文件 I/O，不阻塞主循环   │    │
│  │  bevy (bevy_app)       ── 引擎主循环调度：Update / FixedUpdate / Render Schedule    │    │
│  │  serde_json / toml     ── 配置文件加载（项目设置、启动参数）                        │    │
│  │  tracing-subscriber    ── 初始化全局日志，格式化输出 AI 的每一步操作追踪            │    │
│  └──────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                               │
│  [WASM Target]  wasm-bindgen · web-sys · js-sys  ── 浏览器画布绑定 + WebGL2 上下文          │
└─────────────────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  📚  3RD PARTY SUPPORT  —  贯穿所有层的横切基建                                              │
│                                                                                               │
│  tracing / tracing-subscriber  ── 结构化日志 + 分布式追踪，追踪 AI 的每一个 API 调用        │
│  anyhow / thiserror            ── 统一错误类型：anyhow 在上层聚合，thiserror 在底层定义     │
│  serde / serde_json / ron      ── AI ↔ 引擎的通用语言，贯穿序列化/反序列化全栈             │
│  image                         ── PNG/JPEG → RGBA 像素阵列，供 wgpu 上传为 GPU 纹理         │
│  uuid                          ── Entity / Node / Asset 的全局唯一标识符                    │
│  once_cell                     ── 全局单例（渲染器实例、事件总线）的惰性初始化              │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 各层职责与 Crate 精确映射

### 🧱 Platform Layer — `ummerse-runtime`

| Crate | 版本 | 职责说明 |
|---|---|---|
| **`bevy` (`bevy_winit`)** | 0.15 | 跨平台窗口创建（Win/Mac/Linux/Web），捕获鼠标、键盘、窗口 resize 事件 |
| **`tokio`** (`rt-multi-thread`) | 1.x | 后台异步运行时。MCP 网络请求和文件 I/O 在此运行，**不阻塞** bevy 同步主循环 |
| **`bevy` (`bevy_app`)** | 0.15 | 引擎主循环：`Update` / `FixedUpdate` / `Render` Schedule 三段式调度 |
| `wasm-bindgen` / `web-sys` / `js-sys` | 0.2/0.3 | **[WASM Target]** 浏览器画布绑定，提供 WebGL2 渲染上下文 |
| `tracing-subscriber` | 0.3 | 运行时初始化全局结构化日志，格式化打印 AI 操作的每一步调用链 |
| `serde_json` / `toml` | 1/0.8 | 解析项目启动配置文件（`project.toml`），初始化引擎参数 |

> **⚠️ 与规划对比**：项目当前使用 **`bevy` 集成的 `winit`**（通过 `bevy_winit`）而非裸 `winit`；使用 **`bevy` 的 ECS 调度**替代 `rayon` 直接并行（bevy 内部已使用 rayon 进行系统并行）。`crossbeam` 的跨线程通道职责由 **`tokio::sync::mpsc`** 承担。

---

### 🧠 Core Layer — `ummerse-core` + `ummerse-math`

| Crate | 版本 | 职责说明 |
|---|---|---|
| **`bevy_ecs`** | 0.15 | ECS 绝对核心：Archetype 内存布局，组件紧凑排列在 CPU 缓存中，游戏逻辑彻底解耦 |
| **`bevy_app`** | 0.15 | Plugin 系统与 App 构建器，各功能层以 Plugin trait 注入调度 |
| **`bevy_reflect`** | 0.15 | 运行时类型反射：让 AI 可以通过字符串名称动态获取/修改任意组件字段 |
| **`glam`** (`serde` + `bytemuck`) | 0.29 | 专为游戏设计的 SIMD 数学库：`Vec2/Vec3/Vec4`、`Mat4`、`Quat`，直接 `bytemuck` 转 GPU 字节 |
| `serde` + `ron` | 1/0.8 | **AI 能"看懂"引擎的关键**：将 ECS 组件实时序列化为 RON/JSON，供 Cline 分析并反序列化回内存 |
| `dashmap` | 6 | 无锁并发 HashMap：资源注册表、组件类型表，支持多线程并发读写 |
| `parking_lot` | 0.12 | 高性能 `RwLock` / `Mutex`，替代标准库锁，减少线程调度开销 |
| `bitflags` | 2 | 组件状态标志位（如 `NodeFlags::VISIBLE \| DIRTY`），零开销状态机 |
| `smallvec` | 1 | 事件队列的栈上小容量优化（≤8 个事件不分配堆内存） |
| `indexmap` + `ahash` | 2/0.8 | 确定性有序哈希表 + 超快哈希算法，确保 AI 操作结果可重现 |
| `uuid` | 1 | Entity / Node / Asset 的 128-bit 全局唯一 ID (`v4` 随机生成，`serde` 序列化) |

> **⚠️ 与规划对比**：规划中的 `hecs` 已由 **`bevy_ecs`** 完全覆盖（更强大的调度系统）。`crossbeam` 的无锁 Channel 职责由 **`tokio::sync`** 承担（统一异步生态）。

---

### 📦 Resource Layer — `ummerse-asset` + `ummerse-scene`

| Crate | 所在 Crate | 职责说明 |
|---|---|---|
| **`serde` + `serde_json` + `ron`** | asset + scene | **AI 通信桥梁**：场景树、组件配置的双向序列化。AI 返回 JSON → `serde` 反序列化 → ECS 组件 |
| **`notify`** | asset (non-wasm) | 文件系统热重载监听器：AI 在外部修改配置文件 → `notify` 触发事件 → `AssetServer` 重新加载 |
| **`tokio`** | asset | 异步资产加载：纹理、音频、字体的非阻塞 I/O，通过 `tokio::sync::mpsc` 通知主循环 |
| **`image`** (`png` + `jpeg`) | asset | 将 PNG/JPEG 文件解码为 `RgbaImage`（像素数组），再由 `wgpu` 上传为 GPU 纹理 |
| **`dashmap`** | asset | 并发资产句柄缓存池：`AssetServer` 的内部存储，支持多系统并发读取 |
| `bytemuck` | asset | 零拷贝将 `RgbaImage` 像素数据直接转换为 `&[u8]` 供 `wgpu::Queue::write_texture` |
| `walkdir` | asset | 遍历项目资产目录，构建初始资产索引 |
| `bevy_ecs` | scene | 场景节点背后的 ECS 存储：`SceneTree` 中每个节点对应一个 ECS Entity |
| `indexmap` | scene | 有序节点子节点字典（保持场景树中节点的插入顺序，影响渲染顺序） |
| `smallvec` | scene | 子节点列表的栈上紧凑存储（大多数节点子节点数 < 8） |

---

### ⚙️ Function Layer — `ummerse-renderer` · `ummerse-physics` · `ummerse-audio` · `ummerse-script`

#### ummerse-renderer（图形渲染）

| Crate | 职责说明 |
|---|---|
| **`wgpu`** (v24, `wgsl`) | 跨平台 GPU 抽象层，后端支持 Vulkan / DX12 / Metal / WebGL2。读取 ECS 位置数据 + Resource 层纹理，提交绘制指令 |
| **`naga`** (`wgsl-in/out`) | WGSL 着色器编译与验证，在运行时动态编译/热重载 `.wgsl` 着色器文件 |
| `glam` | 计算 MVP 矩阵（Model-View-Projection），传入 GPU Uniform Buffer |
| `bytemuck` | 将顶点数据结构体（`#[derive(Pod)]`）零拷贝转为 `&[u8]` 写入 `wgpu::Buffer` |
| `image` | PNG/JPEG 纹理数据的最终消费方：像素数组 → `wgpu::Texture` |

#### ummerse-physics（物理引擎）

| Crate | 职责说明 |
|---|---|
| **[`rapier2d` 预留接口]** | 已设计好刚体/碰撞体/关节的抽象层（`world.rs`/`body.rs`/`collider.rs`/`joint.rs`），待接入 rapier2d/3d |
| `glam` | 碰撞检测数学：AABB 包围盒、射线投射、分离轴定理所需向量运算 |
| `serde` | 物理世界快照序列化：AI 可读取当前物理状态进行预测和干预 |

#### ummerse-audio（音频引擎）

| Crate | 职责说明 |
|---|---|
| **[`kira` 预留接口]** | 已设计 `AudioPlayer`/`AudioBus`/`SpatialAudio` 抽象层，待接入 `kira` |
| `glam` | 3D 空间音频的声源/听者位置计算（衰减曲线、方向向量） |
| `serde` | 音频配置序列化（音量、混响参数），允许 AI 通过 MCP 调整音效 |

#### ummerse-script（脚本运行时）

| Crate | 职责说明 |
|---|---|
| **`wasmtime`** (`cranelift` + `component-model`) | Wasm 脚本沙箱运行时：用户编写的游戏逻辑脚本在此安全执行，JIT 编译保证性能 |
| `tokio` | 脚本的异步执行上下文（脚本可 `await` 异步操作而不阻塞主循环） |
| `serde_json` | 脚本 ↔ 引擎 API 的数据交换格式（`HostFunction` 参数通过 JSON 传递） |
| `dashmap` | 脚本实例注册表，快速按 UUID 查找正在运行的脚本 |

---

### 🛠️ Tool Layer — `ummerse-editor` · `ummerse-mcp` · `ummerse-plugin`

#### ummerse-editor（可视化编辑器）

| Crate | 职责说明 |
|---|---|
| **`bevy`** | 编辑器视口渲染（游戏画面实时预览）+ bevy_ui 构建编辑器面板 |
| **`taffy`** | Flexbox/Grid 布局引擎：计算编辑器 UI 面板、侧边栏、属性检查器的布局 |
| **`notify`** | 编辑器热重载：监听工程文件变化，自动刷新资产浏览器和场景树 |
| `walkdir` | 项目文件树遍历，构建 `ProjectPanel` 中的文件导航视图 |
| `tokio` | 编辑器后台任务：AI 请求、资产导入任务的异步处理 |
| `serde_json` / `toml` | 编辑器配置、布局偏好、项目元数据的读写 |
| `dashmap` | 编辑器状态（已打开文件、选中实体）的线程安全缓存 |

> **⚠️ 与规划对比**：`gpui` 方案因与 bevy 生态集成成本高，当前使用 **`bevy_ui` + `taffy`** 替代。Monaco Editor 的代码编辑功能通过 `ummerse-script` 的脚本系统实现。

#### ummerse-mcp（AI 大门 · MCP 服务器）

| Crate | 职责说明 |
|---|---|
| **`tokio`** (`io-std` + `rt-multi-thread`) | **MCP 服务器核心**：通过 `stdin/stdout` 监听 Cline 发来的 JSON-RPC 请求，异步处理不阻塞 |
| **`serde_json`** | MCP 协议的序列化/反序列化：`tools/list`、`tools/call`、`resources/read` 等请求体解析 |
| `uuid` | 为 `spawn_entity()`、`create_node()` 等工具返回的新对象生成唯一 ID |
| `tracing` | 记录每一次 AI 调用的工具名、参数和返回结果，方便调试 AI 的操作链 |
| `anyhow` / `thiserror` | MCP 错误响应的统一封装，将引擎内部错误转为 JSON-RPC `error` 对象返回给 Cline |

**暴露给 Cline 的 MCP 工具（`tools.rs`）**：
```
get_scene_tree()       → 返回当前场景树 JSON（AI 的"眼睛"）
spawn_entity(config)   → 在世界中生成一个新 Entity
get_entity(id)         → 获取指定 Entity 的所有组件
update_component(...)  → 修改 Entity 的某个组件值
delete_entity(id)      → 删除 Entity
get_engine_stats()     → 获取 FPS、实体数、内存占用等统计信息
```

#### ummerse-plugin（插件沙箱）

| Crate | 职责说明 |
|---|---|
| **`wasmtime`** (`component-model`) | 插件安全沙箱：第三方 `.wasm` 扩展在此运行，无法访问主引擎内存空间 |
| `serde_json` | 解析插件的 `plugin.json` 清单文件（能力声明、版本要求） |
| `tokio` | 插件的异步加载与卸载，不阻塞编辑器/运行时主循环 |
| `dashmap` | 插件注册表：已加载插件的快速查找与管理 |
| `walkdir` | 扫描插件目录，发现新安装的 `.wasm` 插件 |

---

### 📚 3rd Party Support — 横切所有层的基建

| Crate | 版本 | 全局职责 |
|---|---|---|
| **`tracing`** + `tracing-subscriber` | 0.1/0.3 | **AI 行为追踪系统**。在控制台清晰展示 AI 的每一步 MCP 调用如何穿透各层到达底层 API |
| **`anyhow`** + `thiserror` | 1/2 | 统一错误处理：`thiserror` 在各 crate 定义类型化错误，`anyhow` 在 `main.rs` 聚合并输出 |
| **`serde`** + `serde_json` + `ron` | 1/1/0.8 | **AI ↔ 引擎通用语言**：JSON 用于 MCP 通信，RON 用于场景/配置文件（更人类可读） |
| **`image`** | 0.25 | 图像解码基础库：PNG/JPEG → RGBA 像素阵列，被 `ummerse-asset` 和 `ummerse-renderer` 共用 |
| **`uuid`** | 1 | 全局唯一 ID：从 Entity 到 Asset 到 Plugin 实例，统一使用 UUID v4 |
| `once_cell` | 1 | 全局单例的惰性初始化（渲染器上下文、事件总线），线程安全的 `static` 替代方案 |
| `glam` | 0.29 | 数学基础：`bytemuck` 特性让向量/矩阵直接转为 GPU 字节；`serde` 特性让数学类型可序列化给 AI |
| `tokio` | 1 | 统一异步运行时：MCP 服务、资产加载、脚本执行共享同一个 `rt-multi-thread` 线程池 |

---

## 数据流演示：Cline 添加一个带重力的方块

```
① Cline (VSCode)
   │  发送 MCP 请求: tools/call { "name": "spawn_entity", "arguments": {"type": "box", "pos": [0,5]} }
   │
   ▼
② ummerse-mcp  [TOOL LAYER]
   │  tokio::io 从 stdin 读取 JSON-RPC
   │  serde_json::from_str() 解析请求体
   │  tracing::info!("spawn_entity called, pos={:?}", pos)  ← 日志记录
   │
   ▼
③ ummerse-runtime  [PLATFORM LAYER]
   │  tokio::sync::mpsc::send(SpawnCommand { pos: Vec2(0.0, 5.0) })
   │  ↑ 跨越异步/同步边界：tokio 线程 → bevy 主循环
   │
   ▼
④ ummerse-core  [CORE LAYER]
   │  bevy_ecs: world.spawn((
   │      Position(Vec2::new(0.0, 5.0)),   // glam::Vec2
   │      RigidBody::Dynamic,
   │      Mesh2D::Square,
   │  ))
   │  → 返回 Entity { id: uuid::Uuid::new_v4() }
   │
   ▼
⑤ ummerse-physics  [FUNCTION LAYER]
   │  PhysicsWorld::create_rigidbody(entity, RigidBodyType::Dynamic)
   │  → rapier2d（预留）注册刚体，施加重力加速度
   │
⑤ ummerse-renderer  [FUNCTION LAYER]
   │  wgpu: 从 ECS Query 读取 Position + Mesh2D
   │  → 生成顶点缓冲区 (bytemuck)
   │  → 更新 GPU Uniform Buffer (glam::Mat4 MVP)
   │  → submit draw call → 方块出现在屏幕
   │
⑤ ummerse-asset（可选）  [RESOURCE LAYER]
   │  如果 Cline 指定了贴图路径：
   │  AssetServer::load("textures/box.png")
   │  tokio → 异步读文件 → image::load() → wgpu::Texture
   │  notify → 若 box.png 被 AI 修改 → 自动热重载
   │
   ▼
⑥ ummerse-mcp  [TOOL LAYER]
   │  serde_json::to_string(EntityCreatedResponse { id: "...", success: true })
   │  → stdout → Cline 收到确认
   ▼
Cline: "✅ 方块已生成，ID: f47ac10b-..."
```

---

## Crate 依赖关系图

```
                    ummerse-editor ──────────────────────────────────────┐
                         │                                               │
                    ummerse-runtime ──────────────────────────────────┐  │
                    ╱    │    ╲    ╲    ╲    ╲    ╲                  │  │
      ummerse-renderer  scene  asset  script  physics  audio  plugin  │  │
              │    ╲      │╱      │       │                           │  │
       ummerse-math  ╲  ummerse-core ◄────┘                          │  │
              │        ╲     │                                        │  │
             glam      bevy_ecs + bevy_app + bevy_reflect             │  │
                                                                      │  │
                    ummerse-mcp (独立二进制，通过 IPC 与 runtime 通信)│  │
                                                                      │  │
┌─────────────────────────────────────────────────────────────────────┘  │
│  贯穿所有层：serde · tokio · tracing · anyhow · uuid · glam            │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 技术选型决策说明

| 规划方案 | 实际实现 | 原因 |
|---|---|---|
| 裸 `winit` | `bevy_winit`（通过 `bevy`）| bevy 已集成 winit，避免重复管理窗口生命周期 |
| `hecs` 或 `bevy_ecs` | **`bevy_ecs`** | bevy_ecs 拥有更完善的调度系统（Stages/Schedules）和 reflect 支持 |
| `crossbeam` channels | **`tokio::sync::mpsc`** | 统一异步生态，tokio 的 channel 支持 `async/await`，与 MCP 服务器无缝集成 |
| `rayon` | **bevy 内置并行系统** | bevy 调度器内部使用 rayon 实现系统并行，上层无需手动管理 |
| `kira` 音频 | **预留接口**（`ummerse-audio`）| 接口已设计完毕，`kira` 作为下一步集成目标 |
| `rapier2d` 物理 | **预留接口**（`ummerse-physics`）| 抽象层已完成（body/collider/joint/world），待接入 rapier |
| `gpui` 编辑器 UI | **`bevy_ui` + `taffy`** | 避免两套 GPU 渲染栈，bevy_ui 已内置 taffy 布局 |
| `reqwest` HTTP | **`tokio` + 标准 I/O** | MCP 协议走 stdio 而非 HTTP，reqwest 暂不需要 |

---

*文档生成时间：2026-03-24 · Ummerse v0.1.0*
