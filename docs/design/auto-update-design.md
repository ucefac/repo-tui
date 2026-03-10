# 自动检测更新功能技术设计文档

**文档版本**: 1.0
**创建日期**: 2026-03-10
**状态**: 待实施
**设计负责人**: Backend Dev

---

## 1. 设计概述

### 1.1 功能目标

实现程序运行期间每天自动检测 GitHub 最新版本，并在发现新版本时向用户显示更新提示。

### 1.2 核心需求

- 程序启动时进行一次更新检测
- 之后每 24 小时自动检测一次
- 使用 GitHub API 获取最新 release 信息
- 支持版本号比较（semver）
- 非侵入式提示，不影响用户正常使用
- 支持用户关闭自动检测功能

### 1.3 架构原则

遵循现有 Elm 架构模式：
- 副作用通过 `Cmd` 封装，在 `Runtime` 中执行
- 结果通过 `AppMsg` 传递，在 `update` 中处理状态变更
- UI 在 `render` 中根据状态渲染

---

## 2. 模块结构设计

### 2.1 模块位置

```
src/
├── update/
│   ├── mod.rs          # 模块入口，导出公共接口
│   ├── types.rs        # 核心类型定义（UpdateInfo, UpdateStatus 等）
│   ├── checker.rs      # 更新检测逻辑（GitHub API 调用、版本比较）
│   ├── config.rs       # 更新配置（是否启用、检测间隔等）
│   └── scheduler.rs    # 定时调度器（基于 tokio::time::interval）
```

### 2.2 文件职责

| 文件 | 职责 |
|------|------|
| `mod.rs` | 模块入口，导出公共类型和函数；实现 `check_for_update()` 主入口 |
| `types.rs` | 定义 `UpdateInfo`、`UpdateStatus`、`Version` 等核心类型 |
| `checker.rs` | 实现 GitHub API 调用、版本号解析和比较逻辑 |
| `config.rs` | 更新相关配置，支持持久化到 config.toml |
| `scheduler.rs` | 定时任务调度器，管理检测间隔和触发时机 |

---

## 3. 核心类型设计

### 3.1 更新信息类型

```rust
// src/update/types.rs

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 更新检测状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    /// 从未检测
    NeverChecked,
    /// 正在检测
    Checking,
    /// 已是最新版本
    UpToDate,
    /// 有新版本可用
    UpdateAvailable { version: String },
    /// 检测失败（网络错误等）
    CheckFailed { error: String },
}

/// 更新信息
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    /// 最新版本号
    pub version: String,
    /// 发布页面 URL
    pub html_url: String,
    /// 发布时间
    pub published_at: String,
    /// 发布说明
    pub body: Option<String>,
}

/// 版本比较结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// 当前版本较新（或相同）
    CurrentIsNewerOrEqual,
    /// 有新版本可用
    UpdateAvailable,
    /// 无法比较（格式错误）
    Incomparable,
}

/// 更新检测结果
#[derive(Debug, Clone)]
pub struct UpdateCheckResult {
    pub status: UpdateStatus,
    pub info: Option<UpdateInfo>,
    pub checked_at: SystemTime,
}
```

### 3.2 更新配置类型

```rust
// src/update/config.rs

use serde::{Deserialize, Serialize};

/// 自动更新配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// 是否启用自动检测
    #[serde(default = "default_true")]
    pub auto_check_enabled: bool,
    /// 检测间隔（小时），默认 24 小时
    #[serde(default = "default_check_interval_hours")]
    pub check_interval_hours: u64,
    /// 忽略的版本号（用户选择跳过此版本）
    #[serde(default)]
    pub ignored_version: Option<String>,
    /// 最后一次检测时间
    #[serde(default)]
    pub last_checked_at: Option<chrono::DateTime<chrono::Local>>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check_enabled: true,
            check_interval_hours: 24,
            ignored_version: None,
            last_checked_at: None,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_check_interval_hours() -> u64 {
    24
}
```

---

## 4. 与现有架构集成方案

### 4.1 扩展 AppMsg（消息定义）

