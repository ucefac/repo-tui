# 开发指南

本文档包含 repotui 项目的开发清单和常见问题解答。

---

## 🚀 开发清单

### Phase 0 收尾 (已完成 ✅)

- [x] 修复配置空路径验证 Bug (见 BUGFIX_EMPTY_PATH.md)
- [x] ~~修复 action/validators.rs (8 处)~~ (代码已正确，无需修复)
- [x] ~~修复 action/execute.rs (2 处)~~ (代码已正确，无需修复)
- [x] ~~修复 ui/render.rs (9 处)~~ (代码已正确，无需修复)
- [x] 清理 unused warnings
- [x] cargo fmt
- [x] cargo clippy

**修复详情**:
- 在 `src/config/validators.rs:31-36` 添加空路径检查
- 添加单元测试 `test_validate_directory_empty_path`
- 所有 46 个测试通过，clippy 无警告

### Phase 1 MVP (已完成 ✅)

- [x] 目录选择 UI (`DirChooser` widget)
  - 文件浏览器界面，支持子目录导航
  - 键盘导航 (j/k, Enter, ←/→)
  - Git 仓库计数显示
  - 所有测试通过
  
- [x] 仓库列表渲染 (`RepoList` widget)
  - 虚拟列表优化，支持大量仓库
  - Git 状态显示 (clean/dirty)
  - 分支名称显示
  - 选中高亮和滚动跟踪
  
- [x] 搜索功能 (`SearchBox` + 过滤逻辑)
  - 实时搜索框，带焦点状态
  - 防抖处理 (300ms)
  - 大小写不敏感过滤
  - 空查询恢复完整列表
  
- [x] 键盘导航 (`keyboard.rs`)
  - 仓库导航 (↑/↓, Home/End, Ctrl+d/u)
  - 搜索模式 (/ 进入，Esc 退出)
  - 操作菜单 (Enter 打开，数字快捷键)
  - 目录选择器 (↑/↓/←/→ 导航，Esc 返回)
  - 帮助面板 (? 打开)

**测试状态**: 95 个测试全部通过 (87 单元测试 + 8 集成测试)

### Phase 2 核心功能 (已完成 ✅)

- [x] 操作菜单 (`ActionMenu` widget)
  - 居中弹出式菜单
  - 键盘导航 (j/k/↑/↓)
  - 快捷键直接执行 (c/w/v/f)
  - Enter 确认，Esc 取消
  
- [x] 帮助面板 (`HelpPanel` widget)
  - 完整快捷键文档
  - 分类显示（Navigation/Search/Actions/Global）
  - 固定尺寸 60x28
  - 自适应小终端
  
- [x] 命令执行功能 (`action/execute.rs`)
  - cd + claude
  - WebStorm/VS Code 打开
  - 文件管理器打开
  - 安全路径验证
  
- [x] 错误处理 UI
  - 统一 `AppError` 类型
  - 用户友好错误消息
  - 错误严重程度分级
  - 错误状态渲染

**测试状态**: 102 个测试全部通过 (94 单元测试 + 8 集成测试)

---

### Phase 3 增强体验 (已完成 ✅)

- [x] Git 状态检测增强 (`src/git/`)
  - TTL 缓存（5 分钟）
  - 异步后台检测
  - 批量检测优化（并发限制 10）
  - 缓存命中 < 1ms（实际 86.93ns）
  
- [x] 主题支持 (`src/ui/theme.rs`)
  - Dark/Light 主题切换
  - 快捷键 `t` 切换
  - 配置持久化
  - 自定义颜色配置
  
- [x] 响应式布局 (`src/ui/layout.rs`)
  - 断点系统：SM(60)/MD(100)/LG(120)
  - 文本中间截断
  - 最小终端尺寸检查（80x24）
  - 自适应渲染策略
  
- [x] 性能优化验证
  - 16 个基准测试
  - 1000 仓库渲染：22.10µs（目标<16ms）
  - 搜索过滤：54.89µs（目标<50ms）
  - Git 缓存命中：86.93ns（目标<1ms）

