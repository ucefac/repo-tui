# 自动检测更新功能测试计划

**文档版本**: 1.0
**创建日期**: 2026-03-10
**测试角色**: Tester
**关联功能**: 自动检测更新 (Auto-update Check)

---

## 1. 概述

### 1.1 测试目标

验证 repotui 应用程序的自动更新检测功能，确保：
- 版本比较逻辑正确
- GitHub API 集成稳定可靠
- 定时检测机制按预期工作
- UI 更新提示友好且可交互
- 配置选项生效

### 1.2 测试范围

| 组件 | 测试类型 | 优先级 |
|------|----------|--------|
| 版本比较逻辑 | 单元测试 | P0 |
| GitHub API 客户端 | 单元测试 + 集成测试 | P0 |
| 定时检测机制 | 单元测试 + 集成测试 | P0 |
| 配置管理 | 单元测试 | P1 |
| UI 渲染 (更新提示) | 单元测试 + UI 渲染测试 | P1 |
| 用户交互流程 | E2E 测试 | P2 |
| 完整功能流程 | E2E 测试 | P2 |

### 1.3 测试环境

- **OS**: macOS, Linux, Windows (GitHub Actions)
- **Rust版本**: 1.75+
- **网络**: 需要互联网连接 (GitHub API)
- **测试工具**: `cargo test`, `mockall`, `wiremock` (推荐)

---

## 2. 测试策略

### 2.1 测试金字塔

```
         E2E (3-5 场景)
        /  集成 (15-25 用例)
       /    单元 (50+ 用例，覆盖率≥90%)
```

### 2.2 Mock 策略

| 依赖 | Mock 方案 | 说明 |
|------|-----------|------|
| GitHub API | `wiremock` / `mockito` | HTTP 请求拦截 |
| 当前时间 | `mock_instant` / 手动注入 | 测试时间边界 |
| 配置文件 | `tempfile` + 内存存储 | 隔离测试环境 |
| HTTP Client | Trait 抽象 + Mock 实现 | 接口替换 |

---

## 3. 测试用例详细设计

### 3.1 版本比较逻辑测试 (单元测试)

#### 3.1.1 标准版本比较

```rust
#[test]
fn test_version_comparison_current_less_than_latest() {
    let current = Version::parse("0.1.0").unwrap();
    let latest = Version::parse("0.2.0").unwrap();
    assert!(current < latest);
    assert!(VersionChecker::needs_update(&current, &latest));
}

#[test]
fn test_version_comparison_current_equals_latest() {
    let current = Version::parse("1.0.0").unwrap();
    let latest = Version::parse("1.0.0").unwrap();
    assert!(!VersionChecker::needs_update(&current, &latest));
}

#[test]
fn test_version_comparison_current_greater_than_latest() {
    // 开发版本情况
    let current = Version::parse("1.1.0").unwrap();
    let latest = Version::parse("1.0.0").unwrap();
    assert!(!VersionChecker::needs_update(&current, &latest));
}
```

**测试数据**:

| 当前版本 | 最新版本 | 期望结果 |
|----------|----------|----------|
| 0.1.0 | 0.2.0 | 需要更新 |
| 1.0.0 | 1.0.0 | 无需更新 |
| 1.1.0 | 1.0.0 | 无需更新 (开发版本) |
| 0.9.9 | 1.0.0 | 需要更新 (大版本) |
| 1.0.0-alpha | 1.0.0 | 需要更新 |
| 2.0.0-beta | 2.0.0 | 需要更新 |

#### 3.1.2 预发布版本处理

```rust
#[test]
fn test_prerelease_version_handling() {
    // semver 预发布标签比较
    let cases = vec![
        ("1.0.0-alpha", "1.0.0", true),    // alpha < stable
        ("1.0.0-beta", "1.0.0", true),     // beta < stable
        ("1.0.0-rc.1", "1.0.0", true),     // rc < stable
        ("1.0.0-alpha", "1.0.0-beta", true), // alpha < beta
        ("1.0.0-beta.1", "1.0.0-beta.2", true), // beta.1 < beta.2
        ("1.0.0", "1.0.0-alpha", false),   // stable > alpha
    ];

    for (current, latest, needs_update) in cases {
        let current_v = Version::parse(current).unwrap();
        let latest_v = Version::parse(latest).unwrap();
        assert_eq!(
            VersionChecker::needs_update(&current_v, &latest_v),
            needs_update,
            "Failed for {} vs {}", current, latest
        );
    }
}
```

#### 3.1.3 无效版本处理