```rust
// src/app/msg.rs - 新增消息

/// Commands for async execution
#[derive(Debug, Clone)]
pub enum Cmd {
    // ... 现有 Cmd 变体 ...

    /// 检查更新
    CheckForUpdate,
}

/// Application messages
#[derive(Debug, Clone)]
pub enum AppMsg {
    // ... 现有 AppMsg 变体 ...

    // === Update Operations ===
    /// 触发更新检测
    TriggerUpdateCheck,

    /// 更新检测完成
    UpdateCheckCompleted(Box<Result<crate::update::UpdateCheckResult, crate::error::UpdateError>>),

    /// 关闭更新提示
    DismissUpdateNotification,

    /// 忽略当前可用版本
    IgnoreUpdateVersion(String),
}
```

### 4.2 扩展 App Model（应用状态）

```rust
// src/app/model.rs - App 结构体新增字段

pub struct App {
    // ... 现有字段 ...

    /// 更新检测状态
    pub update_status: crate::update::UpdateStatus,

    /// 可用更新信息
    pub available_update: Option<crate::update::UpdateInfo>,

    /// 更新提示是否已关闭（当前会话）
    pub update_notification_dismissed: bool,
}
```

### 4.3 扩展 Runtime（命令执行）

```rust
// src/runtime/executor.rs - dispatch 方法新增处理

impl Runtime {
    pub fn dispatch(&self, cmd: Cmd) {
        let msg_tx = self.msg_tx.clone();

        match cmd {
            // ... 现有 Cmd 处理 ...

            Cmd::CheckForUpdate => {
                tokio::spawn(async move {
                    let result = crate::update::check_for_update().await;
                    let _ = msg_tx
                        .send(AppMsg::UpdateCheckCompleted(Box::new(result)))
                        .await;
                });
            }
        }
    }
}
```

### 4.4 扩展 Update 逻辑

```rust
// src/app/update.rs - 新增消息处理

pub fn update(msg: AppMsg, app: &mut App, runtime: &Runtime) {
    match msg {
        // ... 现有消息处理 ...

        AppMsg::TriggerUpdateCheck => {
            app.update_status = crate::update::UpdateStatus::Checking;
            runtime.dispatch(Cmd::CheckForUpdate);
        }

        AppMsg::UpdateCheckCompleted(result) => {
            match *result {
                Ok(check_result) => {
                    app.update_status = check_result.status.clone();
                    if let crate::update::UpdateStatus::UpdateAvailable { .. } = check_result.status {
                        app.available_update = check_result.info;
                        app.update_notification_dismissed = false;
                    }
                }
                Err(e) => {
                    app.update_status = crate::update::UpdateStatus::CheckFailed {
                        error: e.to_string(),
                    };
                }
            }
        }

        AppMsg::DismissUpdateNotification => {
            app.update_notification_dismissed = true;
        }

        AppMsg::IgnoreUpdateVersion(version) => {
            app.update_notification_dismissed = true;
            // 持久化到配置
            if let Some(ref mut config) = app.config {
                config.update.ignored_version = Some(version);
                // 触发配置保存
                runtime.dispatch(Cmd::SaveConfig(config.clone()));
            }
        }
    }
}
```

### 4.5 扩展错误类型

```rust
// src/error.rs - 新增错误类型

/// Update check errors
#[derive(Error, Debug, Clone)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Version parse error: {0}")]
    VersionParseError(String),

    #[error("No releases found")]
    NoReleasesFound,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
```

---

## 5. 定时调度器设计

### 5.1 调度器结构

