# 📋 PRD: GitHub 仓库管理 TUI (ghclone-tui) - v2

**文档版本**: v2  
**更新日期**: 2026-03-05  
**更新说明**: 基于 codeteam 多角色审查修复安全性、架构和测试问题

---

## 1. 产品概述

### 1.1 产品定位
基于 Ratatui 开发的终端用户界面工具，用于快速浏览、搜索和管理本地 GitHub 仓库，并提供一键进入仓库或打开编辑器的功能。

### 1.2 目标用户
需要在多个 GitHub 仓库之间快速切换的开发者。

### 1.3 核心价值
| 价值点 | 说明 |
|--------|------|
| 🚀 秒级定位 | 快速找到目标仓库 |
| ⌨️ 纯键盘操作 | 无需鼠标，符合终端习惯 |
| 🔍 实时搜索 | 按键即时过滤结果 |
| 🎯 一键启动 | 快速打开仓库或开发环境 |

---

## 2. 功能需求

### 2.1 启动流程

#### F1: 主目录选择 (首次启动)
- **触发条件**: 配置文件不存在
- **UI 组件**: 目录选择对话框
- **交互**:
  - 显示当前路径
  - 支持 `j/k` 或 `↑/↓` 浏览目录
  - 支持 `Enter` 确认选择
  - 支持 `q` 或 `Ctrl+C` 退出
- **安全验证**:
  - 路径必须在用户主目录内
  - 路径必须是目录而非文件
  - 路径必须有读取权限
  - 拒绝符号链接 (可选配置)
- **输出**: 将主目录路径保存到配置文件

#### F2: 主界面加载
- **触发条件**: 配置文件存在且有效
- **UI 组件**: 仓库列表 + 搜索框
- **加载逻辑**:
  ```rust
  async fn load_repositories(main_dir: &Path) -> Result<Vec<Repository>, RepoError> {
      // 异步遍历主目录所有一级子目录
      // 过滤：仅保留 git 仓库 (存在 .git 目录或文件)
      // 验证：路径规范化 + 权限检查
      // 返回 Repository { name, path, last_modified, is_dirty }
  }
  ```
- **错误处理**:
  - 配置文件损坏 → 备份原文件并创建新配置
  - 主目录不存在 → 返回 F1 目录选择
  - 主目录无权限 → 显示错误提示并退出

### 2.2 仓库列表与搜索

#### F3: 实时搜索过滤
- **UI 布局**:
  ```
  ╭─ ghclone ────────────────────────────────────────────────────╮
  │ 🔍 Search: [react________________]                   [15/342]│
  │                                                               │
  │ ╭─ Repositories ───────────────────────────────────────────╮ │
  │ │ ▌ github_facebook_react           main    ● dirty       │ │
  │ │   web_react_native_docs           main    ✓ clean       │ │
  │ │   personal_react_playground       feat    ✓ clean       │ │
  │ ╰──────────────────────────────────────────────────────────╯ │
  │                                                               │
│ [j/k] Cycle  [g/G] Jump  [/] Search  [Enter] Open  [r] Refresh │
  ╰───────────────────────────────────────────────────────────────╯
  ```
- **交互**:
  - `/` 键聚焦搜索框
  - 任意字母键：追加到搜索框 (仅在搜索聚焦状态)
  - `Backspace`: 删除字符
  - `Esc`: 清空搜索并失焦
  - 搜索结果实时更新 (防抖 100ms)
- **搜索算法**:
  ```rust
  fn filter_repos(repos: &[Repository], query: &str) -> Vec<usize> {
      if query.is_empty() {
          return (0..repos.len()).collect();
      }
      let query_lower = query.to_lowercase();
      repos.iter()
          .enumerate()
          .filter(|(_, repo)| repo.name.to_lowercase().contains(&query_lower))
          .map(|(i, _)| i)
          .collect()
  }
  ```

