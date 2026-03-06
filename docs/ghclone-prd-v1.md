# 📋 PRD: GitHub 仓库管理 TUI (repotui)

## 1. 产品概述

### 1.1 产品定位
基于 Ratatui 开发的终端用户界面工具，用于快速浏览、搜索和管理本地 GitHub 仓库，并提供一键进入仓库或打开编辑器的功能。

### 1.2 目标用户
需要在多个 GitHub 仓库之间快速切换的开发者

### 1.3 核心价值
- 🚀 秒级定位目标仓库
- ⌨️ 纯键盘操作，无需鼠标
- 🔍 实时搜索过滤
- 🎯 一键打开仓库或启动开发环境

---

## 2. 功能需求

### 2.1 启动流程

#### F1: 主目录选择 (首次启动)
- **触发条件**: 配置文件中不存在主目录配置
- **UI 组件**: 目录选择对话框
- **交互**:
  - 显示当前路径
  - 支持 `j/k` 或 `↑/↓` 浏览目录
  - 支持 `Enter` 确认选择
  - 支持 `q` 或 `Ctrl+C` 退出
- **输出**: 将主目录路径保存到配置文件

#### F2: 主界面加载
- **触发条件**: 配置文件存在有效主目录
- **UI 组件**: 仓库列表 + 搜索框
- **加载逻辑**:
  ```rust
  fn load_repositories(main_dir: &Path) -> Vec<Repository> {
      // 遍历主目录所有一级子目录
      // 过滤：仅保留 git 仓库 (存在 .git 目录或文件)
      // 返回 Repository { name, path, last_modified, is_dirty }
  }
  ```

### 2.2 仓库列表与搜索

#### F3: 实时搜索过滤
- **UI 布局**:
  ```
  ╭─ Search: facebook ______________________ ╮
  │ > github_facebook_react                  │
  │   github_facebook_vue                    │
  │   my_project_facebook_analytics          │
  │                                          │
  │ [3 repositories]                         │
  ╰──────────────────────────────────────────╯
  ```
- **交互**:
  - 任意字母键：追加到搜索框
  - `Backspace`: 删除字符
  - `Esc`: 清空搜索
  - 搜索结果实时更新（每次按键后重新过滤）

#### F4: 列表导航
- **按键映射**:
  | 按键 | 功能 |
  |------|------|
  | `j` / `↓` | 下移一项 |
  | `k` / `↑` | 上移一项 |
  | `g` | 跳转到第一项 |
  | `G` | 跳转到最后一项 |
  | `Ctrl+d` | 向下滚动半屏 |
  | `Ctrl+u` | 向上滚动半屏 |

### 2.3 仓库操作

#### F5: 操作菜单
- **触发**: 选中仓库后按 `Enter` 或 `o` (open)
- **UI 组件**: 弹出式操作菜单

```
╭─ Actions: github_facebook_react ─────────╮
│ [c] cd + cloud (claude)                  │
│ [w] Open in WebStorm                     │
│ [v] Open in VS Code                      │
│ [f] Open in Finder/Explorer              │
│ [q] Cancel                               │
╰──────────────────────────────────────────╯
```

#### F6: 命令执行
| 操作 | 按键 | 实现 |
|------|------|------|
| cd + cloud | `c` | `cd <path> && claude` |
| WebStorm | `w` | `webstorm <absolute_path>` |
| VS Code | `v` | `code <absolute_path>` |
| 文件管理器 | `f` | macOS: `open <path>`, Linux: `xdg-open`, Windows: `explorer` |
| 取消 | `q` / `Esc` | 关闭菜单 |

### 2.4 全局快捷键

| 按键 | 功能 |
|------|------|
| `q` / `Ctrl+C` | 退出程序 |
| `r` | 刷新仓库列表 |
| `?` | 显示帮助面板 |
| `/` | 聚焦搜索框 |

---

## 3. 技术架构

### 3.1 技术栈
```yaml
语言：Rust 1.75+
TUI 框架：ratatui 0.24+
终端后端：crossterm 0.27+
配置管理：serde + toml
路径处理：path-absolutize
异步：tokio (可选，用于后台 git 状态检测)
```

### 3.2 项目结构
```
repotui/
├── Cargo.toml
├── src/
│   ├── main.rs              # 程序入口
│   ├── app.rs               # 应用状态 (Model)
│   ├── ui.rs                # UI 渲染 (View)
│   ├── handler.rs           # 键盘事件处理 (Update)
│   ├── config.rs            # 配置管理
│   ├── repo.rs              # 仓库操作
│   └── action.rs            # 命令执行
├── config/
│   └── config.toml.example
└── README.md
```

### 3.3 配置格式 (`~/.config/repotui/config.toml`)
```toml
# 主目录路径 (必需)
main_directory = "/home/username/projects"

# 编辑器路径 (可选，使用系统 PATH 时可省略)
editors = { webstorm = "/Applications/WebStorm.app/Contents/MacOS/webstorm", vscode = "code" }

# 默认命令 (可选)
default_command = "claude"

# UI 配置 (可选)
[ui]
theme = "dark"  # dark | light
show_git_status = true
```

### 3.4 数据流 (Elm 架构)
```rust
// Model
struct App {
    main_dir: PathBuf,
    repositories: Vec<Repository>,
    filtered_indices: Vec<usize>,  // 过滤后的索引
    search_query: String,
    selected_index: usize,
    state: AppState,  // Running, ChoosingDir, ShowingActions, Quit
}

// Msg
enum AppMsg {
    SearchInput(char),
    NextRepo,
    PreviousRepo,
    OpenActions,
    ExecuteAction(Action),
    ConfigLoaded(Config),
    // ...
}

// Update
fn update(msg: AppMsg, app: &mut App) -> Option<Cmd> {
    match msg {
        AppMsg::SearchInput(c) => {
            app.search_query.push(c);
            app.filtered_indices = filter_repos(&app.repositories, &app.search_query);
            app.selected_index = 0;
        }
        // ...
    }
}

// View
fn view(app: &App) -> Text {
    // 使用 Ratatui widgets 渲染
}
```