```rust
// src/update/scheduler.rs

use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Interval};

/// 更新检测调度器
pub struct UpdateScheduler {
    /// 消息发送器
    msg_tx: mpsc::Sender<crate::app::msg::AppMsg>,
    /// 定时器间隔
    interval: Interval,
    /// 是否启用
    enabled: bool,
}

impl UpdateScheduler {
    /// 创建新的调度器
    pub fn new(
        msg_tx: mpsc::Sender<crate::app::msg::AppMsg>,
        interval_hours: u64,
    ) -> Self {
        let duration = Duration::from_secs(interval_hours * 3600);
        Self {
            msg_tx,
            interval: interval(duration),
            enabled: true,
        }
    }

    /// 启动调度循环
    pub async fn run(mut self) {
        if !self.enabled {
            return;
        }

        // 首次检测在启动后立即执行（短暂延迟避免阻塞启动）
        tokio::time::sleep(Duration::from_secs(5)).await;
        let _ = self
            .msg_tx
            .send(crate::app::msg::AppMsg::TriggerUpdateCheck)
            .await;

        // 后续每间隔周期检测
        loop {
            self.interval.tick().await;
            if self.enabled {
                let _ = self
                    .msg_tx
                    .send(crate::app::msg::AppMsg::TriggerUpdateCheck)
                    .await;
            }
        }
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
```

### 5.2 集成到主循环

```rust
// src/lib.rs - run_app 函数中启动调度器

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    // ... 现有初始化代码 ...

    // 启动更新检测调度器（如果启用）
    let update_config = app
        .config
        .as_ref()
        .map(|c| c.update.clone())
        .unwrap_or_default();

    if update_config.auto_check_enabled {
        let scheduler = crate::update::UpdateScheduler::new(
            msg_tx.clone(),
            update_config.check_interval_hours,
        );
        tokio::spawn(scheduler.run());
    }

    // ... 主循环 ...
}
```

---

## 6. GitHub API 集成

### 6.1 API 调用实现

```rust
// src/update/checker.rs

use reqwest;
use semver::Version;

const GITHUB_API_URL: &str = "https://api.github.com/repos/{owner}/{repo}/releases/latest";
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// 从 GitHub 获取最新 release 信息
pub async fn fetch_latest_release() -> Result<UpdateInfo, UpdateError> {
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    let url = GITHUB_API_URL
        .replace("{owner}", "repotui")
        .replace("{repo}", "repotui");

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| UpdateError::Network(e.to_string()))?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let release: UpdateInfo = response
                .json()
                .await
                .map_err(|e| UpdateError::ApiError(e.to_string()))?;
            Ok(release)
        }
        reqwest::StatusCode::FORBIDDEN => {
            // GitHub API rate limit
            Err(UpdateError::RateLimitExceeded)
        }
        reqwest::StatusCode::NOT_FOUND => {
            Err(UpdateError::NoReleasesFound)
        }
        status => {
            Err(UpdateError::ApiError(format!("HTTP {}", status)))
        }
    }
}

/// 比较版本号
pub fn compare_versions(current: &str, latest: &str) -> VersionComparison {
    let current_ver = match Version::parse(current.trim_start_matches('v')) {
        Ok(v) => v,
        Err(_) => return VersionComparison::Incomparable,
    };

    let latest_ver = match Version::parse(latest.trim_start_matches('v')) {
        Ok(v) => v,
        Err(_) => return VersionComparison::Incomparable,
    };

    if latest_ver > current_ver {
        VersionComparison::UpdateAvailable
    } else {
        VersionComparison::CurrentIsNewerOrEqual
    }
}

/// 主检测入口
pub async fn check_for_update() -> Result<UpdateCheckResult, UpdateError> {
    let release = fetch_latest_release().await?;
    let current_version = env!("CARGO_PKG_VERSION");

    let comparison = compare_versions(current_version, &release.version);

    let status = match comparison {
        VersionComparison::UpdateAvailable => {
            UpdateStatus::UpdateAvailable {
                version: release.version.clone(),
            }
        }
        _ => UpdateStatus::UpToDate,
    };

    Ok(UpdateCheckResult {
        status,
        info: Some(release),
        checked_at: SystemTime::now(),
    })
}
```

---

## 7. 配置集成

### 7.1 更新 Config 类型

```rust
// src/config/types.rs - Config 结构体新增字段

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... 现有字段 ...

    /// 自动更新配置
    #[serde(default)]
    pub update: crate::update::UpdateConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... 现有默认值 ...
            update: crate::update::UpdateConfig::default(),
        }
    }
}
```

---

## 8. UI 渲染方案

### 8.1 更新提示显示

更新提示将以非侵入式方式显示在状态栏或顶部标题栏区域：

