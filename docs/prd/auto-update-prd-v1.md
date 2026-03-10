# PRD: 自动检测更新功能 (repotui-auto-update) - v1

**文档版本**: v1
**更新日期**: 2026-03-10
**审查状态**: 📝 待审查
**关联功能**: 版本管理、用户通知

---

## 1. 产品概述

### 1.1 功能定位
为 repotui 添加自动检测更新功能，程序运行期间每天自动检查一次 GitHub Releases 是否有新版本，并在 TUI 界面中友好地通知用户。

### 1.2 目标用户
使用 repotui 管理 GitHub 仓库的开发者，希望及时获取软件更新。

### 1.3 核心价值

| 价值点 | 说明 |
|--------|------|
| 📢 及时通知 | 自动检测新版本，不错过重要更新 |
| ⚙️ 可配置 | 用户可自主选择是否启用自动检测 |
| 🔄 非侵入 | 后台静默检测，不影响正常使用 |
| 🔒 安全 | 仅访问 GitHub API，无敏感数据传输 |

---

## 2. 功能需求

### 2.1 版本检测机制

#### F1: 版本来源
- **来源**: GitHub Releases API
- **API 端点**: `https://api.github.com/repos/{owner}/{repo}/releases/latest`
- **版本格式**: 语义化版本 (Semantic Versioning)，如 `v0.1.0`, `v1.2.3`
- **当前版本来源**: `Cargo.toml` 中的 `package.version` 字段

#### F2: 检测频率
- **默认频率**: 每天检测一次（程序运行期间）
- **首次检测**: 程序启动后延迟 5 分钟（避免影响启动速度）
- **触发条件**:
  - 程序启动后首次运行
  - 距上次检测超过 24 小时
  - 用户手动触发（可选快捷键）

#### F3: 版本比较逻辑
```rust
/// 版本比较结果
enum VersionComparison {
    /// 当前版本是最新
    UpToDate,
    /// 有新版本可用
    UpdateAvailable { latest: String, current: String },
    /// 当前版本比最新版还新（开发版本）
    Ahead,
}

/// 解析版本字符串 (移除 'v' 前缀)
fn parse_version(version: &str) -> Result<semver::Version, VersionError> {
    let clean = version.trim_start_matches('v');
    semver::Version::parse(clean)
        .map_err(|e| VersionError::InvalidVersion(e.to_string()))
}

/// 比较版本
fn compare_versions(current: &str, latest: &str) -> Result<VersionComparison, VersionError> {
    let current_ver = parse_version(current)?;
    let latest_ver = parse_version(latest)?;

    match current_ver.cmp(&latest_ver) {
        std::cmp::Ordering::Less => Ok(VersionComparison::UpdateAvailable {
            latest: latest.to_string(),
            current: current.to_string(),
        }),
        std::cmp::Ordering::Equal => Ok(VersionComparison::UpToDate),
        std::cmp::Ordering::Greater => Ok(VersionComparison::Ahead),
    }
}
```

### 2.2 用户通知

#### F4: 更新提示 UI
当检测到有新版本时，在 TUI 界面标题栏或状态栏显示更新提示：

```
╭─ ghclone v0.1.0 ──────────────────────────────────── [⬆ Update: v0.2.0] ─╮
│ 🔍 Search: [________________]                                  [15/342] │
```

**显示规则**:
- 在标题栏右侧显示 `[⬆ Update: v{x.x.x}]`
- 使用醒目的颜色（如黄色或绿色）
- 用户按 `U` 键可查看更新详情

#### F5: 更新详情弹窗
按 `U` 键打开更新详情弹窗：

