# repotui 开发指南

**项目**: GitHub 仓库管理 TUI 工具  
**框架**: Rust + Ratatui + Tokio  
**架构**: Elm (Model-View-Update)

---

## 🗂️ 文档索引

### 需求文档

| 文档 | 说明 | 位置 |
|------|------|------|
| [ghclone-prd-v2.md](docs/ghclone-prd-v2.md) | **当前版本** - 基于审查反馈的完整 PRD | 项目根目录 |
| [ghclone-prd-v1.md](docs/ghclone-prd-v1.md) | 初始版本 - 已审查 | 项目根目录 |


### 开发文档

详细文档位于 [`docs/`](docs/) 目录：

| 文档 | 说明      |
|------|---------|
| [docs/README.md](./docs/README.md) | 开发文档总索引 |
| [docs/DEVELOPMENT_GUIDE.md](./docs/DEVELOPMENT_GUIDE.md) | 开发清单    |
| [docs/PHASE0_COMPLETE.md](./docs/PHASE0_COMPLETE.md) | Phase 0 完成报告 |
| [docs/PHASE0_STATUS.md](./docs/PHASE0_STATUS.md) | 详细实施状态 |
| [docs/FIX_PROGRESS.md](./docs/FIX_PROGRESS.md) | 修复进度记录 |
| [docs/BUGFIX_EMPTY_PATH.md](./docs/BUGFIX_EMPTY_PATH.md) | 空路径验证 Bug 修复方案 |


---

## 🎯 快速参考

### 构建命令

```bash
# 开发构建
cargo build

# Release 构建
cargo build --release

# 运行
cargo run

# 测试
cargo test

# 检查
cargo check

# 格式化
cargo fmt

# Lint
cargo clippy -- -D warnings

# 基准测试
cargo bench
```

### 项目结构

```
repotui/
├── src/
│   ├── main.rs          # 程序入口
│   ├── lib.rs           # 库入口 + 主循环
│   ├── app/             # Elm 架构核心
│   │   ├── model.rs     # 应用状态
│   │   ├── msg.rs       # 消息定义 (含 Cmd)
│   │   ├── update.rs    # 状态更新
│   │   └── state.rs     # AppState 枚举
│   ├── config/          # 配置管理
│   │   ├── types.rs     # Config 结构体
│   │   ├── load.rs      # 加载/保存
│   │   └── validators.rs# 安全验证
│   ├── repo/            # 仓库操作
│   │   ├── types.rs     # Repository 定义
│   │   ├── discover.rs  # 仓库发现
│   │   └── status.rs    # Git 状态检测
│   ├── action/          # 命令执行
│   │   ├── types.rs     # Action 枚举
│   │   ├── execute.rs   # 安全执行
│   │   └── validators.rs# 白名单验证
│   ├── ui/              # UI 渲染
│   │   ├── render.rs    # 主渲染逻辑
│   │   ├── theme.rs     # 主题配置
│   │   └── widgets/     # 自定义组件
│   ├── handler/         # 事件处理
│   │   └── keyboard.rs  # 键盘事件
│   ├── runtime/         # 异步运行时
│   │   └── executor.rs  # Cmd 执行器
│   ├── error.rs         # 统一错误类型
│   └── constants.rs     # 常量定义
├── tests/               # 测试
│   ├── integration/     # 集成测试
│   └── e2e/            # E2E 测试
├── benches/             # 基准测试
├── docs/                # 开发文档
└── config.toml.example  # 配置示例
```

---

## 📋 文件规范

**CLAUDE.md 定位**: 开发规范与指南

**允许内容**:
- 架构设计与规范
- 编码标准与最佳实践
- 项目结构与模块说明
- 安全设计原则
- 测试策略
- 依赖管理说明

**禁止内容**:
- 具体任务描述
- 已修复问题记录
- 临时修复脚本
- 待办事项列表
- 进度统计

**相关内容应移至**:
- 修复记录 → `docs/FIX_PROGRESS.md`
- 详细 Bug 分析 → `docs/BUGFIX_*.md`
- 任务计划 → `docs/DEVELOPMENT_GUIDE.md`

---

## 🔐 安全设计要点

### 1. 命令执行安全

```rust
// ✅ 安全：使用 current_dir 而非 shell cd
Command::new("claude")
    .current_dir(repo_path)
    .status()?;

// ❌ 危险：shell 注入风险
Command::new("sh")
    .arg("-c")
    .arg(format!("cd '{}' && claude", path))
```