**测试状态**: 130 个测试全部通过

---

## 🚀 开发清单

### Phase 0 收尾 (已完成 ✅)

- [x] 修复配置空路径验证 Bug (见 BUGFIX_EMPTY_PATH.md)
- [x] ~~修复 action/validators.rs (8 处)~~ (代码已正确，无需修复)
- [x] ~~修复 action/execute.rs (2 处)~~ (代码已正确，无需修复)
- [x] ~~修复 ui/render.rs (9 处)~~ (代码已正确，无需修复)
- [x] 清理 unused warnings
- [x] cargo fmt
- [x] cargo clippy

**修复详情**:
- 在 `src/config/validators.rs:31-36` 添加空路径检查
- 添加单元测试 `test_validate_directory_empty_path`
- 所有 46 个测试通过，clippy 无警告

### Phase 1 MVP (已完成 ✅)

- [x] 目录选择 UI (`DirChooser` widget)
  - 文件浏览器界面，支持子目录导航
  - 键盘导航 (j/k, Enter, ←/→)
  - Git 仓库计数显示
  - 所有测试通过
  
- [x] 仓库列表渲染 (`RepoList` widget)
  - 虚拟列表优化，支持大量仓库
  - Git 状态显示 (clean/dirty)
  - 分支名称显示
  - 选中高亮和滚动跟踪
  
- [x] 搜索功能 (`SearchBox` + 过滤逻辑)
  - 实时搜索框，带焦点状态
  - 防抖处理 (300ms)
  - 大小写不敏感过滤
  - 空查询恢复完整列表
  
- [x] 键盘导航 (`keyboard.rs`)
  - 仓库导航 (↑/↓, Home/End, Ctrl+d/u)
  - 搜索模式 (/ 进入，Esc 退出)
  - 操作菜单 (Enter 打开，数字快捷键)
  - 目录选择器 (↑/↓/←/→ 导航，Esc 返回)
  - 帮助面板 (? 打开)

**测试状态**: 95 个测试全部通过 (87 单元测试 + 8 集成测试)

### Phase 3 增强体验 (已完成 ✅)

- [x] Git 状态检测增强 (`src/git/`)
  - TTL 缓存（5 分钟）
  - 异步后台检测
  - 批量检测优化（并发限制 10）
  - 缓存命中 < 1ms（实际 86.93ns）
  
- [x] 主题支持 (`src/ui/theme.rs`)
  - Dark/Light 主题切换
  - 快捷键 `t` 切换
  - 配置持久化
  - 自定义颜色配置
  
- [x] 响应式布局 (`src/ui/layout.rs`)
  - 断点系统：SM(60)/MD(100)/LG(120)
  - 文本中间截断
  - 最小终端尺寸检查（80x24）
  - 自适应渲染策略
  
- [x] 性能优化验证
  - 16 个基准测试
  - 1000 仓库渲染：22.10µs（目标<16ms）
  - 搜索过滤：54.89µs（目标<50ms）
  - Git 缓存命中：86.93ns（目标<1ms）

**测试状态**: 130 个测试全部通过

---

## 📝 常见问题

### Q: 为什么使用 AppResult 而不是直接 Result？

A: `AppResult<T>` 是 `Result<T, AppError>` 的别名，便于统一错误处理。所有可能失败的函数都应返回 `AppResult`。

### Q: 如何添加新的 Action？

A:
1. 在 `src/action/types.rs` 添加枚举变体
2. 在 `src/action/execute.rs` 添加执行逻辑
3. 在 `src/constants.rs` 添加到白名单
4. 在 `src/ui/render.rs` 添加到菜单显示

### Q: 如何调试 UI 渲染问题？

A: 使用 `tracing` 日志：
```rust
tracing::debug!("Rendering with state: {:?}", app.state);
```

运行：`RUST_LOG=debug cargo run`

---

**最后更新**: 2026-03-06
**Phase 1 状态**: ✅ 已完成
