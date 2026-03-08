# repotui 开发指南

**项目**: GitHub 仓库管理 TUI 工具  
**框架**: Rust + Ratatui + Tokio  
**架构**: Elm (Model-View-Update)

---

## 🗂️ 文档索引

### 文档分类索引

所有项目文档按类型分类存放，通过以下索引文件访问：

| 分类 | 索引文件 | 说明 |
|------|----------|------|
| 需求文档 (PRD) | [docs/prd/index.md](./docs/prd/index.md) | 产品需求文档索引 |
| 设计文档 (UI/UX) | [docs/design/index.md](./docs/design/index.md) | 界面设计规范文档索引 |
| 开发任务 | [docs/task/index.md](./docs/task/index.md) | 开发计划、Phase 报告、修复记录索引 |
| Bug 修复 | [docs/bugs/index.md](./docs/bugs/index.md) | Bug 分析与修复方案索引 |

### 快速参考

- [构建命令](#构建命令)
- [项目结构](#项目结构)
- [安全设计](#安全设计要点)
- [设计规范](#设计规范)

---

## 📁 文档管理规范

### 文档存放规则（强制执行）

| 文档类型 | 存放文件夹 | 索引文件 |
|---------|-----------|---------|
| PRD 需求文档 | `docs/prd/` | `docs/prd/index.md` |
| UI/UX 设计文档 | `docs/design/` | `docs/design/index.md` |
| 开发任务文档 | `docs/task/` | `docs/task/index.md` |
| Bug 修复文档 | `docs/bugs/` | `docs/bugs/index.md` |

**禁止**将上述类型文档直接存放在 `docs/` 根目录。

### 文档创建流程

1. **确定文档类型**：根据内容判断属于哪一类
2. **创建在对应文件夹**：在指定文件夹内创建 `.md` 文件
3. **更新索引文件**：在对应的 `index.md` 中添加文档条目
4. **命名规范**：
   - PRD: `ghclone-prd-v{n}.md`
   - 设计: `component-name.md` (小写，`-` 连接)
   - 任务: `phase{n}-complete.md` 或 `feature-name.md`
   - Bug: `brief-description.md` (小写，`-` 连接)

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

### 最小终端尺寸

**宽度**: 80 列  
**高度**: 25 行（包含标题栏 1 行）

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
│   │   ├── types.rs     # Repository 定义 (含 is_git_repo 字段)
│   │   ├── discover.rs  # 仓库发现 (返回所有目录，标记 git 状态)
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
│   ├── prd/            # PRD 需求文档
│   ├── design/         # UI/UX 设计文档
│   ├── task/           # 开发任务文档
│   └── bugs/           # Bug 修复文档
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

**文档迁移规则**:
- 修复记录 → `docs/task/`
- 详细 Bug 分析 → `docs/bugs/`
- 任务计划 → `docs/task/`
- PRD 文档 → `docs/prd/`
- 设计文档 → `docs/design/`

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

### 核心原则

**单一按键原则**: 一个功能只能有一种触发方式，禁止多个按键触发同一功能。

**设计文档**: [docs/design/ui-guidelines.md](./docs/design/ui-guidelines.md)

### 常用快捷键

| 按键 | 功能 |
|------|------|
| `↑/↓` | 导航 |
| `Enter` | 打开/确认 |
| `Esc` | 取消/返回 |
| `Ctrl+f` | 收藏夹视图 |
| `Ctrl+r` | 最近视图 |
| `v` | 多选模式 |
| `?` | 帮助 |

**完整快捷键**: [docs/design/keyboard-shortcuts.md](./docs/design/keyboard-shortcuts.md)

### 相关设计文档

- [标题栏设计](./docs/design/title-bar.md)
- [主题系统](./docs/design/theme-system.md)
- [UI 设计规范](./docs/design/ui-guidelines.md)

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

### 主题集成

**标题颜色**: 在 `ColorPalette` 中添加 `title_fg` 和 `title_bg` 字段

```rust
pub struct ColorPalette {
    // ... 现有字段 ...
    pub title_fg: ColorRgb,   // 标题前景色
    pub title_bg: ColorRgb,   // 标题背景色
}
```

**样式方法**:

```rust
impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.colors.title_fg.into())
            .bg(self.colors.title_bg.into())
            .add_modifier(Modifier::BOLD)
    }
}
```

## 🔗 相关资源

- [Ratatui 文档](https://ratatui.rs/)
- [Tokio 指南](https://tokio.rs/tokio/tutorial)
- [Thiserror 文档](https://docs.rs/thiserror)
- [Elm 架构](https://guide.elm-lang.org/architecture/)

---

**最后更新**: 2026-03-08  
**维护者**: repotui Team