```
┌─────────────────────────────────────────────────────────────────┐
│  repotui  v0.1.0                              [更新可用: v0.2.0]  │  ← 标题栏显示更新提示
├─────────────────────────────────────────────────────────────────┤
│  Search: [                                      ] Ctrl+S        │
├─────────────────────────────────────────────────────────────────┤
│  > repo1                    [main]  ✓  3 days ago               │
│    repo2                    [dev]   ✗  1 hour ago               │
│    repo3                    [feat]  ✓  2 weeks ago              │
├─────────────────────────────────────────────────────────────────┤
│  ↑↓ 导航  Enter 打开  ? 帮助  3 repos                           │  ← 状态栏显示快捷操作
└─────────────────────────────────────────────────────────────────┘
```

### 8.2 详细更新弹窗

用户按特定键（如 `u`）可打开更新详情弹窗：

```
┌───────────────────────────────────────────────────┐
│  新版本可用                                        │
├───────────────────────────────────────────────────┤
│                                                   │
│  当前版本: v0.1.0                                 │
│  最新版本: v0.2.0                                 │
│  发布时间: 2026-03-09                             │
│                                                   │
│  更新说明:                                        │
│  - 新增自动更新检测功能                           │
│  - 修复主题切换 bug                               │
│  - 优化大仓库加载性能                             │
│                                                   │
├───────────────────────────────────────────────────┤
│  [打开发布页面]  [忽略此版本]  [关闭]             │
└───────────────────────────────────────────────────┘
```

---

## 9. 依赖添加

### 9.1 Cargo.toml 修改

```toml
[dependencies]
# ... 现有依赖 ...

# HTTP client for update checking
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }

# Semantic versioning for version comparison
semver = { version = "1.0", features = ["serde"] }
```

### 9.2 依赖说明

| Crate | 版本 | 用途 |
|-------|------|------|
| reqwest | 0.12 | HTTP 客户端，用于调用 GitHub API |
| semver | 1.0 | 语义化版本解析和比较 |

选择 `rustls-tls` 而非 `native-tls` 以避免系统依赖，使打包更简单。

---

## 10. 测试策略

### 10.1 单元测试

```rust
// src/update/checker.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(
            compare_versions("0.1.0", "0.2.0"),
            VersionComparison::UpdateAvailable
        );
        assert_eq!(
            compare_versions("0.2.0", "0.1.0"),
            VersionComparison::CurrentIsNewerOrEqual
        );
        assert_eq!(
            compare_versions("0.1.0", "0.1.0"),
            VersionComparison::CurrentIsNewerOrEqual
        );
        assert_eq!(
            compare_versions("v0.1.0", "v0.2.0"),
            VersionComparison::UpdateAvailable
        );
    }

    #[test]
    fn test_compare_versions_prerelease() {
        assert_eq!(
            compare_versions("0.1.0-alpha", "0.1.0"),
            VersionComparison::UpdateAvailable
        );
    }
}
```

### 10.2 集成测试

- 模拟 GitHub API 响应测试检测流程
- 测试配置持久化和加载
- 测试调度器定时触发逻辑

---

## 11. 安全考虑

1. **HTTPS 强制**: 所有 API 请求必须使用 HTTPS
2. **超时设置**: API 调用设置 30 秒超时，避免阻塞
3. **User-Agent**: 使用应用名称和版本作为 User-Agent
4. **Rate Limit 处理**: 正确处理 403 状态码，提示用户稍后重试
5. **版本号验证**: 严格验证解析的版本号格式

---

## 12. 实现阶段

| 阶段 | 任务 | 优先级 |
|------|------|--------|
| Phase 1 | 创建 `src/update/` 模块结构，实现类型定义 | 高 |
| Phase 2 | 实现 GitHub API 调用和版本比较 | 高 |
| Phase 3 | 集成到 `Runtime` 和 `AppMsg` | 高 |
| Phase 4 | 实现调度器和配置持久化 | 中 |
| Phase 5 | UI 渲染实现（状态栏提示） | 中 |
| Phase 6 | 编写测试 | 中 |

---

**最后更新**: 2026-03-10
**维护者**: Backend Dev