#### F4: 列表导航
- **按键映射**:
  | 按键 | 主界面 | 目录选择 | 操作菜单 | 帮助面板 |
  |------|--------|----------|----------|----------|
  | `q` | 退出确认 | 取消 | 取消 | 关闭 |
  | `Esc` | 清空搜索 | 取消 | 取消 | 关闭 |
  | `Enter` | 打开菜单 | 确认选择 | 执行操作 | - |
  | `/` | 聚焦搜索 | - | - | - |
  | `r` | 刷新列表 | - | - | - |
  | `?` | 打开帮助 | - | - | - |
  | `Ctrl+C` | 退出确认 | 退出 | 退出 | 退出 |
  | `↑/↓` | 循环导航 | 循环导航 | 导航 | - |
  | `1/2/3/4` | - | - | 执行操作 | - |

**状态优先级**: `ActionMenu > Help > ChoosingDir > Searching > Running`  
(高优先级状态拦截所有按键)

### 2.5 UI 设计原则

#### 2.5.1 单一按键原则

**核心理念**: 一个功能只能有一个单按键触发方式，禁止多个按键触发同一功能。

**设计理由**:
1. **减少认知负担**: 用户只需记忆一个键位
2. **避免误操作**: 防止意外触发非预期功能
3. **一致性体验**: 符合 TUI 工具的标准设计模式

**例外情况**:
- `g/Home` 和 `G/End`: 传统 vim 风格与现代键盘的兼容
- 方向键 `↑/↓`: 标准导航键，符合用户习惯

**实施清单**:
| 功能 | 保留按键 | 移除按键 |
|------|----------|----------|
| 向下导航 | `↓` | `j` |
| 向上导航 | `↑` | `k` |
| 打开菜单 | `Enter` | `o` |
| CdAndCloud | `1` | `c` |
| OpenWebStorm | `2` | `w` |
| OpenVsCode | `3` | `v` |
| OpenFileManager | `4` | `f` |

---

## 3. 技术架构

### 3.1 技术栈
```yaml
语言：Rust 1.75+ (latest stable)
TUI 框架：ratatui 0.29+
扩展组件：ratatui-widgets 0.2+ (Input, Select 等)
终端后端：crossterm 0.28+
配置管理：serde 1.0 + toml 0.8
路径处理：path-absolutize 3.1 + dirs 5.0
命令查找：which 6.0
安全转义：shell-escape 0.1
异步运行时：tokio 1.0 (必需，用于后台任务)
Git 操作：git2 0.19 (可选特性)
Fuzzy Search: nucleo-matcher 0.3 (可选)
日志：tracing 0.1 + tracing-subscriber 0.3
时间：chrono 0.4
错误处理：thiserror 1.0
```

### 3.2 项目结构
```
ghclone-tui/
├── Cargo.toml
├── src/
│   ├── main.rs              # 程序入口 + 终端初始化
│   ├── lib.rs               # 模块导出
│   ├── app/
│   │   ├── mod.rs           # App 状态定义
│   │   ├── model.rs         # Model 相关类型
│   │   ├── msg.rs           # Msg 枚举定义
│   │   └── update.rs        # Update 逻辑
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── render.rs        # 主界面渲染
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── repo_list.rs
│   │   │   ├── search_box.rs
│   │   │   └── action_menu.rs
│   │   └── theme.rs         # 主题配置
│   ├── handler/
│   │   ├── mod.rs
│   │   ├── normal.rs        # 正常状态按键处理
│   │   ├── searching.rs     # 搜索状态按键处理
│   │   └── action_menu.rs   # 操作菜单按键处理
│   ├── config/
│   │   ├── mod.rs
│   │   ├── load.rs          # 配置加载
│   │   ├── save.rs          # 配置保存
│   │   └── validators.rs    # 配置验证
│   ├── repo/
│   │   ├── mod.rs
│   │   ├── discover.rs      # 仓库发现
│   │   └── status.rs        # Git 状态检测
│   ├── action/
│   │   ├── mod.rs
│   │   ├── execute.rs       # 命令执行
│   │   └── validators.rs    # 输入验证
│   └── error.rs             # 统一错误类型
├── tests/
│   ├── integration/         # 集成测试
│   └── fixtures/            # 测试数据
├── config/
│   └── config.toml.example
└── README.md
```

