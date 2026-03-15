> **声明**: 本项目完全通过 vide coding 方式开发

[English Documentation](./README.md)

# repotui

一个用于浏览和管理 GitHub 仓库的终端用户界面（TUI）工具。

## 功能特性

- 🔍 **实时搜索**：输入时即时过滤仓库
- ⌨️ **键盘驱动**：Vim 风格的导航（j/k, g/G, Ctrl+d/u）
- 🎯 **快捷操作**：在 WebStorm、VS Code 或 Finder 中打开仓库
- 🔐 **安全**：命令白名单、路径验证、无 Shell 注入风险
- 🚀 **快速**：异步仓库扫描、虚拟列表渲染
- 🎨 **主题**：支持深色和浅色主题
- 📁 **目录发现**：显示配置主目录下的所有目录，Git 仓库显示分支信息，非 Git 目录显示"Not a git repo"提示

## 安装

### 一键安装（推荐）

macOS (仅 ARM64) 一键安装：

```bash
curl -fsSL https://raw.githubusercontent.com/ucefac/repo-tui/main/install.sh | bash
```

### 手动安装

```bash
# 克隆仓库
git clone https://github.com/ucefac/repo-tui.git
cd repo-tui

# Release 模式构建
cargo build --release

# 运行
./target/release/repo-tui
```

### 卸载

```bash
curl -fsSL https://raw.githubusercontent.com/ucefac/repo-tui/main/uninstall.sh | bash
```

或手动删除：

```bash
rm -rf ~/.config/repo-tui
```

## 快速开始

首次启动时，系统会提示您选择主仓库目录。

### 键盘快捷键

| 按键 | 操作 |
|-----|------|
| `j` / `↓` | 向下移动 |
| `k` / `↑` | 向上移动 |
| `g` | 跳转到顶部 |
| `G` | 跳转到底部 |
| `/` | 聚焦搜索框 |
| `Esc` | 清除搜索 |
| `Enter` / `o` | 打开操作菜单 |
| `c` | cd + cloud（claude） |
| `w` | 在 WebStorm 中打开 |
| `v` | 在 VS Code 中打开 |
| `f` | 在 Finder 中打开 |
| `r` | 刷新列表 |
| `?` | 显示帮助 |
| `q` | 退出 |

## 配置

配置文件位置：`~/.config/repo-tui/config.toml`

```toml
# 配置版本
version = "1.0"

# 扫描仓库的主目录
main_directory = "~/Developer/github"

# 编辑器配置
[editors]
webstorm = "/Applications/WebStorm.app/Contents/MacOS/webstorm"
vscode = "code"

# UI 配置
[ui]
theme = "dark"
show_git_status = true
show_branch = true

# 安全配置
[security]
allow_symlinks = false
max_search_depth = 2
```

## 安全性

repotui 实现了多项安全措施：

- **命令白名单**：只执行预先批准的命令
- **路径验证**：所有路径都验证是否在配置的主目录内
- **无 Shell 注入**：使用 `Command::current_dir()` 代替 Shell `cd` 命令
- **配置文件权限**：配置文件设置为 `chmod 600`（仅 Unix 系统）

## 开发

### 环境要求

- Rust 1.75+
- Git（用于仓库状态检测）

### 构建

```bash
# Debug 构建
cargo build

# Release 构建（优化版）
cargo build --release

# 使用 git2 后端（可选）
cargo build --features git2
```

### 测试

```bash
# 运行所有测试
cargo test

# 带覆盖率运行（需要 cargo-tarpaulin）
cargo tarpaulin --out Html
```

### 基准测试

```bash
cargo bench
```

### 代码检查

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## 架构

repotui 遵循 **Elm 架构**模式：

```
┌─────────────┐     ┌─────────────┐
│     Msg     │────▶│   Update    │
└─────────────┘     └──────┬──────┘
                           │
                           ▼
┌─────────────┐     ┌─────────────┐
│    View     │◀────│    Model    │
└─────────────┘     └─────────────┘
```

### 模块结构

- `app/`：应用状态（Model, Msg, Update）
- `ui/`：UI 渲染（View）
- `handler/`：键盘事件处理
- `config/`：配置管理
- `repo/`：仓库发现和状态检测
- `action/`：命令执行
- `runtime/`：异步任务执行

## 路线图

- [ ] 最近访问的仓库历史
- [ ] 仓库收藏夹
- [ ] 模糊搜索
- [ ] 文件系统监听器自动刷新
- [ ] 自定义快捷键
- [ ] 多配置文件支持

## 许可证

MIT 许可证 - 详情请参见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎贡献！请随时提交 Pull Request。

1. Fork 仓库
2. 创建您的功能分支（`git checkout -b feature/amazing-feature`）
3. 提交您的更改（`git commit -m 'Add some amazing feature'`）
4. 推送到分支（`git push origin feature/amazing-feature`）
5. 打开 Pull Request

## 致谢

- [ratatui](https://github.com/ratatui/ratatui) - TUI 框架
- [crossterm](https://github.com/crossterm-rs/crossterm) - 终端操作库
- [broot](https://github.com/Canop/broot) - 文件浏览器 TUI 灵感来源
- [lazygit](https://github.com/jesseduffield/lazygit) - Git TUI 灵感来源
