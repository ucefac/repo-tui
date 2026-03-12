# Git 状态检测异步优化分析

**创建时间**: 2026-03-12
**作者**: dev-team
**状态**: 已完成

---

## 问题描述

仓库列表渲染后会检测每个仓库的 git 状态，原来的实现在检测期间界面会卡住，无法进行操作。

## 原因分析

### 原有实现问题

在 `src/git/scheduler.rs` 的 `schedule_batch` 方法中：

```rust
// ❌ 问题代码
for chunk in repos.chunks(BATCH_SIZE) {
    let futures: Vec<_> = chunk
        .iter()
        .map(|(idx, repo)| self.schedule_check(*idx, repo.path.clone()))
        .collect();

    // 问题：等待这一批全部完成才继续下一批
    futures::future::join_all(futures).await;
}
```

虽然使用了 `tokio::spawn` 异步执行，但存在以下问题：

1. **按批次等待**：每 10 个一批，必须等这一批全部完成才继续下一批
2. **双重 spawn**：`schedule_batch` 中调用 `schedule_check`，而 `schedule_check` 内部又 spawn 了实际的任务
3. **UI 更新延迟**：一批内的所有任务完成后才统一更新 UI，用户感觉界面卡顿

## 解决方案

使用 `Semaphore`（信号量）限制并发数，而不是按批次等待：

```rust
// ✅ 改进后代码
const CONCURRENT_LIMIT: usize = 10;
let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

// 所有任务立即 spawn，它们会竞争信号量许可
for (idx, repo) in repos {
    let permit = Arc::clone(&semaphore);
    tokio::spawn(async move {
        // 获取许可 - 这限制了并发数
        let _permit = permit.acquire_owned().await;
        // 执行 git 状态检测...
    });
}
```

### 优点

1. **立即返回**：`schedule_batch` 方法立即返回，不等待任务完成
2. **增量更新**：每个任务完成后立即发送消息更新 UI，用户可以看到渐进式的状态更新
3. **并发控制**：通过信号量限制同时执行的任务数（10 个），避免系统资源耗尽
4. **无阻塞**：UI 线程不会被阻塞，用户可以正常操作界面

## 修改范围

| 文件 | 修改内容 |
|------|----------|
| `src/git/scheduler.rs` | 重构 `schedule_batch` 方法，使用 Semaphore 替代批次处理 |
| `src/git/scheduler.rs` | 添加新测试用例 `test_schedule_batch_concurrent` |

## 质量标准

- [x] 所有测试通过（7 个 scheduler 测试）
- [x] 代码编译通过
- [x] 无新增警告
- [x] 保持向后兼容
