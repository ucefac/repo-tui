# repotui 开发文档索引

**项目**: GitHub 仓库管理 TUI 工具  
**当前阶段**: Phase 0 - 安全基础 + 架构搭建  
**更新日期**: 2026-03-06  
**最后更新**: 添加 CONFIG-001 Bug 文档

---

## 📚 文档目录

### 项目状态报告

| 文档 | 说明 | 状态 |
|------|------|------|
| [PHASE0_COMPLETE.md](./PHASE0_COMPLETE.md) | Phase 0 完成报告 | ⚠️ 95% 完成 |
| [PHASE0_STATUS.md](./PHASE0_STATUS.md) | Phase 0 详细实施状态 | 📋 包含 CONFIG-001 Bug 记录 |
| [FIX_PROGRESS.md](./FIX_PROGRESS.md) | 修复进度记录 | 📋 包含 Bug 修复计划 |
| [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md) | CONFIG-001: 空路径验证 Bug 修复方案 | 🔴 **新增** |

### Bug 修复文档

| 文档 | Bug ID | 描述 | 优先级 |
|------|--------|------|--------|
| [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md) | CONFIG-001 | 配置空路径验证不充分 | 🔴 P0 |

---

## 🎯 当前进度摘要

### 总体状态：85% 完成

> ⚠️ **更新**: 发现 CONFIG-001 Bug，进度从 95% 调整至 85%

| 模块 | 完成度 | 状态 | 备注 |
|------|--------|------|------|
| 项目架构 | 100% | ✅ 完成 | - |
| 安全核心 | 90% | ⚠️ 需修复 | 发现空路径验证 Bug |
| Elm 架构 | 100% | ✅ 完成 | - |
| UI 框架 | 95% | ⚠️ 待修复 | 9 处类型错误 |
| 配置管理 | 95% | ⚠️ 需修复 | CONFIG-001 |
| 错误系统 | 100% | ✅ 完成 | - |
| 测试框架 | 80% | ⚠️ 待完善 | - |

### 剩余工作

#### 🔴 运行时 Bug (P0)

- **Bug ID**: CONFIG-001
- **问题**: 配置 `main_directory = ""` 导致程序崩溃
- **影响**: 高（用户体验）
- **修复文档**: [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md)
- **预计修复时间**: 30 分钟

#### ⚠️ 编译错误 (P1)

- **剩余错误**: 19 个
- **错误类型**: 全部为 `mismatched types`
- **分布**: 
  - `action/validators.rs` - 8 处
  - `action/execute.rs` - 2 处
  - `ui/render.rs` - 9 处
- **预计修复时间**: 45 分钟

---

## 📖 关键文档

### 架构设计

- **Elm 架构**: Model-Msg-Update-View-Cmd 完整实现
- **安全设计**: 路径验证（5+1 层验证链）、命令白名单、无 shell 注入
- **错误处理**: 统一 AppError 类型系统

### 核心模块

```
src/
├── app/           # Elm 架构核心 (Model/Msg/Update/State)
│   ├── update.rs  # ⚠️ 需修复：错误处理优化
├── config/        # 配置管理 (加载/保存/验证)
│   ├── validators.rs  # ⚠️ 需修复：添加空路径检查
│   └── types.rs       # 可选：反序列化验证
├── repo/          # 仓库发现和状态检测
├── action/        # 命令执行 (安全实现)
│   ├── validators.rs  # ⚠️ 需修复：8 处类型错误
│   └── execute.rs     # ⚠️ 需修复：2 处类型错误
├── ui/            # Ratatui 渲染
│   └── render.rs      # ⚠️ 需修复：9 处类型错误
├── handler/       # 键盘事件处理
├── runtime/       # 异步任务执行
└── error.rs       # 统一错误类型
```

### 技术栈

- **TUI 框架**: Ratatui 0.29 + Crossterm 0.28
- **异步运行时**: Tokio (最小化特性)
- **配置管理**: Serde + TOML
- **错误处理**: Thiserror
- **路径处理**: path-absolutize + dirs

---

## 🔧 快速开始

### 开发环境

```bash
# 要求
Rust 1.75+
Git (用于仓库状态检测)

# 构建
cargo build

# 测试
cargo test

# 运行
cargo run
```

### 当前已知问题

#### 1. 配置空路径 Bug (CONFIG-001)

**症状**:
```
Failed to scan directory: Failed to read directory : No such file or directory (os error 2)
```

**临时解决方案**:
编辑配置文件 `~/Library/Application Support/repotui/config.toml`:
```toml
# 将
main_directory = ""
# 改为
main_directory = "/Users/你的用户名/Developer/github"
```

**永久修复**: 见 [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md)

#### 2. 编译错误修复

所有剩余错误遵循同一模式：

```rust
// 错误写法:
return Err(ActionError::CommandNotFound("cmd".to_string()));

// 正确写法:
return Err(AppError::Action(ActionError::CommandNotFound("cmd".to_string())));
```

---

## 📊 开发计划

| 阶段 | 目标 | 状态 | 备注 |
|------|------|------|------|
| Phase 0 | 安全基础 + 架构搭建 | ⚠️ 85% | 发现 CONFIG-001 Bug |
| Phase 1 | MVP 核心功能 | ⏳ 待开始 | 目录选择 UI、仓库列表 |
| Phase 2 | 操作执行 + 错误处理 | ⏳ 待开始 | 命令执行、帮助面板 |
| Phase 3 | 性能优化 + 增强体验 | ⏳ 待开始 | Git 状态、模糊搜索 |

### Phase 0 收尾任务

- [ ] 修复 CONFIG-001 空路径验证 Bug
- [ ] 修复 19 个编译错误
- [ ] 清理 warnings
- [ ] cargo fmt & clippy
- [ ] 运行测试套件

---

## 📝 变更记录

### 2026-03-06 (更新)
- 🔴 发现 CONFIG-001 Bug: 配置空路径验证不充分
- 📄 创建 [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md) 修复方案
- 📊 更新进度：95% → 85%（因发现关键 Bug）
- 📝 更新所有状态文档

### 2026-03-06 (初始)
- 创建项目脚手架
- 实现 Elm 架构核心
- 完成安全模块
- 修复 60+ 编译错误 (剩余 19 个)

---

## 🔗 相关链接

- [README.md](../README.md) - 项目说明
- [CLAUDE.md](../CLAUDE.md) - 开发指南
- [ghclone-prd-v2.md](ghclone-prd-v2.md) - 需求文档
- [Cargo.toml](../Cargo.toml) - 依赖配置
- [config.toml.example](../config.toml.example) - 配置示例

---

## 🆘 获取帮助

### 常见问题

**Q: 运行时出现 "Failed to read directory" 错误怎么办？**  
A: 这是 CONFIG-001 Bug。查看 [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md) 了解详情和临时解决方案。

**Q: 编译时出现类型不匹配错误？**  
A: 所有 ActionError 需要包装为 AppError::Action(...)。参考 FIX_PROGRESS.md 的修复模式。

### 报告问题

- Bug 报告: 在 GitHub Issues 创建新 Issue
- 标记为 `bug` 和 `phase-0`
- 包含复现步骤和系统信息

---

**维护者**: repotui Team  
**许可证**: MIT
