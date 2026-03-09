# PRD: Git Clone 功能 - ghclone-tui

**文档版本**: v3-final
**更新日期**: 2026-03-09
**更新说明**: 基于 Designer 和 Backend Developer 审查意见，修复所有 High Priority 问题
**审查状态**: ✅ Designer 通过 | ✅ Backend Developer 通过
**功能概述**: 仓库列表界面新增 "Clone" 功能，支持从 GitHub 等平台 clone 仓库到本地主目录

---

## 1. 需求概述

### 1.1 功能定位
在 repotui 的仓库列表界面增加一键 clone 功能，允许用户通过简单的键盘操作从远程 Git 仓库下载代码到本地指定主目录。

### 1.2 目标用户
- 需要快速获取远程仓库代码的开发者
- 管理多个 GitHub/GitLab 项目的用户

### 1.3 核心价值
| 价值点 | 说明 |
|--------|------|
| ⚡ 快捷操作 | 一键触发 (c)，无需离开 TUI 工具 |
| 🎯 智能命名 | 自动生成规范化的文件夹名称 |
| 📊 实时反馈 | 显示 clone 进度，提供良好用户体验 |
| 🔄 自动刷新 | 完成后自动刷新仓库列表并定位 |

---

## 2. 功能需求

### 2.1 触发方式

**F-CLONE-1: 快捷键触发**
- **按键**: `c`
- **触发条件**: 当前处于 `Running` 状态（仓库列表界面）
- **行为**: 进入 Clone 流程，显示 URL 输入界面
- **优先级**: 与搜索框 (`/`) 同级，在 `Running` 状态拦截

**状态优先级更新**:
```
Cloning (6) > ActionMenu (5) > Help (4) > ChoosingDir (3) > Searching (2) > Running (1)
```

### 2.2 URL 输入界面（含主目录选择）

**F-CLONE-2: URL 输入框**
- **UI 布局**:
  ```
  ╭─ Clone Repository ───────────────────────────────────────────╮
  │                                                               │
  │  Enter Git repository URL:                                    │
  │  ┌─────────────────────────────────────────────────────────┐ │
  │  │ https://github.com/user/repo▌                          │ │
  │  └─────────────────────────────────────────────────────────┘ │
  │                                                               │
  │  [?] Press '?' to show URL examples                          │
  │                                                               │
  │  ─────────────────────────────────────────────────────────────│
  │                                                               │
  │  Target directory:                                            │
  │                                                               │
  │  ▌ ~/Projects/github                                          │
  │    ~/Work/repositories                                        │
  │    ~/Personal/projects                                        │
  │                                                               │
  │            [↑↓] Select Target   [Enter] Confirm   [Esc] Cancel│
  ╰───────────────────────────────────────────────────────────────╯
  ```
- **单主目录场景**: 如果配置中只有一个启用的主目录，下方直接显示该路径，无需选择
  ```
  │  Target directory: ~/Projects/github                          │
  │                                                               │
  │            [Enter] Confirm   [Esc] Cancel                     │
  ```
- **多主目录场景**: 显示主目录列表，用户通过 `↑/↓` 选择目标主目录
- **输入验证**:
  - 实时验证 URL 格式（支持 HTTP/HTTPS/SSH）
  - 支持 GitHub、GitLab、Bitbucket、Gitee 等平台
  - 无效 URL 显示红色错误提示
- **快捷键**:
  - `Enter`: 确认 URL 和主目录选择，进入下一步
  - `Esc`: 取消，返回仓库列表
  - `Backspace`: 删除字符
  - `Ctrl+k`: 清空输入（从光标位置到行尾）
  - `↑/↓`: 在多主目录场景下选择目标主目录
  - **Bracketed Paste**: 支持终端原生粘贴（自动识别批量粘贴内容）
- **粘贴支持**:
  - 实现方式: 使用 crossterm 的 `Event::Paste(String)` 事件
  - 需要在应用启动时启用: `crossterm::execute!(stdout(), EnableBracketedPaste)?`
  - 粘贴内容自动追加到 URL 输入框光标位置

### 2.3 文件夹命名规则

**F-CLONE-3: 自动生成文件夹名**
- **格式**: `{domain}_{owner}_{repo_name}`
- **转换规则**:
  - 移除 URL 协议部分 (`https://`, `git@`, 等)
  - 提取域名（不含 www）
  - 提取仓库所有者名称
  - 提取仓库名称（移除 `.git` 后缀）
  - 特殊字符替换为下划线