```rust
#[test]
fn test_invalid_version_parsing() {
    let invalid_versions = vec![
        "",
        "not-a-version",
        "1.2",
        "v1.0.0",  // 需要处理 'v' 前缀
        "1.0.0.0.0",
        "latest",
        "*",
    ];

    for v in invalid_versions {
        let result = Version::parse(v);
        assert!(result.is_err() || v == "v1.0.0", "Should handle '{}' gracefully", v);
    }
}
```

### 3.2 GitHub API 集成测试

#### 3.2.1 API 请求成功场景

```rust
#[tokio::test]
async fn test_github_api_success() {
    // 使用 wiremock 启动 mock 服务器
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/repotui/repotui/releases/latest"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "tag_name": "v1.0.0",
                "name": "Release 1.0.0",
                "body": "Release notes...",
                "html_url": "https://github.com/repotui/repotui/releases/tag/v1.0.0",
                "published_at": "2026-03-10T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;

    let client = GithubApiClient::new(&mock_server.uri());
    let result = client.fetch_latest_release().await;

    assert!(result.is_ok());
    let release = result.unwrap();
    assert_eq!(release.version.to_string(), "1.0.0");
    assert_eq!(release.url, "https://github.com/repotui/repotui/releases/tag/v1.0.0");
}
```

#### 3.2.2 API 错误处理

```rust
#[tokio::test]
async fn test_github_api_rate_limited() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403)
            .set_body_json(json!({
                "message": "API rate limit exceeded",
                "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
            })))
        .mount(&mock_server)
        .await;

    let client = GithubApiClient::new(&mock_server.uri());
    let result = client.fetch_latest_release().await;

    assert!(matches!(result, Err(UpdateError::RateLimited { .. })));
}

#[tokio::test]
async fn test_github_api_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_delay(Duration::from_secs(35))) // 超过默认超时
        .mount(&mock_server)
        .await;

    let client = GithubApiClient::new(&mock_server.uri())
        .with_timeout(Duration::from_secs(5));
    let result = client.fetch_latest_release().await;

    assert!(matches!(result, Err(UpdateError::Timeout)));
}

#[tokio::test]
async fn test_github_api_network_error() {
    let client = GithubApiClient::new("http://invalid-url-that-will-fail");
    let result = client.fetch_latest_release().await;

    assert!(matches!(result, Err(UpdateError::Network(_))));
}

#[tokio::test]
async fn test_github_api_invalid_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let client = GithubApiClient::new(&mock_server.uri());
    let result = client.fetch_latest_release().await;

    assert!(matches!(result, Err(UpdateError::InvalidResponse(_))));
}

#[tokio::test]
async fn test_github_api_missing_tag_name() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "name": "Release without tag",
                // missing tag_name
            })))
        .mount(&mock_server)
        .await;

    let client = GithubApiClient::new(&mock_server.uri());
    let result = client.fetch_latest_release().await;

    assert!(matches!(result, Err(UpdateError::InvalidResponse(_))));
}
```

### 3.3 定时检测机制测试

#### 3.3.1 每天触发一次

```rust
#[tokio::test]
async fn test_daily_check_trigger() {
    let mut scheduler = UpdateScheduler::new();
    let now = Instant::now();

    // 模拟上次检查是 23 小时前
    scheduler.set_last_check(Some(now - Duration::from_secs(23 * 3600)));
    assert!(!scheduler.should_check());

    // 模拟上次检查是 25 小时前
    scheduler.set_last_check(Some(now - Duration::from_secs(25 * 3600)));
    assert!(scheduler.should_check());
}

#[tokio::test]
async fn test_daily_check_exact_boundary() {
    let mut scheduler = UpdateScheduler::new()
        .with_check_interval(Duration::from_secs(24 * 3600));

    let now = Instant::now();

    // 正好 24 小时
    scheduler.set_last_check(Some(now - Duration::from_secs(24 * 3600)));
    assert!(scheduler.should_check());

    // 24 小时减 1 秒
    scheduler.set_last_check(Some(now - Duration::from_secs(24 * 3600 - 1)));
    assert!(!scheduler.should_check());
}
```

#### 3.3.2 程序重启后恢复计时

