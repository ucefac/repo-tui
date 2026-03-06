# Phase 0 修复进度

**时间**: 2026-03-06  
**状态**: 进行中，发现 CONFIG-001 Bug

---

## 🐛 运行时 Bug

### CONFIG-001: 配置空路径验证不充分

**状态**: 🔴 待修复  
**优先级**: P0  
**文档**: [BUGFIX_EMPTY_PATH.md](./BUGFIX_EMPTY_PATH.md)

#### 问题描述
当配置文件中 `main_directory = ""` 为空字符串时，程序崩溃。

#### 修复计划
1. ✅ 诊断完成 - 已确定根因（验证逻辑与实际使用不一致）
2. ⏳ 阶段 1 - validators.rs 添加空路径检查
3. ⏳ 阶段 2 - update.rs 优化错误处理
4. ⏳ 阶段 3（可选）- types.rs 反序列化验证
5. ⏳ 阶段 4（可选）- render.rs 错误 UI 优化

#### 修复文件
- `src/config/validators.rs` - 添加空路径检查（10 行）
- `src/app/update.rs` - 错误处理优化（20 行）
- `src/config/types.rs` - 反序列化验证（可选，15 行）

---

## ⚙️ 编译错误修复

### 已修复

1. ✅ error.rs - 添加了 Clone trait、Result 类型、user_message 方法
2. ✅ config/validators.rs - 修复导入和返回类型
3. ✅ app/state.rs - 修复 DirEntry 导入
4. ✅ app/model.rs - 修复 ListState API

### 剩余关键错误

1. **AppState 可见性** - enum AppState 是私有的，需要 pub
2. **config 模块导出** - load_or_create_config 未正确导出
3. **ui/render.rs** - 缺少 Paragraph 等 widget 导入
4. **lib.rs** - crossterm::ErrorKind API 变更

### 类型不匹配错误（19 处）

**错误模式**: `ActionError` 需要包装为 `AppError::Action(...)`

**待修复文件**:
- `src/action/validators.rs` - 8 处
- `src/action/execute.rs` - 2 处
- `src/ui/render.rs` - 9 处

**修复示例**:
```rust
// ❌ 错误:
return Err(ActionError::CommandNotFound("cmd".to_string()));

// ✅ 正确:
return Err(AppError::Action(ActionError::CommandNotFound("cmd".to_string())));
```

---

## 📋 修复优先级

### P0 (立即修复)

- [ ] CONFIG-001: 空路径验证 Bug
  - [ ] validators.rs 添加空路径检查
  - [ ] update.rs 优化错误处理

### P1 (编译阻塞)

- [ ] error.rs 类型定义
- [ ] ActionError 包装修复 (19 处)
- [ ] AppState 可见性
- [ ] 模块导出修复

### P2 (代码质量)

- [ ] 清理 unused imports
- [ ] cargo fmt
- [ ] cargo clippy

---

## 🎯 建议策略

由于错误数量较多 (约 60 个) 且相互关联，建议采用以下策略：

1. **先修复运行时 Bug** - CONFIG-001 影响用户体验
2. **再修复编译错误** - 按优先级逐个模块修复
3. **暂时注释掉复杂功能** - 如异步运行时整合（如有必要）
4. **分阶段验证** - 每修复一个模块就 cargo check

---

## ⏱️ 时间估算

| 任务 | 预计时间 | 状态 |
|------|----------|------|
| CONFIG-001 修复 | 30 分钟 | ⏳ 待开始 |
| 编译错误修复 | 45 分钟 | ⏳ 待开始 |
| 代码清理 | 15 分钟 | ⏳ 待开始 |
| 测试验证 | 20 分钟 | ⏳ 待开始 |
| **总计** | **~1.5 小时** | - |

---

## 📊 进度统计

- **运行时 Bug**: 1 个 (0% 完成)
- **编译错误**: ~60 个 (约 70% 完成)
- **代码清理**: 待开始
- **测试**: 待开始

**整体进度**: 约 80%

---

**建议下一步**: 
1. 立即修复 CONFIG-001 空路径验证 Bug
2. 继续系统性修复编译错误
3. 或者先完成所有编译错误修复，再统一处理运行时 Bug

**最后更新**: 2026-03-06