- **示例**:
  | 输入 URL | 生成文件夹名 |
  |----------|--------------|
  | `https://github.com/farion1231/cc-switch` | `github_farion1231_cc-switch` |
  | `https://github.com/facebook/react.git` | `github_facebook_react` |
  | `https://gitlab.com/user/my-project` | `gitlab_user_my-project` |
  | `git@github.com:rust-lang/rust.git` | `github_rust-lang_rust` |

### 2.4 文件夹已存在处理

**F-CLONE-4: 冲突检测与提示**
- **检测时机**: 确认主目录后，执行 clone 前
- **检测逻辑**: 检查 `{target_dir}/{generated_name}` 是否存在
- **UI 布局**:
  ```
  ╭─ Repository Already Exists ──────────────────────────────────╮
  │                                                               │
  │  ⚠️  Folder already exists:                                   │
  │     ~/Projects/github/github_farion1231_cc-switch             │
  │                                                               │
  │  Do you want to remove it and re-clone?                       │
  │                                                               │
  │            [1] Yes, remove and re-clone                       │
  │            [2] No, cancel                                     │
  │                                                               │
  ╰───────────────────────────────────────────────────────────────╯
  ```
- **选项**:
  - `Y/Enter`: 删除旧文件夹，执行 clone
  - `N/Esc`: 取消操作，返回仓库列表
- **安全要求**:
  1. 删除前确认路径在允许的主目录内
  2. **验证目标是 Git 仓库**: 检查目标路径下存在 `.git` 目录，防止误删非 Git 目录
  3. **安全检查代码**:
  ```rust
  fn validate_folder_replace(path: &Path, allowed_dirs: &[PathBuf]) -> Result<(), ReplaceError> {
      // 1. 路径必须在允许的主目录内
      let canonical = path.canonicalize()?;
      let in_allowed = allowed_dirs.iter().any(|d| canonical.starts_with(d));
      if !in_allowed {
          return Err(ReplaceError::OutsideAllowedDirectory);
      }

      // 2. 验证是 Git 仓库（防止误删普通文件夹）
      let git_dir = path.join(".git");
      if !git_dir.exists() {
          return Err(ReplaceError::NotAGitRepository);
      }

      // 3. 禁止删除家目录、系统目录等敏感路径
      let home = dirs::home_dir().ok_or(ReplaceError::HomeNotFound)?;
      if path == home || path == Path::new("/") {
          return Err(ReplaceError::ProtectedPath);
      }

      Ok(())
  }
  ```

### 2.5 Clone 执行与进度显示

**F-CLONE-5: 进度显示界面**
- **UI 布局**:
  ```
  ╭─ Cloning Repository ─────────────────────────────────────────╮
  │                                                               │
  │  URL:    https://github.com/farion1231/cc-switch              │
  │  Target: ~/Projects/github/github_farion1231_cc-switch        │
  │                                                               │
  │  ┌─────────────────────────────────────────────────────────┐ │
  │  │ Cloning into 'github_farion1231_cc-switch'...           │ │
  │  │ remote: Enumerating objects: 142, done.                 │ │
  │  │ remote: Counting objects: 100% (142/142), done.         │ │
  │  │ remote: Compressing objects: 100% (98/98), done.        │ │
  │  │ Receiving objects: 45% (64/142), 1.2 MiB | 2.5 MiB/s   │ │
  │  │ ...                                                     │ │
  │  └─────────────────────────────────────────────────────────┘ │
  │                                                               │
  │            [Esc] Cancel                                       │
  ╰───────────────────────────────────────────────────────────────╯
  ```
- **行为**:
  - 实时显示 git clone 输出
  - 支持取消操作（发送 SIGTERM 给 git 进程）
  - 自动滚动显示最新输出
- **实现方式**: 异步执行 `git clone`，通过管道捕获 stdout/stderr

### 2.6 完成处理

**F-CLONE-6: 成功处理**
- **UI 反馈**: 显示成功提示 2 秒后自动关闭
  ```
  ╭─ Clone Completed ────────────────────────────────────────────╮
  │  ✅ Successfully cloned to:                                   │
  │     ~/Projects/github/github_farion1231_cc-switch             │
  ╰───────────────────────────────────────────────────────────────╯
  ```
- **自动操作**:
  1. 刷新仓库列表（触发 `Cmd::LoadRepositories`）
  2. 自动选中新添加的仓库
  3. 返回 `Running` 状态