### 2. 路径验证 (5+1 层验证链)

```rust
// 5 层验证链 + 1 层空路径检查（新增）
1. 空路径检查 - 拒绝空字符串路径
2. absolutize() - 规范化为绝对路径
3. exists() - 检查存在性
4. is_dir() - 检查是目录
5. starts_with(home) - 检查在主目录内
6. read_dir() - 检查读取权限
```

**重要更新**: 空路径检查必须在 absolutize() 之前进行，否则空字符串会被转换为当前工作目录，导致验证通过但实际使用失败。

### 3. 命令白名单

```rust
// src/constants.rs
pub const ALLOWED_COMMANDS: &[&str] = &["claude", "cursor", "cline"];
pub const ALLOWED_EDITORS: &[&str] = &["code", "webstorm", "vim", ...];
```

---

## 🏗️ 架构设计

### Elm 架构

```
┌─────────────┐     ┌─────────────┐
│   AppMsg    │────▶│   update    │
└─────────────┘     └──────┬──────┘
                           │
                           ▼
┌─────────────┐     ┌─────────────┐
│   render    │◀────│    App      │
└─────────────┘     └─────────────┘
```

### 核心类型

```rust
// Model
pub struct App {
    pub config: Option<Config>,
    pub repositories: Vec<Repository>,
    pub filtered_indices: Vec<usize>,
    pub search_query: String,
    pub list_state: ListState,
    pub state: AppState,
    pub msg_tx: mpsc::Sender<AppMsg>,
}

// Msg
pub enum AppMsg {
    SearchInput(char),
    NextRepo,
    PreviousRepo,
    ConfigLoaded(Result<Config, ConfigError>),
    RepositoriesLoaded(Result<Vec<Repository>, RepoError>),
    ExecuteAction(Action),
    Quit,
}

// Cmd (副作用)
pub enum Cmd {
    LoadConfig,
    LoadRepositories(PathBuf),
    CheckGitStatus(usize, PathBuf),
    ExecuteAction(Action, Repository),
}
```

### 状态优先级

```
ActionMenu (5) > Help (4) > ChoosingDir (3) > Searching (2) > Running (1)
```

---

## 📜 设计规范

### 单一按键原则

**核心理念**: 一个功能只能有一种触发方式，禁止多个按键触发同一功能。

**例外**: 方向键 `↑/↓` 作为标准导航键。

### 键盘交互规范

| 按键       | 用途      | 说明               |
|----------|---------|------------------|
| `↑/↓`    | 上下移动/滚动 | 通用导航键，支持**循环滚动** |
| `Ctrl+c` | 退出      | 关闭程序             |
| `Esc`    | 返回/关闭   | 关闭弹窗、退出搜索焦点      |
| `Enter`  | 确认/执行   | 打开菜单、执行操作        |

**循环滚动**: 从底部继续向下回到顶部，从顶部继续向上回到底部。

---

## 🧪 测试策略

### 测试金字塔

```
         E2E (5-10 场景)
        /  集成 (30-50 用例)
       /    单元 (200+ 用例，覆盖率≥80%)
```

### 运行测试

```bash
# 单元测试
cargo test

# 带覆盖率
cargo tarpaulin --out Html

# 基准测试
cargo bench

# 特定模块测试
cargo test -p repotui config
```

---

## 📦 依赖管理

### 核心依赖

| Crate | 版本 | 用途 |
|-------|------|------|
| ratatui | 0.29 | TUI 框架 |
| crossterm | 0.28 | 终端后端 |
| tokio | 1.x | 异步运行时 |
| serde | 1.0 | 序列化 |
| toml | 0.8 | 配置格式 |
| thiserror | 1.0 | 错误处理 |
| which | 6.0 | 命令查找 |

### 可选特性

```toml
[features]
git2 = ["dep:git2"]           # Git 状态检测 (Phase 3)
fuzzy = ["dep:nucleo-matcher"] # 模糊搜索 (Phase 3)
watcher = ["dep:notify"]       # 文件监听 (Phase 3)
```

## 🔗 相关资源

- [Ratatui 文档](https://ratatui.rs/)
- [Tokio 指南](https://tokio.rs/tokio/tutorial)
- [Thiserror 文档](https://docs.rs/thiserror)
- [Elm 架构](https://guide.elm-lang.org/architecture/)

---

**最后更新**: 2026-03-06
**维护者**: repotui Team
