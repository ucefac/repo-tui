# 自动检测更新功能实现总结

## 功能概述

实现了程序运行期间每天自动检测 GitHub 最新版本的功能。当有新版本可用时，在标题栏显示更新提示。

## 实现内容

### 1. 新增模块 (`src/update/`)

| 文件 | 职责 |
|------|------|
| `mod.rs` | 模块入口，导出公共接口 |
| `types.rs` | 核心类型定义（UpdateStatus, UpdateInfo, UpdateCheckResult） |
| `checker.rs` | GitHub API 调用和版本比较逻辑 |
| `config.rs` | 更新配置（UpdateConfig） |
| `scheduler.rs` | 定时调度器（每24小时检测一次） |

### 2. 架构集成

#### 新增依赖 (`Cargo.toml`)
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
semver = { version = "1.0", features = ["serde"] }
```

#### 扩展消息 (`AppMsg`)
- `TriggerUpdateCheck` - 触发更新检测
- `UpdateCheckCompleted` - 更新检测完成
- `DismissUpdateNotification` - 关闭更新提示
- `IgnoreUpdateVersion` - 忽略当前版本

#### 扩展命令 (`Cmd`)
- `CheckForUpdate` - 执行更新检测

#### 扩展配置 (`Config`)
- `update: UpdateConfig` - 自动更新配置

### 3. UI 更新

#### 标题栏更新提示
- 当有新版本可用时，在标题栏右侧显示 `⬆ v{x.x.x}`
- 黄色加粗样式，醒目但不突兀

#### 帮助面板
- 添加 `U` 键说明：手动触发更新检查

### 4. 键盘快捷键

| 按键 | 功能 |
|------|------|
| `U` | 手动触发更新检查 |

### 5. 配置选项

```toml
[update]
auto_check_enabled = true          # 是否启用自动检测
check_interval_hours = 24          # 检测间隔（小时）
ignored_version = null             # 忽略的版本
github_owner = "yyyyyyh"           # GitHub 仓库所有者
github_repo = "ghclone"            # GitHub 仓库名称
```

### 6. 测试覆盖

- `update::checker::tests` - 版本比较测试
- `update::config::tests` - 配置测试
- `update::scheduler::tests` - 调度器测试
- `ui::widgets::title_bar::tests` - UI 渲染测试

## 工作流程

1. **程序启动**
   - 启动更新调度器（如果启用）
   - 延迟 5 秒后首次检测

2. **定时检测**
   - 每 24 小时自动检测一次
   - 调用 GitHub API: `https://api.github.com/repos/{owner}/{repo}/releases/latest`

3. **版本比较**
   - 使用 semver 进行语义化版本比较
   - 支持 `v` 前缀（如 v0.1.0）
   - 正确处理预发布版本

4. **UI 提示**
   - 有新版本时标题栏显示 `⬆ v{x.x.x}`
   - 用户按 `U` 键可手动触发检测

5. **错误处理**
   - 网络错误：记录失败状态
   - API 限流：显示 RateLimitExceeded 错误
   - 无发布：显示 NoReleasesFound 错误

## 安全考虑

- 使用 HTTPS 强制加密
- 30 秒超时设置
- 使用 User-Agent 标识应用
- 正确处理 GitHub API 限流 (403)

## 性能指标

- 首次检测延迟：5 秒
- 检测间隔：24 小时
- API 请求超时：30 秒
- 版本解析：本地完成，无网络开销

## 文件变更列表

### 新增文件
- `src/update/mod.rs`
- `src/update/types.rs`
- `src/update/checker.rs`
- `src/update/config.rs`
- `src/update/scheduler.rs`

### 修改文件
- `Cargo.toml` - 添加 reqwest, semver 依赖
- `src/lib.rs` - 添加 update 模块，启动调度器
- `src/error.rs` - 添加 UpdateError
- `src/app/msg.rs` - 添加更新相关消息
- `src/app/model.rs` - 添加更新状态字段
- `src/app/update.rs` - 处理更新消息
- `src/config/types.rs` - 添加 UpdateConfig
- `src/runtime/executor.rs` - 处理 CheckForUpdate 命令
- `src/handler/keyboard.rs` - 添加 'U' 键处理
- `src/ui/render.rs` - 传递 update_status
- `src/ui/widgets/title_bar.rs` - 显示更新提示
- `src/ui/widgets/help_panel.rs` - 添加帮助说明
- `tests/helpers/mod.rs` - 修复测试配置
- `tests/integration_clone.rs` - 修复测试配置

## 使用说明

### 启用自动更新检测
配置文件 `~/.config/repotui/config.toml`:
```toml
[update]
auto_check_enabled = true
check_interval_hours = 24
```

### 手动检查更新
在程序运行状态下按 `U` 键。

### 忽略特定版本
（可在配置中设置）
```toml
[update]
ignored_version = "v0.2.0"
```

## 后续可扩展

1. 更新详情弹窗（显示 release notes）
2. 一键打开浏览器到 release 页面
3. 下载进度显示
4. 自动下载和安装更新

---

**实现日期**: 2026-03-10
**分支**: feature/auto-update
**Worktree**: `.worktrees/auto-update-feature`
