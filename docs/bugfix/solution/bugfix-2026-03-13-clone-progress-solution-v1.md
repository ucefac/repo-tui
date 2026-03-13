# Bug 修复方案

## 方案对比

### 方案 A: 在 CloneCompleted 和 CancelClone 处理中清除 progress_lines

**修改位置**: `src/app/update.rs`

**修改内容**:
1. 在 `AppMsg::CloneCompleted` 处理中，无论成功或失败，都清除 `progress_lines`
2. 在 `AppMsg::CancelClone` 处理中，清除 `progress_lines`

**优点**:
- 修改简单直接，代码改动最小
- 逻辑清晰：Clone 结束（无论何种原因）后清除进度日志
- 符合预期行为：进度日志只在 Clone 过程中显示

**缺点**:
- 如果用户想查看 Clone 历史日志（调试目的），则无法实现

**风险评估**: 低风险，仅影响 UI 显示，不影响功能逻辑

### 方案 B: 使用临时缓冲区，Clone 完成后自动丢弃

**修改位置**: `src/app/state.rs`, `src/app/update.rs`

**修改内容**:
1. 将 `progress_lines` 改为 `Option<Vec<String>>`
2. Clone 完成后设置为 `None`

**优点**:
- 更明确的语义：`None` 表示 Clone 已结束
- 类型系统保证不会误用已结束的进度日志

**缺点**:
- 需要修改 `CloneState` 结构
- 需要修改所有访问 `progress_lines` 的代码
- 改动范围较大

**风险评估**: 中等风险，涉及多个文件修改

## 推荐方案

**推荐方案 A**

理由：
1. 修改简单，只需要在两个消息处理中添加一行代码
2. 风险低，不影响现有逻辑
3. 符合用户预期：Clone 完成后进度日志应该消失
4. 如果需要保留历史日志，可以另外添加日志功能

## 实施计划

### 步骤 1: 修改 `src/app/update.rs`

**修改 `AppMsg::CloneCompleted` 处理**（约第 1251 行）:

```rust
AppMsg::CloneCompleted(result) => {
    match result {
        Ok(_repo) => {
            // Clone successful - refresh repositories
            // ... existing code ...

            // Clear progress lines before returning to Running state
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.clear_progress();
            }

            // Return to Running state
            app.state = AppState::Running;
        }
        Err(e) => {
            // Clone failed - show error
            if let Some(clone_state) = app.state.clone_state_mut() {
                clone_state.stage = crate::app::state::CloneStage::Error(e);
                // Clear progress lines to prepare for retry or cancel
                clone_state.clear_progress();
            }
        }
    }
}
```

**修改 `AppMsg::CancelClone` 处理**（约第 1296 行）:

```rust
AppMsg::CancelClone => {
    // Cancel clone operation and return to Running state
    if let Some(clone_state) = app.state.clone_state_mut() {
        clone_state.cancel();
        clone_state.clear_progress();  // Add this line
    }
    app.state = AppState::Running;
}
```

### 步骤 2: 编译测试

```bash
cd .worktrees/bugfix-clone-progress
cargo build
cargo test
```

### 步骤 3: 验证修复

1. 运行应用
2. 执行 Clone Repository 操作
3. Clone 完成后检查界面，确认没有 "remote:" 等进度日志残留
4. 测试取消 Clone 操作，确认进度日志被清除
5. 测试 Clone 失败场景，确认进度日志被清除

## 风险评估

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| 修改引入编译错误 | 低 | 低 | 编译测试可发现 |
| 修改影响其他功能 | 低 | 低 | 仅修改 UI 显示逻辑 |
| 测试覆盖不足 | 中 | 低 | 手动验证 + 单元测试 |

## 验收标准

1. Clone 成功后，界面不显示 "remote:" 等进度日志
2. Clone 失败后，界面不显示进度日志（只显示错误信息）
3. 取消 Clone 后，界面不显示进度日志
4. 所有现有测试通过
5. 代码符合项目规范（cargo fmt, cargo clippy）