```
╭─ Update Available ────────────────────────────────────────╮
│                                                           │
│  A new version of repotui is available!                   │
│                                                           │
│  Current version:  v0.1.0                                 │
│  Latest version:   v0.2.0                                 │
│                                                           │
│  Release Notes:                                           │
│  ──────────────────────────────────────────────────────   │
│  • Added fuzzy search support                             │
│  • Improved performance for large repositories            │
│  • Fixed bug with special characters in paths             │
│                                                           │
│  [Open Release Page]  [Remind Later]  [Disable Auto-check]│
│                                                           │
╰───────────────────────────────────────────────────────────╯
```

**交互选项**:
- `Enter` / `O`: 打开浏览器访问 GitHub Release 页面
- `Esc` / `R`: 关闭弹窗，24 小时后再次提醒
- `D`: 禁用自动检测功能

### 2.3 配置选项

#### F6: 配置结构
在 `config.toml` 中添加更新检测配置：

```toml
[update]
# 是否启用自动检测 (默认: true)
enabled = true

# 检测频率 (小时，默认: 24)
check_interval_hours = 24

# 首次检测延迟 (分钟，默认: 5)
initial_delay_minutes = 5

# GitHub 仓库信息
[update.github]
owner = "repotui"
repo = "repotui"
```

#### F7: 配置类型定义
```rust
// src/config/types.rs

/// Update check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Enable automatic update check
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Check interval in hours
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,

    /// Initial delay before first check (minutes)
    #[serde(default = "default_initial_delay")]
    pub initial_delay_minutes: u64,

    /// GitHub repository info
    #[serde(default)]
    pub github: GithubRepoConfig,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_hours: 24,
            initial_delay_minutes: 5,
            github: GithubRepoConfig::default(),
        }
    }
}

/// GitHub repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRepoConfig {
    /// Repository owner
    #[serde(default = "default_owner")]
    pub owner: String,

    /// Repository name
    #[serde(default = "default_repo")]
    pub repo: String,
}

impl Default for GithubRepoConfig {
    fn default() -> Self {
        Self {
            owner: "repotui".to_string(),
            repo: "repotui".to_string(),
        }
    }
}
```

---

## 3. 技术架构

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         repotui Application                         │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │    Model    │  │    View     │  │   Update    │  │   Cmd       │ │
│  │             │  │             │  │             │  │             │ │
│  │ - update_   │  │ - Title bar │  │ - CheckUpdate│  │ - CheckUpdate│ │
│  │   available │  │   indicator │  │ - Dismiss   │  │   (async)   │ │
│  │ - last_check│  │ - Update    │  │ - Toggle    │  │ - OpenUrl   │ │
│  │ - update_   │  │   modal     │  │   Enabled   │  │             │ │
│  │   config    │  │             │  │             │  │             │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                │                │                │        │
│         └────────────────┴────────────────┴────────────────┘        │
│                              │                                      │
│                    ┌─────────┴─────────┐                            │
│                    │   UpdateChecker   │                            │
│                    │   (background)    │                            │
│                    └─────────┬─────────┘                            │
│                              │                                      │
└──────────────────────────────┼──────────────────────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │  GitHub Releases API│
                    │  api.github.com     │
                    └─────────────────────┘
```

### 3.2 Elm 架构集成

#### Model 扩展
```rust
// src/app/model.rs

pub struct App {
    // ... 现有字段 ...

    /// Update check state
    pub update_state: UpdateState,
}

/// Update check state
#[derive(Debug, Clone)]
pub struct UpdateState {
    /// Whether an update is available
    pub update_available: bool,

    /// Latest version string (if available)
    pub latest_version: Option<String>,

    /// Current version
    pub current_version: String,

    /// Last check timestamp
    pub last_check: Option<chrono::DateTime<chrono::Local>>,

    /// Whether update modal is showing
    pub showing_update_modal: bool,

    /// Update configuration
    pub config: UpdateConfig,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            update_available: false,
            latest_version: None,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            last_check: None,
            showing_update_modal: false,
            config: UpdateConfig::default(),
        }
    }
}
```

#### Msg 扩展
```rust
// src/app/msg.rs

pub enum AppMsg {
    // ... 现有消息 ...