```rust
#[test]
fn test_persist_last_check_time() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("update_state.json");

    // 创建状态并保存
    let check_time = Utc::now() - Duration::from_secs(12 * 3600); // 12 小时前
    let state = UpdateState {
        last_check: Some(check_time),
        last_version: Some("0.9.0".to_string()),
        skipped_version: None,
    };
    state.save(&state_file).unwrap();

    // 从文件恢复
    let restored = UpdateState::load(&state_file).unwrap();
    assert_eq!(restored.last_check, Some(check_time));

    // 验证剩余等待时间
    let scheduler = UpdateScheduler::from_state(restored);
    assert!(!scheduler.should_check()); // 还需要等待 12 小时
}

#[test]
fn test_state_file_corruption_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let state_file = temp_dir.path().join("update_state.json");

    // 写入损坏的 JSON
    fs::write(&state_file, "{invalid json").unwrap();

    // 应该能优雅恢复
    let result = UpdateState::load(&state_file);
    assert!(result.is_ok());
    assert!(result.unwrap().last_check.is_none());
}
```

#### 3.3.3 配置禁用检测

```rust
#[test]
fn test_check_disabled_in_config() {
    let config = UpdateConfig {
        enabled: false,
        check_interval_hours: 24,
    };

    let scheduler = UpdateScheduler::new()
        .with_config(config);

    // 即使很长时间未检查，也不应该检查
    scheduler.set_last_check(Some(Instant::now() - Duration::from_secs(30 * 24 * 3600)));
    assert!(!scheduler.should_check());
}

#[test]
fn test_check_enabled_by_default() {
    let config = UpdateConfig::default();
    assert!(config.enabled);
}
```

### 3.4 配置管理测试

```rust
#[test]
fn test_update_config_serialization() {
    let config = UpdateConfig {
        enabled: true,
        check_interval_hours: 48,
    };

    let toml = toml::to_string(&config).unwrap();
    assert!(toml.contains("enabled = true"));
    assert!(toml.contains("check_interval_hours = 48"));

    let deserialized: UpdateConfig = toml::from_str(&toml).unwrap();
    assert_eq!(deserialized.enabled, true);
    assert_eq!(deserialized.check_interval_hours, 48);
}

#[test]
fn test_update_config_defaults() {
    let config: UpdateConfig = toml::from_str("").unwrap_or_default();
    assert!(config.enabled);
    assert_eq!(config.check_interval_hours, 24);
}

#[test]
fn test_config_integration_with_main_config() {
    let config_toml = r#"
version = "2.0"

[update]
enabled = true
check_interval_hours = 12

[ui]
theme = "dark"
"#;

    let config: Config = toml::from_str(config_toml).unwrap();
    assert!(config.update.enabled);
    assert_eq!(config.update.check_interval_hours, 12);
}
```

### 3.5 UI 渲染测试

#### 3.5.1 更新提示组件

```rust
#[test]
fn test_update_notification_render() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();

    let notification = UpdateNotification::new("1.0.0")
        .with_url("https://github.com/repotui/repotui/releases");

    terminal.draw(|f| {
        f.render_widget(notification, f.area());
    }).unwrap();

    // 验证渲染输出
    let buffer = terminal.backend().buffer().clone();
    // 检查是否包含版本信息
}

#[test]
fn test_update_notification_in_status_bar() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();

    let status_bar = StatusBar::new("Update available: v1.0.0", &theme)
        .update_available(true);

    terminal.draw(|f| {
        f.render_widget(status_bar, f.area());
    }).unwrap();

    // 验证状态栏显示更新提示样式
}
```

#### 3.5.2 更新对话框

```rust
#[test]
fn test_update_dialog_render() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();

    let dialog = UpdateDialog::new("1.0.0", "0.9.0")
        .with_release_notes("New features:\n- Auto update check\n- Bug fixes");

    terminal.draw(|f| {
        let area = centered_rect(60, 12, f.area());
        f.render_widget(dialog, area);
    }).unwrap();
}
```

### 3.6 E2E 测试

```rust
#[tokio::test]
async fn test_full_update_check_flow() {
    // 启动 mock GitHub API
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "tag_name": "v1.0.0",
                "html_url": "https://github.com/repotui/repotui/releases/tag/v1.0.0"
            })))
        .mount(&mock_server)
        .await;

    // 创建测试应用实例
    let (tx, mut rx) = mpsc::channel(100);
    let mut app = App::new(tx);

    // 设置当前版本为 0.9.0
    app.set_current_version("0.9.0");

    // 触发更新检查
    app.check_for_updates(&mock_server.uri()).await;

    // 验证消息队列
    let msg = rx.recv().await;
    assert!(matches!(msg, Some(AppMsg::UpdateAvailable { .. })));
}

#[tokio::test]
async fn test_user_dismiss_update() {
    let mut app = create_test_app().await;

    // 模拟用户选择忽略更新
    app.handle_message(AppMsg::UpdateDismissed {
        version: "1.0.0".to_string(),
        remind_later: false,
    });

    // 验证状态
    assert_eq!(app.update_state.skipped_version, Some("1.0.0".to_string()));

    // 再次检查，不应提示
    app.check_for_updates().await;
    assert!(app.update_notification.is_none());
}
```