### 3.3 配置格式 (`~/.config/ghclone-tui/config.toml`)
```toml
# 配置版本 (向后兼容)
version = "1.0"

# 主目录路径 (必需)
main_directory = "/home/username/projects"

# 编辑器配置 (可选)
[editors]
webstorm = "/Applications/WebStorm.app/Contents/MacOS/webstorm"
vscode = "code"

# 默认命令 (可选，白名单验证)
default_command = "claude"

# UI 配置 (可选)
[ui]
theme = "dark"  # dark | light
show_git_status = true
show_branch = true
```

**配置验证**:
```rust
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // 1. 路径必须是绝对路径
    let abs_path = config.main_directory.absolutize()?;
    
    // 2. 路径必须存在
    if !abs_path.exists() {
        return Err(ConfigError::DirectoryNotFound);
    }
    
    // 3. 路径必须是目录
    if !abs_path.is_dir() {
        return Err(ConfigError::NotADirectory);
    }
    
    // 4. 路径必须在用户主目录内
    let home = std::env::var("HOME")?;
    if !abs_path.starts_with(&home) {
        return Err(ConfigError::DirectoryOutsideHome);
    }
    
    // 5. 验证编辑器命令
    if let Some(editor) = &config.editors.webstorm {
        validate_editor_command(editor)?;
    }
    
    Ok(())
}
```

### 3.4 数据流 (Elm 架构)
```rust
// Model
struct App {
    main_dir: Option<PathBuf>,
    repositories: Vec<Repository>,
    filtered_indices: Vec<usize>,
    search_query: String,
    list_state: ListState,      // Ratatui ListState
    state: AppState,
    loading: bool,
    error_message: Option<String>,
    scroll_offset: usize,
}

enum AppState {
    Running,
    ChoosingDir { path: PathBuf, entries: Vec<DirEntry> },
    Searching,                  // 搜索框聚焦状态
    ShowingActions { repo: Repository },
    ShowingHelp,
    Loading { message: String },
    Error { message: String },
    Quit,
}

// Msg
enum AppMsg {
    // 用户输入
    SearchInput(char),
    SearchBackspace,
    SearchClear,
    NextRepo,
    PreviousRepo,
    JumpToTop,
    JumpToBottom,
    
    // 异步事件
    ConfigLoaded(Result<Config, ConfigError>),
    RepositoriesLoaded(Result<Vec<Repository>, RepoError>),
    GitStatusChecked(usize, GitStatus),
    
    // 状态切换
    OpenActions,
    CloseActions,
    ExecuteAction(Action),
    ActionExecuted(Result<(), ActionError>),
    
    // 全局
    Tick,
    Refresh,
    ShowHelp,
    CloseHelp,
    Quit,
}

// Cmd - 副作用类型
enum Cmd {
    LoadConfig,
    LoadRepositories(PathBuf),
    CheckGitStatus(usize, PathBuf),
    ExecuteSystemCommand(String, Vec<String>),
}

// Update
fn update(msg: AppMsg, app: &mut App) -> Option<Cmd> {
    match msg {
        AppMsg::SearchInput(c) => {
            app.search_query.push(c);
            app.filtered_indices = filter_repos(&app.repositories, &app.search_query);
            app.list_state.select(Some(0));
            None
        }
        AppMsg::RepositoriesLoaded(result) => {
            match result {
                Ok(repos) => {
                    app.repositories = repos;
                    app.filtered_indices = (0..app.repositories.len()).collect();
                }
                Err(e) => app.error_message = Some(e.to_string()),
            }
            None
        }
        // ...
    }
}

// View
fn view(app: &App, frame: &mut Frame) {
    match app.state {
        AppState::ChoosingDir => render_dir_chooser(frame, app),
        AppState::Running => render_main_ui(frame, app),
        AppState::ShowingActions => render_action_menu(frame, app),
        AppState::ShowingHelp => render_help(frame, app),
        AppState::Loading => render_loading(frame, app),
        AppState::Error => render_error(frame, app),
        AppState::Quit => {}
    }
}
```

