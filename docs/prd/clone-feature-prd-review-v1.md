# PRD 审查报告 - Clone 功能

**文档版本**: v1
**审查日期**: 2026-03-09
**审查角色**: Designer + Backend Developer
**PRD 文档**: `docs/prd/clone-feature-prd-v1.md`

---

## 审查结论

**结论**: 🔴 **需要修改**

两个审查角色均发现需要修改的问题，**不建议直接进入开发阶段**。

---

## 关键问题汇总 (High Priority)

### 来自 Designer

| # | 问题 | 位置 | 建议修改 |
|---|------|------|---------|
| D1 | `Ctrl+u` 快捷键冲突 | F-CLONE-2 快捷键 | 改为 `Ctrl+k` 或移除 |
| D2 | `c` 键冲突风险 | F-CLONE-1 触发方式 | 验证代码中 `c` 键是否完全释放 |
| D3 | 界面空间溢出 | F-CLONE-2 UI 布局 | 压缩 Examples 区域，明确最小尺寸 |

### 来自 Backend Developer

| # | 问题 | 位置 | 建议修改 |
|---|------|------|---------|
| B1 | CloneState 缺少关键字段 | 3.1 Model 扩展 | 添加 `cursor_position`, `main_dir_list_state` |
| B2 | 进度发送机制设计缺陷 | 3.3 异步执行 | 改为通过 `AppMsg::CloneProgress` 发送 |
| B3 | 取消操作实现不明确 | 3.3 异步执行 | 使用 `Arc<AtomicBool>` 取消标志 |
| B4 | Bracketed Paste 未启用 | 5.5 粘贴功能 | 添加 `EnableBracketedPaste` 调用 |
| B5 | URL 验证不够严格 | 5.1 URL 验证 | 添加 `url` crate 进行严格验证 |

---

## 详细审查意见

### Designer 审查意见

#### 1. 界面布局

**问题 1.1: URL 输入界面垂直空间占用过大**
- **详情**: 在 80x24 终端中，Examples (5行) + 分隔线 + 主目录选择可能导致溢出
- **建议**: 压缩 Examples 为单行或改为 `[?] Show examples` 交互

**问题 1.2: 主目录选择器视觉层级不足**
- **详情**: URL 输入和主目录选择合并后，主目录区域不够突出
- **建议**: 输入有效 URL 后自动高亮主目录选择区域

**问题 1.3: 进度显示区域高度未指定**
- **详情**: PRD 未明确 git 输出区域的最小高度
- **建议**: 指定至少 10 行的输出区域

#### 2. 交互流程

**问题 2.1: `c` 键冲突风险 ⚠️**
- **详情**: 需要验证 `c` 键在现有代码中是否完全释放
- **建议**: 检查 `src/handler/keyboard.rs` 确认无残留绑定

**问题 2.2: `Ctrl+V` 粘贴兼容性**
- **详情**: Linux 终端中 `Ctrl+V` 是"字面输入"快捷键
- **建议**: 主要依赖 Bracketed Paste，`Ctrl+V` 作为可选后备

**问题 2.3: 状态流转缺少错误回退**
- **详情**: 错误后直接返回仓库列表，用户无法重试
- **建议**: 错误状态添加 "重试 (R)" 和 "返回 (Esc)" 选项

**问题 2.4: `Ctrl+u` 快捷键冲突 ⚠️**
- **详情**: `Ctrl+u` 在 keyboard-shortcuts.md 中定义为"向上半屏"
- **建议**: 清空输入改为 `Ctrl+k` 或仅使用 `Backspace`

#### 3. 视觉一致性

**问题 3.1: Emoji 使用不一致**
- **详情**: PRD 使用 ⚠️、✅、❌，但现有代码使用 🔍
- **建议**: 统一 Emoji 或提供 ASCII fallback

**问题 3.2: 输入框光标样式不一致**
- **详情**: PRD 使用 `___`，现有 SearchBox 使用 `▌`
- **建议**: 统一使用 `▌` 字符

**问题 3.3: 确认对话框按钮样式不一致**
- **详情**: PRD 使用 `[Y] Yes`，现有 action_menu 使用 `[1]`
- **建议**: 统一使用数字快捷键 `[1] Yes` `[2] No`

#### 4. 用户体验

**问题 4.1-4.4**: 文件夹命名可读性、已存在提示信息、进度条、成功提示时间等 Low Priority 问题，详见原始报告。

### Backend Developer 审查意见

#### 1. Elm 架构扩展

