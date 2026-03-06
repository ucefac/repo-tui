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

### Phase 1 MVP (下一步)

- [ ] 目录选择 UI
- [ ] 仓库列表渲染
- [ ] 搜索功能
- [ ] 键盘导航

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