**F-CLONE-7: 错误处理**
- **错误类型**:
  - 网络连接失败
  - 仓库不存在/无权限
  - 磁盘空间不足
  - git 命令未安装
- **UI 反馈**:
  ```
  ╭─ Clone Failed ───────────────────────────────────────────────╮
  │  ❌ Failed to clone repository                                │
  │                                                               │
  │  Error: Repository not found                                  │
  │  Check the URL and try again.                                 │
  │                                                               │
  │            [Enter] OK   [R] Retry                             │
  ╰───────────────────────────────────────────────────────────────╯
  ```

---

## 3. 技术架构

### 3.1 Elm 架构扩展

**Model 扩展**:
```rust
pub struct App {
    // ... 现有字段
    pub clone_state: Option<CloneState>,
}

pub struct CloneState {
    pub url_input: String,
    pub cursor_position: usize,              // 新增：光标位置
    pub parsed_url: Option<ParsedGitUrl>,
    pub target_main_dir: Option<usize>,      // 主目录索引
    pub stage: CloneStage,
    pub progress_lines: Vec<String>,
    pub main_dir_list_state: ListState,      // 新增：主目录列表状态
    pub cancel_flag: Arc<AtomicBool>,        // 新增：取消标志
}

pub enum CloneStage {
    InputUrl,
    SelectMainDir,
    ConfirmReplace { existing_path: PathBuf },
    Executing,
    Error(CloneError),                        // 修改为 Error 状态保留用户输入
}

pub struct ParsedGitUrl {
    pub domain: String,
    pub owner: String,
    pub repo: String,
    pub original_url: String,
}
```

**Msg 扩展**:
```rust
pub enum AppMsg {
    // ... 现有消息
    // Clone 相关
    StartClone,
    CloneUrlInput(char),
    CloneUrlPaste(String),    // 粘贴事件（支持 bracketed paste）
    CloneUrlBackspace,
    CloneUrlClear,
    CloneUrlConfirm,
    CloneSelectMainDir(usize),
    CloneConfirmReplace(bool),
    CloneProgress(String),
    CloneCompleted(Result<Repository, CloneError>),
    CloneRetry,
    CancelClone,
}
```

**Cmd 扩展**:
```rust
pub enum Cmd {
    // ... 现有命令
    CloneRepository {
        url: String,
        target_path: PathBuf,
        msg_tx: mpsc::Sender<AppMsg>,
    },
}
```

**AppState 扩展**:
```rust
pub enum AppState {
    // ... 现有状态
    Cloning,
}
```

### 3.2 URL 解析模块

**文件**: `src/repo/clone.rs` (新建)

```rust
/// 解析 Git URL
pub fn parse_git_url(url: &str) -> Result<ParsedGitUrl, UrlError> {
    // 支持格式:
    // - https://github.com/owner/repo
    // - https://github.com/owner/repo.git
    // - git@github.com:owner/repo.git
    // - ssh://git@github.com/owner/repo.git
}

/// 生成文件夹名称
pub fn generate_folder_name(parsed: &ParsedGitUrl) -> String {
    format!("{}_{}_{}",
        sanitize(&parsed.domain),
        sanitize(&parsed.owner),
        sanitize(&parsed.repo)
    )
}

/// 清理特殊字符
fn sanitize(s: &str) -> String {
    s.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")
}
```

### 3.3 Clone 执行模块

**异步执行**:
```rust
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn clone_repository(
    url: &str,
    target_path: &Path,
    msg_tx: mpsc::Sender<AppMsg>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), CloneError> {
    let mut child = Command::new("git")
        .args(&["clone", "--progress", url, target_path.to_str().unwrap()])
        .stderr(std::process::Stdio::piped())  // git clone 输出到 stderr
        .spawn()?;

    // 读取进度输出
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            // 检查取消标志
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = child.kill().await;
                return Err(CloneError::Cancelled);
            }
            // 发送进度消息
            msg_tx.send(AppMsg::CloneProgress(line)).await?;
        }
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(CloneError::GitFailed(status.code()));
    }

    Ok(())
}
```

---

## 4. UI 设计

### 4.1 组件列表

| 组件 | 文件 | 说明 |
|------|------|------|
| CloneDialog | `src/ui/widgets/clone_dialog.rs` | 主对话框容器 |
| UrlInput | `src/ui/widgets/clone_dialog.rs` | URL 输入框 |
| MainDirSelector | `src/ui/widgets/clone_dialog.rs` | 主目录选择器 |
| ConfirmDialog | `src/ui/widgets/clone_dialog.rs` | 确认对话框 |
| ProgressView | `src/ui/widgets/clone_dialog.rs` | 进度显示区域 |

