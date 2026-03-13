# 仓库移动功能 - 交付总结

**功能名称**: 仓库移动到指定主目录
**交付日期**: 2026-03-13
**版本**: v1.0

---

## 1. 功能概述

在仓库列表界面添加 `M` 键（Shift+m），触发将当前选择的仓库移动到指定主目录的操作。

### 核心特性
- **快捷键触发**: 按下 `M` 键打开移动目标选择器
- **智能跳过**: 如果目标主目录与当前相同，自动跳过并显示提示
- **Toast 通知**: 移动成功/失败显示友好的提示信息
- **仓库类型支持**: 支持 standard 仓库和 git 仓库
- **跨文件系统移动**: 自动 fallback 到 copy+delete 模式

---

## 2. 交付物清单

### 2.1 代码变更

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `src/error.rs` | 修改 | 添加 `MoveError` 错误类型 |
| `src/repo/move_module.rs` | 新增 | 仓库移动核心逻辑 |
| `src/repo/mod.rs` | 修改 | 导出移动模块 |
| `src/app/msg.rs` | 修改 | 添加移动相关消息类型 |
| `src/app/state.rs` | 修改 | 添加 `ChoosingMoveTarget` 状态 |
| `src/app/model.rs` | 修改 | 添加 `toast_manager` 字段 |
| `src/app/update.rs` | 修改 | 添加移动消息处理逻辑 |
| `src/handler/keyboard.rs` | 修改 | 添加 `M` 键和移动选择器键盘处理 |
| `src/runtime/executor.rs` | 修改 | 添加 `MoveRepository` 命令执行 |
| `src/ui/widgets/mod.rs` | 修改 | 导出新组件 |
| `src/ui/widgets/toast.rs` | 新增 | Toast 通知组件 |
| `src/ui/widgets/move_target_selector.rs` | 新增 | 移动目标选择器组件 |
| `src/ui/render.rs` | 修改 | 添加选择器和 Toast 渲染 |

### 2.2 文档

| 文件 | 说明 |
|------|------|
| `docs/feature/prd/ghclone-repo-move-prd-v1.md` | PRD 需求文档 |
| `docs/feature/design/ghclone-repo-move-design-v1.md` | UI/UX 设计规范 |
| `docs/feature/review/ghclone-repo-move-review-v1.md` | PRD 审查报告 |
| `docs/feature/test/ghclone-repo-move-test-plan.md` | 测试计划 |

---

## 3. 技术实现要点

### 3.1 移动逻辑

```rust
// 同文件系统：使用 rename (原子操作)
fs::rename(&repo.path, &target_path)

// 跨文件系统：fallback 到 copy+delete
move_cross_device(&repo.path, &target_path)
```

### 3.2 状态机

```
Running ──[M 键]──> ChoosingMoveTarget
                        │
         ┌──────────────┼──────────────┐
         │              │              │
       [Esc]         [Enter]       [Enter - 同目录]
         │              │              │
         ▼              ▼              ▼
      Running    执行移动操作    显示 Info + Running
                        │
              ┌─────────┴─────────┐
              │                   │
            [成功]              [失败]
              │                   │
              ▼                   ▼
         更新路径 + Toast    显示 Error Toast
```

### 3.3 Toast 系统

新增通用 Toast 组件，支持四种类型：
- **Success**: 绿色，3 秒
- **Error**: 红色，5 秒
- **Warning**: 琥珀色，5 秒
- **Info**: 蓝色，2 秒

---

## 4. 质量指标

### 4.1 测试覆盖率

| 模块 | 单元测试 | 集成测试 |
|------|----------|----------|
| 移动逻辑 | ✅ 3 个测试 | ⏳ 待执行 |
| Toast 组件 | ✅ 4 个测试 | ⏳ 待执行 |
| 选择器组件 | ✅ 2 个测试 | ⏳ 待执行 |

### 4.2 回归测试

- ✅ 所有现有测试通过 (65 个测试)
- ✅ 无破坏性变更

### 4.3 代码质量

- ✅ `cargo check` 通过
- ✅ 无新增 clippy 警告
- ✅ 符合项目编码规范

---

## 5. 使用说明

### 5.1 快捷键

| 按键 | 功能 |
|------|------|
| `M` | 打开移动目标选择器 |
| `↑/↓` 或 `k/j` | 导航选择 |
| `Enter` | 确认移动 |
| `Esc` | 取消操作 |
| `Home/End` | 跳转到第一/最后一项 |

### 5.2 操作流程

1. 在仓库列表中选择要移动的仓库
2. 按下 `M` 键
3. 使用方向键选择目标主目录
4. 按 `Enter` 确认移动
5. 等待移动完成，显示 Toast 通知

---

## 6. 已知限制

| 限制 | 影响 | 缓解措施 |
|------|------|----------|
| 大仓库移动时 UI 可能短暂卡顿 | 用户体验 | 未来可考虑进度条 |
| 不支持重命名移动 | 功能限制 | 移动到目标目录后手动重命名 |
| 符号链接直接移动不跟随 | 设计决策 | PRD 中已明确 |

---

## 7. 后续改进建议

### Phase 2 (可选)
- [ ] 添加移动进度条显示
- [ ] 支持移动时重命名
- [ ] 批量移动多个仓库
- [ ] 移动历史记录

### Phase 3 (可选)
- [ ] 拖拽移动支持
- [ ] 最近移动目标快捷选择
- [ ] 移动冲突解决对话框

---

## 8. 项目干系人

| 角色 | 人员 |
|------|------|
| Product Manager | AI Agent |
| Designer | AI Agent |
| Backend Developer | AI Agent |
| Frontend Developer | AI Agent |
| Tester | AI Agent |
| Code Reviewer | AI Agent |

---

## 9. 审批签字

| 角色 | 状态 | 日期 |
|------|------|------|
| Product Manager | ✅ 验收通过 | 2026-03-13 |
| Designer | ✅ 设计验收通过 | 2026-03-13 |
| Tester | ⏳ 待验收 | - |

---

**交付完成时间**: 2026-03-13
**下次审查日期**: 2026-03-20