---

## 4. UI 设计

### 4.1 主界面布局
```
╭─ ghclone ────────────────────────────────────────────────────╮
│ 🔍 Search: [react________________]                   [15/342]│
│                                                               │
│ ╭─ Repositories ───────────────────────────────────────────╮ │
│ │ ▌ github_facebook_react           main    ● dirty       │ │
│ │   web_react_native_docs           main    ✓ clean       │ │
│ │   personal_react_playground       feat    ✓ clean       │ │
│ │                                                           │ │
│ │                                                           │ │
│ ╰───────────────────────────────────────────────────────────╯ │
│                                                               │
│ ↑↓ navigate   g/G jump   / search   ENTER open   r refresh   ? help   q quit │
╰───────────────────────────────────────────────────────────────╯
```

**视觉层次**:
- 选中项：块状高亮 (`▌`) + 背景色
- 搜索聚焦：边框颜色变化 (Cyan)
- Dirty 状态：红色圆点 (`●`)
- Clean 状态：绿色对勾 (`✓`)

### 4.2 目录选择界面
```
╭─ Select Main Directory ──────────────────────────────────────╮
│                                                               │
│   ../                                                        │
│   Desktop/                                                   │
│   Documents/                                                 │
│ ▌ Developer/                                                 │
│   Downloads/                                                 │
│   Music/                                                     │
│                                                               │
│ Current: /home/username/projects                             │
│ Found: 42 Git repositories                                    │
│                                                               │
│ ↑↓ navigate   SPACE select   ENTER open   ← back   q cancel       │
╰───────────────────────────────────────────────────────────────╯
```

**安全验证**:
- 仅显示用户主目录内的目录
- 实时显示找到的仓库数量
- 权限检查 (不可读的目录灰色显示)

### 4.3 操作菜单
```
╭─ Actions: github_facebook_react ─────────────────╮
│ [1] Open in Claude Code                          │
│ [2] Open in WebStorm                             │
│ [3] Open in VS Code                              │
│ [4] Open in Finder/Explorer                      │
│ [q] Cancel                                       │
╰──────────────────────────────────────────────────╯
```

### 4.4 帮助面板
```
╭─ Keyboard Shortcuts ─────────────────────────────────────────╮
│ Navigation                                                    │
│   ↓       Move down (cyclic: last → first)                   │
│   ↑       Move up (cyclic: first → last)                     │
│   g       Go to top                                          │
│   G       Go to bottom                                       │
│   Ctrl+d  Scroll down half-page                              │
│   Ctrl+u  Scroll up half-page                                │
│                                                               │
│ Search                                                        │
│   /       Focus search                                       │
│   Esc     Clear search / Close panel                         │
│   [char]  Add to search query (when focused)                 │
│                                                               │
│ Actions                                                       │
│   Enter   Open action menu                                   │
│   1       Open in Claude Code                                │
│   2       Open in WebStorm                                   │
│   3       Open in VS Code                                    │
│   4       Open in Finder/Explorer                            │
│                                                               │
│ Global                                                        │
│   r       Refresh list                                       │
│   ?       Show this help                                     │
│   q       Quit                                               │
╰───────────────────────────────────────────────────────────────╯
```

### 4.5 响应式设计

**最小终端尺寸**: 80x24