---

## 4. 测试数据和 Fixtures

### 4.1 版本测试数据

```rust
// tests/fixtures/versions.rs
pub const VERSION_TEST_CASES: &[(str, str, bool)] = &[
    // (current, latest, needs_update)
    ("0.1.0", "0.2.0", true),
    ("1.0.0", "1.0.0", false),
    ("1.1.0", "1.0.0", false),
    ("0.9.9", "1.0.0", true),
    ("1.0.0-alpha", "1.0.0", true),
    ("1.0.0-beta", "1.0.0", true),
    ("1.0.0-rc.1", "1.0.0", true),
    ("1.0.0-alpha", "1.0.0-beta", true),
    ("2.0.0", "1.9.9", false),
];
```

### 4.2 GitHub API Response Fixtures

```rust
// tests/fixtures/github_responses.rs
pub fn latest_release_response(version: &str) -> serde_json::Value {
    json!({
        "tag_name": format!("v{}", version),
        "name": format!("Release {}", version),
        "body": "Test release notes",
        "html_url": format!("https://github.com/repotui/repotui/releases/tag/v{}", version),
        "published_at": "2026-03-10T00:00:00Z",
        "prerelease": false
    })
}

pub fn rate_limit_response() -> serde_json::Value {
    json!({
        "message": "API rate limit exceeded for 127.0.0.1",
        "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
    })
}
```

---

## 5. 测试实现步骤

### 5.1 第一阶段：基础单元测试

1. **版本比较模块** (`src/update/version.rs`)
   - [ ] 实现 `Version` 类型包装
   - [ ] 添加比较逻辑测试
   - [ ] 添加预发布版本处理测试

2. **配置模块扩展** (`src/config/types.rs`)
   - [ ] 添加 `UpdateConfig` 结构体
   - [ ] 添加序列化/反序列化测试

### 5.2 第二阶段：API 客户端测试

1. **GitHub API 客户端** (`src/update/github.rs`)
   - [ ] 定义 API 接口 trait
   - [ ] 实现 mock 客户端
   - [ ] 添加所有错误场景测试

### 5.3 第三阶段：定时机制测试

1. **调度器** (`src/update/scheduler.rs`)
   - [ ] 实现时间检查逻辑
   - [ ] 添加状态持久化测试
   - [ ] 添加边界条件测试

### 5.4 第四阶段：UI 测试

1. **更新提示组件** (`src/ui/widgets/update_notification.rs`)
   - [ ] 实现组件
   - [ ] 添加渲染测试
   - [ ] 添加交互测试

### 5.5 第五阶段：集成和 E2E 测试

1. **完整流程测试** (`tests/integration_update_check.rs`)
2. **E2E 场景测试** (`tests/e2e_update_flow.rs`)

---

## 6. 预期测试结果

### 6.1 覆盖率目标

| 模块 | 目标覆盖率 | 最小覆盖率 |
|------|-----------|-----------|
| version.rs | 100% | 95% |
| github.rs | 95% | 85% |
| scheduler.rs | 95% | 85% |
| update_notification.rs | 90% | 80% |

### 6.2 性能指标

| 指标 | 目标值 |
|------|--------|
| 版本比较 | < 1ms |
| API 请求超时 | 5-10s |
| UI 渲染延迟 | < 16ms |
| 状态保存/加载 | < 10ms |

---

## 7. 风险评估与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| GitHub API 不稳定 | 中 | 高 | 实现重试机制和优雅降级 |
| 版本格式不一致 | 低 | 高 | 严格遵循 semver，添加解析容错 |
| 时区问题 | 中 | 中 | 统一使用 UTC 时间存储 |
| 网络代理环境 | 中 | 中 | 支持 HTTP_PROXY 环境变量 |

---

## 8. 相关文档

- [CLAUDE.md](../../CLAUDE.md) - 项目开发规范
- [docs/task/index.md](./index.md) - 任务文档索引
- [GitHub API 文档](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)
- [Semantic Versioning 规范](https://semver.org/)

---

**测试计划审批**:
**审批日期**:
**测试负责人**: Tester Role