    // Update check messages
    /// Trigger update check
    CheckForUpdates,

    /// Update check completed
    UpdateCheckCompleted(Result<UpdateCheckResult, UpdateError>),

    /// Show update modal
    ShowUpdateModal,

    /// Dismiss update modal
    DismissUpdateModal,

    /// Open release page in browser
    OpenReleasePage,

    /// Toggle auto-update enabled
    ToggleAutoUpdate,
}

/// Update check result
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub latest_version: Option<String>,
    pub release_notes: Option<String>,
    pub release_url: String,
}
```

#### Cmd 扩展
```rust
// src/app/msg.rs (Cmd enum)

pub enum Cmd {
    // ... 现有命令 ...

    /// Check for updates
    CheckForUpdates(UpdateConfig),

    /// Open URL in browser
    OpenBrowser(String),
}
```

### 3.3 更新检测模块

```rust
// src/update/mod.rs

use std::time::Duration;
use tokio::time::interval;
use serde::Deserialize;

/// GitHub Release API response
#[derive(Debug, Clone, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: Option<String>,
    pub html_url: String,
    pub published_at: String,
}

/// Update checker
pub struct UpdateChecker {
    config: UpdateConfig,
    current_version: String,
}

impl UpdateChecker {
    pub fn new(config: UpdateConfig) -> Self {
        Self {
            config,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Check for updates once
    pub async fn check(&self) -> Result<UpdateCheckResult, UpdateError> {
        if !self.config.enabled {
            return Err(UpdateError::Disabled);
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.config.github.owner,
            self.config.github.repo
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("repotui-update-checker")
            .build()?;

        let release: GithubRelease = client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        let comparison = compare_versions(&self.current_version, &release.tag_name)?;

        Ok(UpdateCheckResult {
            update_available: matches!(comparison, VersionComparison::UpdateAvailable { .. }),
            latest_version: Some(release.tag_name),
            release_notes: release.body,
            release_url: release.html_url,
        })
    }

    /// Start background update checking
    pub async fn run_background(&self, tx: mpsc::Sender<AppMsg>) {
        // Initial delay
        tokio::time::sleep(Duration::from_secs(
            self.config.initial_delay_minutes * 60
        )).await;

        let mut interval = interval(Duration::from_secs(
            self.config.check_interval_hours * 3600
        ));

        loop {
            interval.tick().await;

            match self.check().await {
                Ok(result) => {
                    let _ = tx.send(AppMsg::UpdateCheckCompleted(Ok(result))).await;
                }
                Err(e) => {
                    // Log error but don't disturb user
                    tracing::warn!("Update check failed: {}", e);
                }
            }
        }
    }
}
```

### 3.4 依赖项

新增依赖（`Cargo.toml`）:

```toml
[dependencies]
# ... 现有依赖 ...

# HTTP client for update checking
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }

# Semantic version parsing
semver = "1.0"

# URL opening (cross-platform)
open = "5.0"
```

---

## 4. UI 设计

### 4.1 更新指示器

**位置**: 标题栏右侧
**触发条件**: 检测到有新版本可用
**样式**: 黄色/绿色文字，带向上箭头图标

```
╭─ ghclone v0.1.0 ──────────────────────────────────── [⬆ Update: v0.2.0] ─╮
```

**颜色方案**:
| 状态 | 颜色 | 说明 |
|------|------|------|
| 正常 | 灰色 | 无更新或检查中 |
| 有更新 | 黄色 | 新版本可用 |
| 严重更新 | 红色 | 安全更新（如包含安全修复） |

### 4.2 更新详情弹窗

**尺寸**: 60x16 (最小终端 80x24 内居中显示)
**快捷键**: `U` 打开，`Esc` 关闭
**内容**:
- 当前版本 vs 最新版本对比
- Release Notes 摘要（前 5 行）
- 操作按钮

### 4.3 设置界面

在设置菜单中添加更新检测选项：

```
╭─ Settings ────────────────────────────────────────────────╮
│                                                           │
│  [✓] Enable automatic update checks                       │
│      Check interval: 24 hours                            │
│      Last checked: 2026-03-09 14:30                      │
│                                                           │
│  [Check Now]                                              │
│                                                           │
╰───────────────────────────────────────────────────────────╯
```

---

## 5. 错误处理

### 5.1 错误类型

```rust
// src/update/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Update checking is disabled")]
    Disabled,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    #[error("API rate limit exceeded")]
    RateLimited,

    #[error("Repository not found")]
    RepoNotFound,

    #[error("Timeout")]
    Timeout,
}
```

### 5.2 错误处理策略

| 错误场景 | 用户通知 | 重试策略 |
|----------|----------|----------|
| 网络超时 | 静默失败，记录日志 | 下次检测周期重试 |
| API 限流 | 静默失败 | 1 小时后重试 |
| 仓库不存在 | 记录错误日志 | 不再重试（配置错误） |
| 版本解析失败 | 记录警告日志 | 下次检测周期重试 |

**原则**: 更新检测失败不打扰用户，仅记录日志。

---

## 6. 安全考虑

### 6.1 数据传输安全
- 使用 HTTPS 访问 GitHub API
- 不传输任何用户数据或敏感信息
- 仅发送基本的 User-Agent 标识

### 6.2 命令执行安全
```rust
/// 安全地打开浏览器
fn open_release_page(url: &str) -> Result<(), UpdateError> {
    // 验证 URL 格式
    let parsed = url::Url::parse(url)?;

    // 仅允许 https 协议
    if parsed.scheme() != "https" {
        return Err(UpdateError::InvalidUrl("Only HTTPS allowed".to_string()));
    }

    // 仅允许 github.com 域名
    if parsed.host_str() != Some("github.com") {
        return Err(UpdateError::InvalidUrl("Only github.com allowed".to_string()));
    }

    // 使用 open crate 安全打开
    open::that(url)?;
    Ok(())
}
```

### 6.3 隐私保护
- 不收集用户 ID、IP 或其他标识信息
- 不使用追踪 Cookie
- 遵守 GitHub API 使用规范

---

## 7. 测试策略

### 7.1 单元测试

| 模块 | 测试范围 | 目标覆盖率 |
|------|----------|------------|
| update/checker.rs | 版本比较、API 响应解析 | 90%+ |
| update/version.rs | 语义化版本解析 | 95%+ |
| config/types.rs | 配置序列化/反序列化 | 90%+ |

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(matches!(
            compare_versions("0.1.0", "0.2.0").unwrap(),
            VersionComparison::UpdateAvailable { .. }
        ));

        assert!(matches!(
            compare_versions("0.2.0", "0.2.0").unwrap(),
            VersionComparison::UpToDate
        ));

        assert!(matches!(
            compare_versions("0.3.0", "0.2.0").unwrap(),
            VersionComparison::Ahead
        ));
    }

    #[test]
    fn test_version_with_v_prefix() {
        assert!(matches!(
            compare_versions("v0.1.0", "v0.2.0").unwrap(),
            VersionComparison::UpdateAvailable { .. }
        ));
    }

    #[tokio::test]
    async fn test_check_disabled() {
        let config = UpdateConfig {
            enabled: false,
            ..Default::default()
        };
        let checker = UpdateChecker::new(config);

        let result = checker.check().await;
        assert!(matches!(result, Err(UpdateError::Disabled)));
    }
}
```

### 7.2 集成测试

| 测试场景 | 验证点 |
|----------|--------|
| 首次启动检测 | 延迟 5 分钟后触发检测 |
| 配置变更 | 禁用/启用切换生效 |
| 网络故障恢复 | 失败后下次周期正常检测 |
| UI 交互 | 弹窗显示/关闭/跳转正常 |

### 7.3 Mock 测试

```rust
/// Mock GitHub API for testing
#[cfg(test)]
mod mock_tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_check_with_mock_api() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/repotui/repotui/releases/latest"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "tag_name": "v0.2.0",
                    "name": "Release v0.2.0",
                    "body": "Test release notes",
                    "html_url": "https://github.com/repotui/repotui/releases/tag/v0.2.0"
                })))
            .mount(&mock_server)
            .await;

        // Test with mock server URL...
    }
}
```

---

## 8. 性能指标

### 8.1 量化指标

| 指标 | 目标 | 测量方法 |
|------|------|----------|
| 检测延迟 | < 500ms | API 往返时间 |
| 内存占用 | < 1MB | 更新模块 RSS |
| 启动影响 | 0ms | 后台异步执行 |
| 网络超时 | 10s | 超时配置 |

### 8.2 资源限制
- 单次 API 请求超时: 10 秒
- 后台任务优先级: 低（不影响主线程）
- 失败重试间隔: 1 小时（限流时）

---

## 9. 开发计划

### Phase 1: 核心功能 (2-3 天)
- [ ] 创建 `src/update/` 模块
- [ ] 实现版本比较逻辑
- [ ] 实现 GitHub API 客户端
- [ ] 添加配置类型和序列化
- [ ] 编写单元测试

### Phase 2: 架构集成 (2-3 天)
- [ ] 扩展 App Model 添加 UpdateState
- [ ] 添加 Update 相关 Msg 和 Cmd
- [ ] 实现 Update 逻辑到 update.rs
- [ ] 集成后台定时任务
- [ ] 编写集成测试

### Phase 3: UI 实现 (2-3 天)
- [ ] 标题栏更新指示器
- [ ] 更新详情弹窗
- [ ] 设置界面选项
- [ ] 快捷键绑定 (`U` 键)
- [ ] UI 渲染测试

### Phase 4: 测试与优化 (1-2 天)
- [ ] E2E 测试（模拟网络场景）
- [ ] 性能基准测试
- [ ] 错误场景测试
- [ ] 文档更新

---

## 10. 验收标准

### 功能验收
- [ ] 程序启动后 5 分钟自动检测更新
- [ ] 检测到新版本时标题栏显示更新提示
- [ ] 按 `U` 键可查看更新详情弹窗
- [ ] 弹窗中可点击打开 GitHub Release 页面
- [ ] 用户可在设置中禁用/启用自动检测
- [ ] 配置变更后下次启动生效

### 性能验收
- [ ] 更新检测不阻塞主线程
- [ ] API 请求超时后 gracefully 失败
- [ ] 内存占用增加 < 1MB

### 安全验收
- [ ] 仅使用 HTTPS 访问 GitHub
- [ ] URL 打开前验证域名白名单
- [ ] 不传输用户敏感信息

### 体验验收
- [ ] 更新检测失败不打扰用户
- [ ] UI 提示清晰可见但不突兀
- [ ] 支持快捷键操作
- [ ] 设置选项即时生效

---

## 11. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| GitHub API 限流 | 中 | 合理的检测频率 (24h)，失败时优雅降级 |
| 网络不可用 | 低 | 静默失败，记录日志，下次重试 |
| 版本格式不兼容 | 低 | 严格的 semver 解析，异常时记录日志 |
| 新增依赖体积 | 低 | 使用轻量级 HTTP 客户端，可选 feature |
| 用户隐私顾虑 | 中 | 文档明确说明不收集数据，可完全禁用 |

---

## 12. 附录

### 12.1 参考文档
- [GitHub Releases API](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)
- [Semantic Versioning](https://semver.org/)
- [reqwest 文档](https://docs.rs/reqwest)

### 12.2 相关文件
- 配置文件: `~/.config/repotui/config.toml`
- 版本定义: `Cargo.toml` 中的 `package.version`

### 12.3 变更日志
| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-03-10 | v1 | 初始版本创建 |