### 4.2 样式规范

**复用现有主题**:
- 边框颜色: `theme.colors.border` / `theme.colors.border_focused`
- 文本颜色: `theme.colors.foreground` / `theme.colors.text_muted`
- 强调色: `theme.colors.primary`
- 成功色: `theme.colors.success`
- 错误色: `theme.colors.error`

**弹窗尺寸**:
- URL 输入: 60% x 40%
- 主目录选择: 50% x 50%
- 确认对话框: 50% x 30%
- 进度显示: 70% x 60%

### 4.3 快捷键一致性

保持与现有 UI 一致的快捷键风格:
- `Enter`: 确认
- `Esc`: 取消/返回
- `↑/↓`: 导航
- `Backspace`: 删除

---

## 5. 安全设计

### 5.1 URL 验证

**依赖建议**: 添加 `url = "2.5"` 到 Cargo.toml 进行严格的 URL 解析验证

```rust
use url::Url;

fn validate_git_url(url: &str) -> Result<(), UrlError> {
    // 1. 长度检查 (防止超长输入)
    if url.len() > MAX_URL_LENGTH {
        return Err(UrlError::TooLong);
    }

    // 2. 使用 url crate 严格解析
    let parsed = Url::parse(url).map_err(|_| UrlError::InvalidFormat)?;

    // 3. 协议白名单
    let allowed_schemes = ["https", "http", "git", "ssh"];
    if !allowed_schemes.contains(&parsed.scheme()) {
        return Err(UrlError::InvalidScheme(parsed.scheme().to_string()));
    }

    // 4. 禁止 URL 以 '-' 开头（防止 git 命令注入）
    if parsed.path().starts_with("/-") {
        return Err(UrlError::InvalidCharacters);
    }

    // 5. 域名白名单 (可选配置)
    let allowed_domains = config.allowed_git_domains.unwrap_or_default();
    if !allowed_domains.is_empty() {
        let domain = parsed.host_str().unwrap_or("");
        if !allowed_domains.iter().any(|d| domain.ends_with(d)) {
            return Err(UrlError::UnsupportedHost(domain.to_string()));
        }
    }

    Ok(())
}
```

### 5.2 路径安全

```rust
fn validate_target_path(path: &Path, allowed_dirs: &[PathBuf]) -> Result<(), PathError> {
    // 1. 路径必须在允许的主目录内
    let canonical = path.canonicalize()?;
    let in_allowed = allowed_dirs.iter().any(|d| canonical.starts_with(d));

    if !in_allowed {
        return Err(PathError::OutsideAllowedDirectory);
    }

    // 2. 防止路径遍历
    if path.components().any(|c| c == std::path::Component::ParentDir) {
        return Err(PathError::PathTraversal);
    }

    Ok(())
}
```

### 5.3 命令安全

- 使用 `tokio::process::Command` 直接执行，不使用 shell
- URL 作为参数传递，不拼接命令字符串
- 禁止 `--exec` 等危险参数

---

## 6. 错误处理

### 6.1 错误类型

```rust
#[derive(Error, Debug)]
pub enum CloneError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("URL too long (max {MAX_URL_LENGTH} characters)")]
    UrlTooLong,

    #[error("Unsupported Git host: {0}")]
    UnsupportedHost(String),

    #[error("Repository already exists at: {0}")]
    AlreadyExists(PathBuf),

    #[error("Git command failed with code: {0:?}")]
    GitFailed(Option<i32>),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Disk full")]
    DiskFull,

    #[error("Git not installed")]
    GitNotFound,

    #[error("Operation cancelled by user")]
    Cancelled,
}
```

### 6.2 用户友好提示

| 错误类型 | 提示消息 | 建议操作 |
|----------|----------|----------|
| InvalidUrl | "Invalid repository URL. Please check the format." | 显示 URL 格式示例 |
| Network | "Network error. Please check your connection." | 提供重试选项 |
| GitFailed | "Failed to clone repository. The repository may not exist or is private." | 检查 URL 和权限 |
| PermissionDenied | "Permission denied. Check directory permissions." | 检查主目录权限 |
| DiskFull | "Not enough disk space." | 清理磁盘空间 |

---

## 7. 测试策略

### 7.1 单元测试