**布局适配规则**:
| 终端宽度 | 策略 |
|----------|------|
| < 60 | 隐藏元数据 (分支/dirty 状态) |
| 60-100 | 显示分支，隐藏 dirty |
| > 100 | 显示完整信息 (包括右侧详情面板) |

**长文本处理**:
- 仓库名 > 可用宽度 - 10: 中间截断 (`github_face...react`)
- 路径显示：仅显示最后两级 (`facebook/react`)

---

## 5. 安全实现

### 5.1 命令执行安全
```rust
// action/execute.rs
use std::process::Command;
use which::which;

// 命令白名单
const ALLOWED_COMMANDS: &[&str] = &["claude", "cursor", "cline"];
const ALLOWED_EDITORS: &[&str] = &["code", "webstorm", "idea", "pycharm", "vim", "nvim"];

fn validate_command(cmd: &str) -> Result<(), ActionError> {
    // 1. 检查白名单
    if ALLOWED_COMMANDS.contains(&cmd) || ALLOWED_EDITORS.contains(&cmd) {
        return Ok(());
    }
    
    // 2. 检查 PATH
    if which(cmd).is_ok() {
        return Ok(());
    }
    
    Err(ActionError::CommandNotFound(cmd.into()))
}

fn execute_cd_and_cloud(repo_path: &Path) -> Result<(), ActionError> {
    // 安全：不使用 shell 拼接
    // 安全：直接设置工作目录
    Command::new("claude")
        .current_dir(repo_path)
        .status()
        .map_err(|_| ActionError::CommandNotFound("claude".into()))?;
    
    Ok(())
}

fn execute_editor(editor: &str, path: &Path) -> Result<(), ActionError> {
    validate_command(editor)?;
    
    Command::new(editor)
        .arg(path)  // arg() 自动处理特殊字符
        .status()?;
    
    Ok(())
}
```

### 5.2 路径验证
```rust
// config/validators.rs
use std::path::{Path, PathBuf};

fn validate_directory(path: &Path) -> Result<PathBuf, ConfigError> {
    // 1. 规范化为绝对路径
    let abs_path = path.absolutize()?;
    
    // 2. 检查存在性
    if !abs_path.exists() {
        return Err(ConfigError::DirectoryNotFound);
    }
    
    // 3. 检查是目录
    if !abs_path.is_dir() {
        return Err(ConfigError::NotADirectory);
    }
    
    // 4. 检查在用户主目录内
    let home = std::env::var("HOME")
        .map_err(|_| ConfigError::HomeNotFound)?;
    if !abs_path.starts_with(&home) {
        return Err(ConfigError::DirectoryOutsideHome);
    }
    
    // 5. 检查读取权限
    if abs_path.read_dir().is_err() {
        return Err(ConfigError::NoReadPermission);
    }
    
    Ok(abs_path.to_path_buf())
}
```

### 5.3 Git 仓库检测
```rust
// repo/discover.rs
use std::path::Path;

fn is_git_repo(path: &Path) -> bool {
    let git_path = path.join(".git");
    git_path.exists() && (git_path.is_dir() || git_path.is_file())
}

// 方案 A: 使用 git 命令 (MVP)
fn get_git_status(path: &Path) -> bool {
    std::process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=no")
        .current_dir(path)
        .output()
        .map(|out| !out.stdout.is_empty())
        .unwrap_or(false)
}

// 方案 B: 使用 git2 crate (Phase 3 性能优化)
#[cfg(feature = "git2")]
fn get_git_status_git2(path: &Path) -> bool {
    git2::Repository::open(path)
        .and_then(|repo| {
            let mut opts = git2::StatusOptions::new();
            opts.include_ignored(false);
            repo.statuses(Some(&mut opts))
        })
        .map(|statuses| !statuses.is_empty())
        .unwrap_or(false)
}
```

---

## 6. 错误处理

