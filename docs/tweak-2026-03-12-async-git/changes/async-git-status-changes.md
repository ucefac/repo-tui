# Git 状态检测异步优化修改记录

**创建时间**: 2026-03-12
**作者**: dev-team
**状态**: 已完成

---

## 修改内容

### 文件：`src/git/scheduler.rs`

#### 1. 添加 Semaphore 导入

```rust
use tokio::sync::{mpsc, Semaphore};
```

#### 2. 重构 `schedule_batch` 方法

**修改前**（批次处理，阻塞 UI）：
```rust
pub async fn schedule_batch(&self, repos: Vec<(usize, Repository)>) {
    if repos.is_empty() {
        return;
    }

    const BATCH_SIZE: usize = 10;

    for chunk in repos.chunks(BATCH_SIZE) {
        let futures: Vec<_> = chunk
            .iter()
            .map(|(idx, repo)| self.schedule_check(*idx, repo.path.clone()))
            .collect();

        // 等待这一批全部完成
        futures::future::join_all(futures).await;
    }
}
```

**修改后**（信号量控制并发，非阻塞）：
```rust
pub async fn schedule_batch(&self, repos: Vec<(usize, Repository)>) {
    if repos.is_empty() {
        return;
    }

    // 使用信号量限制并发数
    const CONCURRENT_LIMIT: usize = 10;
    let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

    // 所有任务立即 spawn，竞争信号量许可
    for (idx, repo) in repos {
        let cache = Arc::clone(&self.cache);
        let msg_tx = self.msg_tx.clone();
        let permit = Arc::clone(&semaphore);

        tokio::spawn(async move {
            // 获取许可 - 限制并发数
            let _permit = permit.acquire_owned().await;

            // 检查缓存
            if let Some(cached) = cache.get(&repo.path) {
                let _ = msg_tx
                    .send(AppMsg::GitStatusChecked(idx, Ok(cached.status)))
                    .await;
                return;
            }

            // 缓存未命中 - 检查 git 状态
            match check_git_status(&repo.path).await {
                Ok(status) => {
                    cache.insert(repo.path.clone(), status.clone());
                    let _ = msg_tx
                        .send(AppMsg::GitStatusChecked(idx, Ok(status)))
                        .await;
                }
                Err(e) => {
                    let repo_error = crate::error::RepoError::GitError(e.to_string());
                    let _ = msg_tx
                        .send(AppMsg::GitStatusChecked(idx, Err(repo_error)))
                        .await;
                }
            }
        });
    }
}
```

#### 3. 添加新测试用例

```rust
#[tokio::test]
async fn test_schedule_batch_concurrent() {
    use std::time::Instant;

    let cache = Arc::new(StatusCache::new(60));
    let (tx, mut rx) = mpsc::channel::<AppMsg>(100);
    let scheduler = GitStatusScheduler::new(cache, tx);

    let temp_dir = TempDir::new().unwrap();
    // 创建 20 个仓库（超过并发限制 10）
    let repos: Vec<(usize, Repository)> = (0..20)
        .map(|i| {
            (
                i,
                Repository {
                    name: format!("repo{}", i),
                    path: temp_dir.path().join(format!("repo{}", i)),
                    last_modified: None,
                    is_dirty: false,
                    branch: None,
                    is_git_repo: false,
                    source: RepoSource::Standalone,
                },
            )
        })
        .collect();

    let start = Instant::now();

    // 立即调度所有任务 - 应该立即返回
    scheduler.schedule_batch(repos).await;

    // 方法应该立即返回，不等待所有任务
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100, "schedule_batch 应该立即返回");

    // 给异步任务时间完成
    tokio::time::sleep(Duration::from_millis(500)).await;

    // 应该收到 20 条消息
    let mut count = 0;
    while let Ok(msg) = rx.try_recv() {
        if let AppMsg::GitStatusChecked(_, _) = msg {
            count += 1;
        }
    }
    assert_eq!(count, 20);
}
```

---

## 测试结果

```
running 7 tests
test git::scheduler::tests::test_cache_len ... ok
test git::scheduler::tests::test_schedule_check_cache_hit ... ok
test git::scheduler::tests::test_refresh_status ... ok
test git::scheduler::tests::test_schedule_check_cache_miss ... ok
test git::scheduler::tests::test_schedule_batch ... ok
test git::scheduler::tests::test_schedule_batch_concurrent ... ok
test git::scheduler::tests::test_cleanup_cache ... ok

test result: ok. 7 passed; 0 failed
```

---

## 效果对比

| 指标 | 修改前 | 修改后 |
|------|--------|--------|
| UI 阻塞 | 是（按批次等待） | 否（立即返回） |
| 状态更新 | 批次完成后统一更新 | 每个完成后立即更新 |
| 并发控制 | 批次大小（10） | 信号量（10） |
| 用户体验 | 卡顿 | 流畅 |

---

## 提交信息

```
perf(git): 优化 Git 状态检测为完全异步执行

- 使用 Semaphore 替代批次处理，限制并发数但不阻塞
- 所有任务立即 spawn，完成一个更新一个
- UI 不再卡顿，用户体验更流畅
- 添加并发测试验证行为
```
