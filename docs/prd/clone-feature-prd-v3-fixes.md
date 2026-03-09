# PRD v3 修复总结报告

**文档**: clone-feature-prd-v3.md
**创建日期**: 2026-03-09
**修复依据**: Designer 和 Backend Developer 对 v2 的审查意见

---

## 修复的问题清单

### Designer 提出的必须修改问题

| 编号 | 问题 | 修复方式 | 位置 |
|------|------|----------|------|
| **D1** | URL 输入界面空间占用过大 | Examples 区域改为 `[?] Press '?' to show URL examples` 单行提示 | 2.2 节 UI 布局 |
| **D9** | 光标样式不一致 | 将 `___▌` 改为 `▌` 块状光标 | 2.2 节 UI 布局 |
| **D10** | 确认对话框按钮样式 | `[Y] Yes` → `[1] Yes`, `[N] No` → `[2] No` | 2.4 节 F-CLONE-4 |

### Backend Developer 提出的必须修改问题

| 编号 | 问题 | 修复方式 | 位置 |
|------|------|----------|------|
| **B5** | 进度发送机制不一致 | 更新 `clone_repository` 函数签名，使用 `msg_tx: mpsc::Sender<AppMsg>` 发送 `AppMsg::CloneProgress` | 3.3 节 |
| **B6** | 取消操作示例不完整 | 添加 `cancel_flag: Arc<AtomicBool>` 参数，展示如何定期检查取消标志并终止子进程 | 3.3 节 |
| **B7** | URL 验证不够严格 | 添加 `url = "2.5"` crate 建议，使用 `Url::parse()` 进行严格验证 | 5.1 节 |
| **B9** | 删除文件夹安全措施不足 | 添加 `.git` 目录验证代码示例，防止误删非 Git 目录 | 2.4 节 F-CLONE-4 |

---

## 详细修复内容

### 1. UI 布局优化 (D1, D9)

**修复前**:
```
│  │ https://github.com/user/repo___________________________▌│  │
│                                                               │
│  Examples:                                                    │
│    • https://github.com/owner/repo                            │
│    • https://gitlab.com/owner/repo                            │
│    • git@github.com:owner/repo.git                            │
```

**修复后**:
```
│  │ https://github.com/user/repo▌                          │  │
│                                                               │
│  [?] Press '?' to show URL examples                          │
```

**修复说明**:
- 光标样式统一为 `▌`，与现有 SearchBox 组件一致
- Examples 区域从 5 行压缩为 1 行，节省垂直空间
- 适应 80x24 最小终端尺寸

---

### 2. 确认对话框按钮样式 (D10)

**修复前**:
```
│            [Y] Yes, remove and re-clone                       │
│            [N] No, cancel                                     │
```

**修复后**:
```
│            [1] Yes, remove and re-clone                       │
│            [2] No, cancel                                     │
```

**修复说明**: 与现有 action_menu 使用数字快捷键的风格保持一致

---

### 3. Clone 执行模块 (B5, B6)

**修复前**:
```rust
pub async fn clone_repository(
    url: &str,
    target_path: &Path,
    progress_tx: mpsc::Sender<String>,
) -> Result<(), CloneError> {
    // ...
    progress_tx.send(line).await?;
}
```

**修复后**:
```rust
pub async fn clone_repository(
    url: &str,
    target_path: &Path,
    msg_tx: mpsc::Sender<AppMsg>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), CloneError> {
    // ...
    // 检查取消标志
    if cancel_flag.load(Ordering::Relaxed) {
        let _ = child.kill().await;
        return Err(CloneError::Cancelled);
    }
    // 发送进度消息
    msg_tx.send(AppMsg::CloneProgress(line)).await?;
}
```

**修复说明**:
- 使用 `msg_tx` 统一发送消息，符合 Elm 架构
- 添加 `cancel_flag` 支持取消操作
- 定期检查取消标志，及时终止 git 子进程

---

### 4. URL 验证 (B7)

**修复前**:
```rust
fn validate_git_url(url: &str) -> Result<(), UrlError> {
    // 手动字符串检查
    if url.contains(';') || url.contains('|') || url.contains('&') {
        return Err(UrlError::InvalidCharacters);
    }
    Ok(())
}
```

**修复后**:
```rust
// 依赖建议: url = "2.5"
use url::Url;

fn validate_git_url(url: &str) -> Result<(), UrlError> {
    // 使用 url crate 严格解析
    let parsed = Url::parse(url).map_err(|_| UrlError::InvalidFormat)?;

    // 协议白名单检查
    let allowed_schemes = ["https", "http", "git", "ssh"];
    if !allowed_schemes.contains(&parsed.scheme()) {
        return Err(UrlError::InvalidScheme(parsed.scheme().to_string()));
    }

    // 禁止 URL 以 '-' 开头（防止 git 命令注入）
    if parsed.path().starts_with("/-") {
        return Err(UrlError::InvalidCharacters);
    }
    // ...
}
```

**修复说明**:
- 添加 `url` crate 依赖建议
- 使用 `Url::parse()` 进行严格的 URL 解析
- 添加协议白名单和路径注入防护

---

### 5. 删除文件夹安全验证 (B9)

**新增内容**:
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

**修复说明**:
- 三重验证：主目录范围 + Git 仓库验证 + 敏感路径保护
- 防止误删非 Git 目录或系统关键路径

---

## 文档状态更新

| 文档 | 版本 | 状态 |
|------|------|------|
| clone-feature-prd-v1.md | v1 | 📝 已审查 |
| clone-feature-prd-v2.md | v2 | 📝 已审查 |
| clone-feature-prd-v3.md | v3 | ✅ **当前有效** |

---

## 建议

PRD v3 已修复所有 High Priority 的审查问题，建议：

1. **进入最终审查** - 可再次派遣 Designer 和 Backend Developer 快速确认
2. **直接进入开发** - 基于 v3 开始 Phase 1 开发
3. **冻结 PRD** - v3 作为开发基准，后续需求变更走变更流程
