# Bug 分析报告

## Bug 信息

- **Bug ID**: clone-progress-logs-2026-03-13
- **严重程度**: 低（视觉干扰，不影响功能）
- **发现时间**: 2026-03-13
- **报告者**: 用户

## 问题描述

在执行 Clone Repository 操作完成后，仓库列表界面底部显示了多行 "remote:" 日志（git clone 的进度输出）。这些日志是 Clone 过程中的临时输出，应该在 Clone 完成后被清除，但实际上残留在界面上。

**Bug 现象截图**:
- `clone.png`: Clone 完成后，界面底部显示 7 行 "remote:"
- `clone-before.png`: Clone 完成前的正常界面（无 "remote:" 行）

## 根因分析

### 代码流程分析

1. **Clone 进度收集** (`src/runtime/executor.rs:267-288`):
   ```rust
   // git clone --progress 输出到 stderr
   // 每一行都被发送到 AppMsg::CloneProgress
   let trimmed = line.trim_end().to_string();
   if !trimmed.is_empty() {
       let _ = msg_tx.send(AppMsg::CloneProgress(trimmed)).await;
   }
   ```

2. **进度行存储** (`src/app/state.rs:170-177`):
   ```rust
   pub fn add_progress(&mut self, line: String) {
       self.progress_lines.push(line);
       // Keep only last 100 lines
       if self.progress_lines.len() > 100 {
           self.progress_lines.remove(0);
       }
   }
   ```

3. **Clone 完成处理** (`src/app/update.rs:1251-1286`):
   ```rust
   AppMsg::CloneCompleted(result) => {
       match result {
           Ok(_repo) => {
               // ... 刷新 repositories ...
               app.state = AppState::Running;  // ← 状态切换
               // ← 但没有清除 progress_lines
           }
           Err(e) => {
               // Clone failed - show error
               if let Some(clone_state) = app.state.clone_state_mut() {
                   clone_state.stage = CloneStage::Error(e);
               }
               // ← 也没有清除 progress_lines
           }
       }
   }
   ```

4. **取消 Clone 处理** (`src/app/update.rs:1296-1302`):
   ```rust
   AppMsg::CancelClone => {
       if let Some(clone_state) = app.state.clone_state_mut() {
           clone_state.cancel();
       }
       app.state = AppState::Running;  // ← 状态切换
       // ← 但没有清除 progress_lines
   }
   ```

### 问题根因

在 `CloneCompleted`（成功或失败）和 `CancelClone` 处理后，`AppState` 从 `Cloning` 变为 `Running`，但 `CloneState` 中的 `progress_lines` 没有被清除。

虽然 `CloneState` 在下次 Clone 时会被 `reset()` 清除，但如果 UI 在状态切换时还在渲染 Clone Dialog 的最后一帧，`progress_lines` 中的 "remote:" 行可能会短暂显示在界面上。

此外，从截图看，"remote:" 行显示在仓库列表的底部，这可能是 UI 渲染缓存或状态切换时序问题导致的。

## 影响范围

- **影响模块**: `src/app/update.rs`, `src/ui/widgets/clone_dialog.rs`
- **影响用户**: 所有使用 Clone Repository 功能的用户
- **影响功能**: Clone 操作完成后的界面显示

## 相关代码位置

| 文件 | 行号 | 说明 |
|------|------|------|
| `src/runtime/executor.rs` | 267-288 | Clone 进度收集 |
| `src/app/state.rs` | 170-177 | 进度行存储 |
| `src/app/state.rs` | 84-96 | `reset()` 方法 |
| `src/app/update.rs` | 1251-1286 | Clone 完成处理 |
| `src/app/update.rs` | 1296-1302 | 取消 Clone 处理 |
| `src/ui/widgets/clone_dialog.rs` | 291-310 | 进度显示渲染 |