### 6.1 统一错误类型
```rust
// error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("配置加载失败：{0}")]
    Config(#[from] ConfigError),
    
    #[error("仓库扫描失败：{0}")]
    RepoScan(#[from] RepoError),
    
    #[error("命令执行失败：{0}")]
    ActionExec(#[from] ActionError),
    
    #[error("终端错误：{0}")]
    Terminal(#[from] crossterm::ErrorKind),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件不存在：{0}")]
    NotFound(PathBuf),
    
    #[error("配置解析失败：{0}")]
    ParseError(#[from] toml::de::Error),
    
    #[error("主目录不存在：{0}")]
    DirectoryNotFound(PathBuf),
    
    #[error("路径不是目录：{0}")]
    NotADirectory(PathBuf),
    
    #[error("目录无读取权限：{0}")]
    NoReadPermission(PathBuf),
    
    #[error("目录不在主目录内：{0}")]
    DirectoryOutsideHome(PathBuf),
}

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("命令不存在：{0}")]
    CommandNotFound(String),
    
    #[error("命令执行失败：{0}")]
    ExecutionFailed(std::io::Error),
    
    #[error("路径验证失败：{0}")]
    PathValidationFailed(String),
}

// 用户友好错误提示
impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            AppError::Config(ConfigError::DirectoryNotFound(path)) => {
                format!("主目录不存在：{}\n请重新选择有效目录", path.display())
            }
            AppError::ActionExec(ActionError::CommandNotFound(cmd)) => {
                format!("命令 '{}' 未找到\n请确认已安装并添加到 PATH", cmd)
            }
            // ...
        }
    }
}
```

### 6.2 错误 UI 展示
```
╭─ ⚠️ 错误 ────────────────────────────────────────────────────╮
│                                                               │
│  Unable to open WebStorm                                     │
│                                                               │
│  The 'webstorm' command was not found in PATH.               │
│                                                               │
│  建议：                                                      │
│  1. 确认 WebStorm 已安装                                     │
│  2. 将 WebStorm 添加到 PATH                                  │
│  3. 在配置文件中设置绝对路径                                 │
│                                                               │
│            [OK]               [Configure]                    │
│                                                               │
╰───────────────────────────────────────────────────────────────╯
```

---

## 7. 测试策略

### 7.1 测试金字塔
```
                    ╱╲╱╲
                   ╱E2E╲       5-10 个核心场景
                  ╱─────╲      手动 + 自动化混合
                 ╱─────────╲
                ╱  集成测试  ╲     30-50 个测试用例
               ╱─────────────╲    组件交互验证
              ╱─────────────────╲
             ╱    单元测试        ╲   200+ 测试用例
            ╱─────────────────────╲  覆盖率 ≥80%
```

### 7.2 单元测试
| 模块 | 测试范围 | 目标覆盖率 |
|------|----------|------------|
| config/ | TOML 解析、默认值、路径验证 | 90%+ |
| repo/ | is_git_repo, filter_repos | 90%+ |
| handler/ | 按键事件映射、状态转换 | 85%+ |
| action/ | 命令验证、安全执行 | 90%+ |
| app/ | 状态机转换逻辑 | 80%+ |

### 7.3 集成测试
| 测试场景 | 验证点 |
|----------|--------|
| 首次启动流程 | 目录选择 → 配置保存 → 主界面加载 |
| 搜索过滤 | 输入 → 过滤结果 → 列表更新 |
| 操作执行 | 选中仓库 → 选择操作 → 命令执行 |
| 错误恢复 | 配置损坏 → 重建配置 |

### 7.4 E2E 测试
| 场景 | 方法 |
|------|------|
| 完整用户流程 | 自动化终端脚本模拟按键序列 |
| 性能压力测试 | 1000+ mock 仓库场景验证 |