| 测试项 | 测试内容 | 预期结果 |
|--------|----------|----------|
| URL 解析 | 各种格式 URL 解析 | 正确提取 domain/owner/repo |
| 文件夹命名 | 特殊字符处理 | 生成合法文件名 |
| 路径验证 | 路径遍历攻击 | 正确拒绝非法路径 |
| 主目录选择 | 单/多主目录场景 | 正确分支处理 |

### 7.2 集成测试

```rust
#[tokio::test]
async fn test_clone_flow_success() {
    // 1. 触发 StartClone
    // 2. 模拟 URL 输入
    // 3. 模拟主目录选择
    // 4. 验证 Cmd::CloneRepository 发送
    // 5. 模拟 CloneCompleted
    // 6. 验证仓库列表刷新
}

#[tokio::test]
async fn test_clone_existing_folder() {
    // 验证已存在文件夹的确认流程
}
```

### 7.3 E2E 测试场景

1. **正常流程**: 输入 URL → 选择主目录 → clone → 成功 → 自动选中
2. **取消流程**: 输入 URL → 按 Esc → 返回仓库列表
3. **替换流程**: 输入 URL → 检测到已存在 → 确认替换 → 成功
4. **错误流程**: 输入无效 URL → 显示错误 → 可以重试

---

## 8. 验收标准

### 8.1 功能验收

- ✅ 按 `c` 键正确触发 Clone 流程 (F-CLONE-1)
- ✅ 支持多种 Git URL 格式 (F-CLONE-2)
- ✅ URL 输入框支持粘贴功能（Bracketed Paste）(F-CLONE-2)
- ✅ 单主目录时自动使用，多主目录时在 URL 界面中选择 (F-CLONE-2)
- ✅ 文件夹命名符合 `{domain}_{owner}_{repo}` 规范 (F-CLONE-3)
- ✅ 检测到已存在文件夹时提示确认 (F-CLONE-4)
- ✅ 实时显示 git clone 进度 (F-CLONE-5)
- ✅ 完成后自动刷新仓库列表并选中新仓库 (F-CLONE-6)
- ✅ 支持全程按 Esc 取消 (F-CLONE-7)
- ✅ 错误状态支持重试 (F-CLONE-7)

### 8.2 安全验收

- ✅ URL 验证防止注入攻击
- ✅ 目标路径限制在配置的主目录内
- ✅ 使用安全的命令执行方式
- ✅ 删除文件夹前需要用户确认

### 8.3 体验验收

- ✅ 所有操作可通过键盘完成
- ✅ 错误提示清晰友好
- ✅ 界面风格与现有 UI 一致
- ✅ 进度显示流畅无卡顿

---

## 9. 开发计划

### Phase 1: 基础结构 (Day 1)
- [ ] 扩展 AppState、AppMsg、Cmd 类型
- [ ] 实现 URL 解析模块
- [ ] 实现文件夹命名生成

### Phase 2: UI 组件 (Day 2)
- [ ] 实现 CloneDialog 组件
- [ ] 实现 URL 输入界面（含主目录选择）
- [ ] 实现确认对话框
- [ ] 实现进度显示界面

### Phase 3: 业务逻辑 (Day 3)
- [ ] 实现 update.rs 中的 clone 逻辑
- [ ] 实现 keyboard.rs 中的按键处理（含粘贴支持）
- [ ] 实现 runtime executor 中的 Cmd 处理
- [ ] 实现 render.rs 中的渲染

### Phase 4: 集成测试 (Day 4)
- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 手动测试各种场景

---

## 10. 相关文档

- [设计文档: keyboard-shortcuts.md](../design/keyboard-shortcuts.md)
- [现有 PRD: ghclone-prd-v2.md](./ghclone-prd-v2.md)
- [代码架构: CLAUDE.md](../../CLAUDE.md)

---

## 11. 附录

### 11.1 URL 格式支持

| 格式 | 示例 |
|------|------|
| HTTPS | `https://github.com/owner/repo` |
| HTTPS with .git | `https://github.com/owner/repo.git` |
| SSH | `git@github.com:owner/repo.git` |
| SSH full | `ssh://git@github.com/owner/repo.git` |
| Git 协议 | `git://github.com/owner/repo.git` |

### 11.2 支持的 Git 平台

- GitHub (`github.com`)
- GitLab (`gitlab.com` 及自建)
- Bitbucket (`bitbucket.org`)
- Gitee (`gitee.com`)
- Gitea (自建)
- 其他标准 Git 托管平台