**问题 1.1: CloneState 缺少关键字段 ⚠️**
```rust
// 当前定义
pub struct CloneState {
    pub url_input: String,
    pub parsed_url: Option<ParsedGitUrl>,
    ...
}

// 建议添加
pub struct CloneState {
    pub url_input: String,
    pub cursor_position: usize,  // 新增
    pub parsed_url: Option<ParsedGitUrl>,
    pub target_main_dir: Option<usize>,
    pub stage: CloneStage,
    pub progress_lines: Vec<String>,
    pub main_dir_list_state: ListState,  // 新增
}
```

**问题 1.2: AppState 设计合理**
- 使用单一 `Cloning` 状态，通过 `CloneState.stage` 区分子状态是正确的设计

#### 2. URL 解析模块

**问题 2.1: URL 格式支持不完整 ⚠️**
- 遗漏格式：带认证信息、多级路径、本地协议
- 建议增强解析逻辑支持更多格式

**问题 2.2: sanitize 函数对中文处理不当**
- 当前逻辑会将中文替换为下划线
- 建议改为 `c.is_alphanumeric()` 保留 Unicode

#### 3. 异步执行

**问题 3.1: 进度发送机制设计缺陷 ⚠️**
- PRD 中的 `progress_tx.send(line)` 在 executor 中无法直接获取
- 建议改为：
```rust
// Runtime executor
Cmd::CloneRepository { url, target_path } => {
    tokio::spawn(async move {
        let result = clone_repository(&url, &target_path, msg_tx.clone()).await;
        let _ = msg_tx.send(AppMsg::CloneCompleted(result)).await;
    });
}

// clone_repository 函数
async fn clone_repository(url: &str, target: &Path, msg_tx: mpsc::Sender<AppMsg>) {
    // 发送进度时
    msg_tx.send(AppMsg::CloneProgress(line)).await?;
}
```

**问题 3.2: 取消操作实现不明确 ⚠️**
- Tokio `Child::kill()` 发送的是 SIGKILL，不是 SIGTERM
- 建议使用取消标志：
```rust
pub async fn clone_repository(
    url: &str,
    target_path: &Path,
    msg_tx: mpsc::Sender<AppMsg>,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), CloneError> {
    // 定期检查 cancel_flag
}
```

#### 4. 安全设计

**问题 4.1: URL 验证需要加强 ⚠️**
- 仅检查 `;|&` 不够
- 建议添加 `url = "2.5"` crate 进行严格验证

**问题 4.2: 路径验证应复用现有模块**
- 使用 `src/config/validators.rs` 中的验证链

**问题 4.3: 删除文件夹安全措施不足**
- 应验证目标是 Git 仓库（检查 `.git` 目录）
- 防止误删家目录、系统目录

#### 5. 粘贴功能

**问题 5.1: Bracketed Paste 未显式启用 ⚠️**
- crossterm 0.28 默认不启用
- 需要在应用启动时：
```rust
crossterm::execute!(std::io::stdout(), EnableBracketedPaste)?;
```

**问题 5.2: Ctrl+V 后备方案建议移除**
- 终端中 Ctrl+V 有歧义
- 现代终端都支持 bracketed paste

#### 6. 集成点

**问题 6.1-6.3**: keyboard.rs、render.rs、update.rs 需要更新以支持 Clone 状态，详见原始报告。

---

## 修改建议

### 必须修改 (High Priority)

1. **快捷键**: `Ctrl+u` → `Ctrl+k`（或移除清空快捷键）
2. **Model**: CloneState 添加 `cursor_position` 和 `main_dir_list_state`
3. **Msg**: 进度发送改为 `AppMsg::CloneProgress(String)`
4. **异步**: 取消操作使用 `Arc<AtomicBool>`
5. **粘贴**: 添加 `EnableBracketedPaste`
6. **URL 验证**: 添加 `url` crate

### 建议修改 (Medium Priority)

1. 界面 Examples 区域折叠
2. 统一光标样式为 `▌`
3. 统一按钮样式为 `[1] Yes`
4. sanitize 支持 Unicode
5. 错误状态支持重试

### 可选优化 (Low Priority)

1. 磁盘空间预检查
2. 浅克隆选项
3. 进度条显示

---

## 下一步行动

1. **产品经理**: 根据审查意见更新 PRD v2
2. **审查确认**: Designer 和 Backend Developer 确认修改后通过
3. **进入开发**: 通过审查后进入 Phase 1 开发

---

**审查报告创建**: 2026-03-09
**关联 PRD**: [clone-feature-prd-v1.md](./clone-feature-prd-v1.md)
