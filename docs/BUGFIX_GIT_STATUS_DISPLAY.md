# Git 状态显示 Bug 修复

**Bug ID**: UI-001  
**严重程度**: 🔴 高  
**状态**: ✅ 已修复  
**修复日期**: 2026-03-07

---

##  问题描述

用户报告仓库列表没有显示 Git 状态信息（分支名称、dirty/clean 状态）。

**现象**:
- 所有仓库只显示 `✓` 图标
- 没有显示分支名称
- 所有仓库都显示为 clean 状态

**根因**: Phase 3 实现了 Git 状态检测模块，但**仓库加载后没有触发 Git 状态检测**。

---

## 🔍 根因分析

### 问题流程

```
仓库加载 → app.repositories = repos → 结束 ❌
                                        ↓
                            Git 状态字段保持默认值
                            (is_dirty: false, branch: None)
```

### 代码问题

**文件**: `src/app/update.rs:119-131`

```rust
// ❌ 问题代码
AppMsg::RepositoriesLoaded(result) => match result {
    Ok(repos) => {
        app.repositories = repos;
        app.apply_filter();
        app.state = AppState::Running;
        // ⚠️ 缺少：Git 状态检测调度
    }
    // ...
}
```

---

## 🛠️ 修复方案

### 修改 1: `src/app/update.rs`

添加 Git 状态检测调度：

```rust
AppMsg::RepositoriesLoaded(result) => match result {
    Ok(repos) => {
        app.repositories = repos;
        app.apply_filter();
        app.state = AppState::Running;
        
        // ✅ 新增：调度 Git 状态检测
        if let Some(ref scheduler) = app.git_scheduler {
            let repos_with_idx: Vec<_> = app.repositories
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect();
            
            let scheduler_clone = Arc::clone(scheduler);
            tokio::spawn(async move {
                scheduler_clone.schedule_batch(repos_with_idx).await;
            });
        }
    }
    // ...
}
```

### 修改 2: `src/app/model.rs`

将 `git_scheduler` 包装为 `Arc` 以支持线程安全共享：

```rust
// 修改前
pub git_scheduler: Option<GitStatusScheduler>,

// 修改后
pub git_scheduler: Option<Arc<GitStatusScheduler>>,
```

---

## ✅ 验收测试

### 单元测试
```bash
cargo test --lib
# result: ok. 130 passed; 0 failed
```

### 手动测试

1. **启动 repotui**
   ```bash
   cargo run
   ```

2. **检查仓库列表显示**
   - ✅ 每个仓库显示 `✓` (clean) 或 `●` (dirty)
   - ✅ 显示分支名称如 `(main)`、`(feat/xxx)`
   - ✅ Dirty 仓库显示红色 `●`
   - ✅ Clean 仓库显示绿色 `✓`

3. **修改文件触发 dirty 状态**
   ```bash
   # 在某个仓库中修改文件
   cd ~/Developer/repo/some-repo
   echo "test" >> README.md
   
   # 等待 5 分钟后刷新 repotui
   # 应该看到该仓库显示红色 `●`
   ```

---

## 📊 修复影响

| 文件 | 变更行数 | 说明 |
|------|----------|------|
| `src/app/update.rs` | +17 行 | 添加 Git 状态调度 |
| `src/app/model.rs` | +2/-2 行 | Arc 包装 |
| **总计** | **+19 行** | |

---

## 🎯 性能验证

修复后 Git 状态检测性能：

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| 缓存命中 | < 1ms | 86.93ns | ✅ 11506x 优于目标 |
| 批量检测 100 仓库 | < 5s | 7.93ms | ✅ 630x 优于目标 |
| 不阻塞 UI | - | ✅ 异步 | - |

---

## 📝 相关文件

- [Phase 3 完成报告](./PHASE3_COMPLETE.md)
- [Git 状态检测模块文档](./PHASE3_TASK1_COMPLETE.md)
- [性能测试报告](./PERFORMANCE_REPORT.md)

---

**修复人**: AI Assistant  
**修复时间**: 2026-03-07  
**测试通过**: ✅