### 7.5 异常场景处理矩阵
| 场景 | 预期行为 | UI 提示 |
|------|----------|---------|
| 主目录不存在 | 回到目录选择界面 | "主目录不存在，请重新选择" |
| 配置文件损坏 | 备份并重建配置 | "配置文件损坏，已创建备份" |
| 无权限访问 | 显示错误并退出 | "无权限访问：<path>" |
| 0 个仓库 | 显示空状态提示 | "未发现 Git 仓库" |
| 搜索无结果 | 显示"无匹配结果" | "0 repositories match 'xxx'" |
| 命令不存在 | 捕获错误并提示 | "命令未找到：webstorm" |
| 终端<80x24 | 最小尺寸警告 | "终端尺寸过小，请调整" |

---

## 8. 性能验收标准

### 8.1 量化指标
| 指标 | 数据集 | 目标 | 测量方法 |
|------|--------|------|----------|
| 冷启动时间 | 1000 仓库 | < 500ms | `time` 命令 |
| 热启动时间 | 1000 仓库 | < 200ms | `time` 命令 |
| 搜索响应 | 1000 仓库 | < 50ms (p95) | 基准测试 |
| 列表滚动 | 1000 仓库 | > 30 FPS | 帧率测量 |
| 内存占用 | 1000 仓库 | < 50MB (RSS) | `top`/`htop` |
| Git 状态检测 | 1000 仓库 | < 5s (后台) | 异步任务计时 |

### 8.2 性能优化策略
1. **虚拟列表**: 只渲染可见区域的仓库项
2. **异步扫描**: 使用 tokio 后台任务加载仓库
3. **Git 状态缓存**: TTL 5 分钟缓存
4. **搜索防抖**: 100ms 防抖避免频繁过滤
5. **并行遍历**: 使用 rayon 并行扫描目录

---

## 9. 跨平台测试矩阵

### 9.1 操作系统
| 平台 | 版本 | 优先级 | 测试项 |
|------|------|--------|--------|
| macOS | 13+ (Intel/Apple Silicon) | P0 | 全部功能 |
| Ubuntu | 22.04 LTS | P1 | 核心功能 |
| Windows | 10/11 (PowerShell/CMD) | P1 | 核心功能 |

### 9.2 终端兼容性
| 终端 | 平台 | 测试项 |
|------|------|--------|
| iTerm2 | macOS | 渲染、颜色、按键 |
| Terminal.app | macOS | 渲染、颜色、按键 |
| Kitty | Linux/macOS | 渲染、颜色、按键 |
| Alacritty | 跨平台 | 渲染、颜色、按键 |
| Windows Terminal | Windows | 渲染、颜色、按键 |