---

## 4. UI 设计

### 4.1 主界面布局
```
╭─ repotui ─────────────────────────────────────────────╮
│ Search: react ___________________________________________ │
│                                                           │
│ ╭─ Repositories (3) ────────────────────────────────────╮ │
│ │ > github_facebook_react                               │ │
│ │   web_react_native_docs                               │ │
│ │   personal_react_playground                           │ │
│ │                                                       │ │
│ │                                                       │ │
│ │                                                       │ │
│ ╰───────────────────────────────────────────────────────╯ │
│                                                           │
│ [j/k] Navigate  [Enter] Open  [r] Refresh  [q] Quit      │
╰───────────────────────────────────────────────────────────╯
```

### 4.2 目录选择界面
```
╭─ Select Main Directory ───────────────────────────────────╮
│                                                           │
│   ../                                                    │
│   Desktop/                                               │
│   Documents/                                             │
│ > Developer/                                             │
│   Downloads/                                             │
│   Music/                                                 │
│                                                           │
│ Current: /home/username/projects                         │
│                                                           │
│ [j/k] Navigate  [Enter] Select  [q] Cancel               │
╰───────────────────────────────────────────────────────────╯
```

### 4.3 帮助面板
```
╭─ Keyboard Shortcuts ──────────────────────────────────────╮
│ Navigation                                                │
│   j/↓     Move down                                      │
│   k/↑     Move up                                        │
│   g       Go to top                                      │
│   G       Go to bottom                                   │
│                                                           │
│ Search                                                    │
│   /       Focus search                                   │
│   Esc     Clear search                                   │
│   [char]  Add to search query                            │
│                                                           │
│ Actions                                                   │
│   Enter   Open action menu                               │
│   r       Refresh list                                   │
│   ?       Show this help                                 │
│   q       Quit                                           │
╰───────────────────────────────────────────────────────────╯
```

---

## 5. 实现细节

### 5.1 Git 仓库检测
```rust
fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

fn get_git_status(path: &Path) -> GitStatus {
    // 使用 git2 crate 或执行 git status --porcelain
    // 返回：是否有未提交更改
}
```

### 5.2 搜索算法
```rust
fn filter_repos(repos: &[Repository], query: &str) -> Vec<usize> {
    repos.iter()
        .enumerate()
        .filter(|(_, repo)| {
            repo.name.to_lowercase().contains(&query.to_lowercase())
        })
        .map(|(i, _)| i)
        .collect()
}
```

### 5.3 命令执行
```rust
fn execute_command(action: Action, repo: &Repository) -> std::io::Result<()> {
    match action {
        Action::CdAndCloud => {
            // 需要特殊处理：启动新 shell
            std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("cd '{}' && claude", repo.path.display()))
                .status()?;
        }
        Action::WebStorm => {
            std::process::Command::new("webstorm")
                .arg(&repo.path)
                .status()?;
        }
        Action::VsCode => {
            std::process::Command::new("code")
                .arg(&repo.path)
                .status()?;
        }
        // ...
    }
    Ok(())
}
```

---

## 6. 开发计划

### Phase 1: MVP (1-2 周)
- [ ] 项目脚手架搭建
- [ ] 配置文件读写
- [ ] 主目录选择界面
- [ ] 仓库列表渲染
- [ ] 基础搜索功能
- [ ] 键盘导航

### Phase 2: 核心功能 (1 周)
- [ ] 操作菜单
- [ ] cd + cloud 命令执行
- [ ] 编辑器打开功能
- [ ] 帮助面板

### Phase 3: 增强体验 (1 周)
- [ ] Git 状态检测与显示
- [ ] 主题支持
- [ ] 性能优化（大量仓库时）
- [ ] 错误处理与用户提示

### Phase 4: 可选增强
- [ ] 最近打开记录
- [ ] 收藏夹功能
- [ ] 仓库元数据显示（分支、最后提交时间）
- [ ] 批量操作

---

## 7. 验收标准

### 功能验收
- ✅ 首次启动能选择并保存主目录
- ✅ 正确列出主目录下的所有 git 仓库
- ✅ 搜索框实时过滤（<50ms 响应）
- ✅ 键盘导航流畅无卡顿
- ✅ 能正确执行 cd+cloud 命令
- ✅ 能正确打开 WebStorm 和 VS Code

### 性能验收
- ✅ 启动时间 < 500ms
- ✅ 支持 1000+ 仓库无性能问题
- ✅ 内存占用 < 50MB

### 体验验收
- ✅ 所有操作可通过键盘完成
- ✅ 错误提示清晰友好
- ✅ 帮助文档完善

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Ratatui 学习曲线 | 中 | 参考官方 examples 和 awesome-ratatui |
| 跨平台兼容性 | 低 | 优先支持 macOS，后续扩展 |
| 大量仓库性能 | 中 | 使用虚拟列表，懒加载 |
| 命令执行权限 | 低 | 添加命令存在性检查 |

---

## 9. 附录

### 9.1 参考项目
- [ratatui 官方示例](https://github.com/ratatui/ratatui/tree/main/examples)
- [broot](https://github.com/Canop/broot) - 文件浏览器 TUI
- [lazygit](https://github.com/jesseduffield/lazygit) - Git TUI

### 9.2 相关文件
- ghclone 原脚本：`/usr/local/bin/repotui`
- 目标主目录：`~/Developer/github` (可配置)