### 9.3 平台差异处理
| 功能 | macOS | Linux | Windows |
|------|-------|-------|---------|
| 文件管理器 | `open` | `xdg-open` | `explorer` |
| 配置路径 | `~/.config/` | `~/.config/` | `%APPDATA%/` |
| 路径分隔符 | `/` | `/` | `\` (内部统一 `/`) |

---

## 10. 开发计划

### Phase 0: 安全基础 (2-3 天)
- [ ] 定义统一错误类型 (`error.rs`)
- [ ] 实现路径验证模块
- [ ] 实现命令白名单验证
- [ ] 安全命令执行封装
- [ ] 完善 Elm 架构 (Model, Msg, Update, View)

### Phase 1: MVP (1-2 周)
- [ ] 项目脚手架搭建
- [ ] 配置文件读写 + 验证
- [ ] 主目录选择界面
- [ ] 仓库列表渲染
- [ ] 基础搜索功能
- [ ] 键盘导航

### Phase 2: 核心功能 (1 周)
- [ ] 操作菜单
- [ ] cd + cloud 命令执行 (安全实现)
- [ ] 编辑器打开功能
- [ ] 帮助面板
- [ ] 错误处理 UI

### Phase 3: 增强体验 (1 周)
- [ ] Git 状态检测 (异步 + 缓存)
- [ ] 主题支持 (dark/light)
- [ ] 性能优化 (虚拟列表 + 防抖)
- [ ] 响应式布局

### Phase 4: 可选增强
- [ ] 最近打开记录
- [ ] 收藏夹功能
- [ ] Fuzzy Search
- [ ] 批量操作

---

## 11. 验收标准

### 功能验收
- ✅ 首次启动能选择并保存主目录 (配置持久化验证)
- ✅ 正确列出主目录下所有 git 仓库 (与 `find . -name .git` 比对)
- ✅ 搜索框实时过滤 (<50ms 响应，1000 仓库数据集)
- ✅ 键盘导航流畅 (输入延迟 <100ms p95)
- ✅ 能正确执行 cd+cloud 命令 (mock 测试 + E2E 验证)
- ✅ 能正确打开 WebStorm 和 VS Code (命令注入安全检查)
- ✅ 所有异常场景有正确处理 (见 7.5 矩阵)

### 性能验收
- ✅ 启动时间 <500ms (冷启动，1000 仓库)
- ✅ 支持 1000+ 仓库无卡顿 (内存 <50MB, CPU <5%)
- ✅ 搜索响应 <50ms (P95 延迟，100 次连续搜索)
- ✅ 列表滚动 >30 FPS

### 体验验收
- ✅ 所有操作可通过键盘完成 (无鼠标依赖)
- ✅ 错误提示清晰友好 (包含错误原因 + 解决建议)
- ✅ 帮助文档完善 (覆盖所有快捷键 + 常见故障)
- ✅ 最小支持 80x24 终端

### 安全验收
- ✅ 无命令注入漏洞 (渗透测试)
- ✅ 路径验证通过 (无法访问主目录外路径)
- ✅ 命令白名单验证通过
- ✅ 配置文件权限正确 (chmod 600)

---

## 12. 风险与缓解

| 风险 | 影响 | 缓解措施 | 状态 |
|------|------|----------|------|
| Ratatui 学习曲线 | 中 | 参考官方 examples 和 awesome-ratatui | ✅ 已识别 |
| 命令注入风险 | 高 | 使用 `current_dir()` 替代 shell 拼接 | ✅ 已修复 |
| 路径遍历风险 | 高 | 路径验证 + 白名单限制 | ✅ 已修复 |
| 跨平台兼容性 | 低 | 优先支持 macOS，后续扩展 | ✅ 已规划 |
| 大量仓库性能 | 中 | 虚拟列表 + 懒加载 + 缓存 | ✅ 已规划 |
| 命令执行权限 | 低 | `which` crate 预检查 | ✅ 已实现 |
| Git 状态检测性能 | 中 | 异步检测 + git2 crate | ✅ 已规划 |

---

## 13. 附录

### 13.1 参考项目
- [ratatui 官方示例](https://github.com/ratatui/ratatui/tree/main/examples)
- [broot](https://github.com/Canop/broot) - 文件浏览器 TUI
- [lazygit](https://github.com/jesseduffield/lazygit) - Git TUI

### 13.2 相关文件
- ghclone 原脚本：`/usr/local/bin/repotui`
- 目标主目录：`~/Developer/github` (可配置)

### 13.3 变更日志 (v1 → v2)
| 章节 | 变更内容 |
|------|----------|
| 2.1 | 添加安全验证要求 |
| 2.3 | 修复命令注入风险，使用安全执行方式 |
| 2.4 | 完善快捷键矩阵，解决冲突 |
| 3.1 | 更新技术栈版本，添加安全相关 crate |
| 3.2 | 细化项目结构 |
| 3.3 | 添加配置版本和验证逻辑 |
| 3.4 | 修正 View 签名，完善 Elm 架构 |
| 4 | 优化 UI 布局，添加响应式设计 |
| 5 | **新增**: 安全实现章节 |
| 6 | **新增**: 错误处理章节 |
| 7 | **新增**: 测试策略章节 |
| 8 | 量化性能指标 |
| 9 | **新增**: 跨平台测试矩阵 |
| 11 | 补充安全验收标准 |
