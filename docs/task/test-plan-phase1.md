# Phase 1 MVP 测试策略文档

**版本**: 1.0  
**日期**: 2026-03-06  
**目标**: 为 Phase 1 MVP 的 4 个核心功能制定完整测试策略

---

## 1. 测试范围

### Phase 1 MVP 功能

| 功能 | 模块 | 测试优先级 |
|------|------|----------|
| 目录选择 UI | `ui::render`, `handler::keyboard` | P0 |
| 仓库列表渲染 | `ui::render`, `app::model` | P0 |
| 搜索功能 | `app::model`, `handler::keyboard` | P0 |
| 键盘导航 | `handler::keyboard`, `app::update` | P0 |

### 测试目标

- **单元测试覆盖率**: ≥80%
- **集成测试**: 4 个核心场景
- **测试执行时间**: <10 秒

---

## 2. 测试架构

### 2.1 测试金字塔

```
                    ╱╲╱╲
                   ╱E2E╲       2-4 个场景 (Phase 2)
                  ╱─────╲
                 ╱─────────╲
                ╱  集成测试  ╲     4-8 个测试用例
               ╱─────────────╲    流程验证
              ╱─────────────────╲
             ╱    单元测试        ╲   50+ 测试用例
            ╱─────────────────────╲  覆盖率 ≥80%
```

### 2.2 目录结构

```
tests/
├── unit/                      # 单元测试（与源码内联）
├── integration/               # 集成测试
│   ├── directory_selection.rs # 目录选择流程
│   ├── search_filtering.rs    # 搜索过滤
│   ├── keyboard_navigation.rs # 键盘导航
│   └── repo_list_rendering.rs # 仓库列表渲染
├── helpers/                   # 测试辅助工具
│   ├── mod.rs
│   ├── mock_fs.rs            # Mock 文件系统
│   └── mock_terminal.rs      # Mock 终端
└── fixtures/                  # 测试数据
    └── repos/
```

---

## 3. 单元测试计划

### 3.1 `config::validators` - 路径验证 (目标: 90%+ 覆盖)

**文件**: `src/config/validators.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_validate_directory_exists` | 存在的目录 | Ok |
| `test_validate_directory_empty` | 空字符串 | PathError |
| `test_validate_directory_not_found` | 不存在的路径 | DirectoryNotFound |
| `test_validate_directory_not_a_dir` | 文件路径 | NotADirectory |
| `test_validate_directory_outside_home` | /etc | DirectoryOutsideHome |
| `test_validate_directory_symlink` | 符号链接 | SymlinkNotAllowed |
| `test_validate_editor_whitelist` | "code", "vim" | Ok |
| `test_validate_editor_absolute_path` | "/usr/bin/code" | Ok/NotFound |
| `test_validate_config_complete` | 完整配置 | Ok |

**已存在**: ✅ `test_validate_directory_empty_path`, `test_validate_directory_not_exists`, `test_validate_directory_not_a_dir`, `test_validate_editor_whitelist`

**需添加**: 外部目录检查、符号链接检查

### 3.2 `repo::discover` - 仓库发现 (目标: 90%+ 覆盖)

**文件**: `src/repo/discover.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_discover_repositories` | 包含 git 目录 | 正确识别 |
| `test_discover_empty_directory` | 空目录 | 空 Vec |
| `test_discover_nested_repos` | 嵌套 git 目录 | 仅顶层 |
| `test_discover_permission_denied` | 无权限目录 | ScanFailed |
| `test_discover_sorting` | 多个仓库 | 按名称排序 |
| `test_is_git_repository` | .git 目录/文件 | true/false |
| `test_is_git_repository_submodule` | .git 文件 | true |

**已存在**: ✅ `test_discover_repositories`, `test_is_git_repository`

**需添加**: 空目录、权限错误、排序验证

### 3.3 `app::model` - 应用模型 (目标: 85%+ 覆盖)

**文件**: `src/app/model.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_apply_filter` | 搜索 "repo1" | 过滤正确 |
| `test_apply_filter_empty` | 空查询 | 显示全部 |
| `test_apply_filter_no_match` | "xyz" | 空结果 |
| `test_apply_filter_case_insensitive` | "REPO" | 匹配 "repo" |
| `test_selected_repository` | 选择第0项 | 返回正确 repo |
| `test_selected_repository_none` | 无选择 | None |
| `test_visible_count` | 终端高度 30 | 正确计算 |
| `test_update_scroll_offset` | 滚动场景 | offset 正确 |
| `test_repository_count` | 2 个仓库 | 返回 2 |

**已存在**: ✅ `test_apply_filter`, `test_apply_filter_empty`, `test_selected_repository`

**需添加**: 大小写不敏感、滚动逻辑、边界条件

### 3.4 `handler::keyboard` - 键盘处理 (目标: 85%+ 覆盖)

**文件**: `src/handler/keyboard.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_handle_running_keys_navigation` | j/k/↑/↓ | 发送正确消息 |
| `test_handle_running_keys_search` | / 或字符 | 进入搜索模式 |
| `test_handle_running_keys_actions` | Enter/o | OpenActions |
| `test_handle_running_keys_jump` | g/G | JumpToTop/Bottom |
| `test_handle_search_keys_input` | 字符 | SearchInput |
| `test_handle_search_keys_backspace` | Backspace | SearchBackspace |
| `test_handle_search_keys_esc` | Esc | 退出搜索 |
| `test_handle_action_menu_keys` | c/w/v/f | ExecuteAction |
| `test_handle_action_menu_close` | q/Esc | CloseActions |
| `test_handle_chooser_keys` | j/k/Enter | 目录导航 |
| `test_handle_global_quit` | Ctrl+C/q | Quit |

**已存在**: ✅ `test_handle_running_keys_navigation`, `test_handle_action_menu_keys`, `test_handle_search_keys`

**需添加**: 状态转换、边界测试、修饰键组合

### 3.5 `app::update` - 状态更新 (目标: 80%+ 覆盖)

**文件**: `src/app/update.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_update_search_input` | SearchInput('a') | query += 'a' |
| `test_update_search_backspace` | SearchBackspace | 删除字符 |
| `test_update_search_clear` | SearchClear | 清空查询 |
| `test_update_next_repo` | NextRepo | selected + 1 |
| `test_update_previous_repo` | PreviousRepo | selected - 1 |
| `test_update_jump_to_top` | JumpToTop | selected = 0 |
| `test_update_jump_to_bottom` | JumpToBottom | selected = last |
| `test_update_open_actions` | OpenActions | ShowingActions |
| `test_update_close_actions` | CloseActions | Running |
| `test_update_directory_nav` | DirectoryNavDown/Up | index ±1 |
| `test_update_directory_selected` | DirectorySelected | 保存配置 |
| `test_update_config_loaded_ok` | ConfigLoaded(Ok) | 加载仓库 |
| `test_update_config_loaded_err` | ConfigLoaded(Err) | 显示错误/选择器 |
| `test_update_repos_loaded` | RepositoriesLoaded | 更新列表 |

**已存在**: ✅ `test_directory_nav_down`, `test_directory_nav_up`

**需添加**: 所有其他状态转换

### 3.6 `action::types` - 操作类型 (目标: 90%+ 覆盖)

**文件**: `src/action/types.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_action_shortcut` | CdAndCloud | 'c' |
| `test_action_description` | OpenVsCode | 包含 "VS Code" |
| `test_action_all` | - | 4 个动作 |

**已存在**: ✅ 全部完成

### 3.7 `error` - 错误处理 (目标: 80%+ 覆盖)

**文件**: `src/error.rs`

| 测试用例 | 输入 | 预期结果 |
|---------|------|---------|
| `test_error_user_message` | DirectoryNotFound | 友好消息 |
| `test_error_severity` | NotFound | Info |
| `test_config_error_user_message` | NoReadPermission | 权限提示 |
| `test_action_error_user_message` | CommandNotFound | 命令提示 |

**已存在**: ✅ 全部完成

---

## 4. 集成测试计划

### 4.1 目录选择流程测试

**文件**: `tests/integration/directory_selection.rs`

```rust
#[tokio::test]
async fn test_directory_selection_flow() {
    // 1. 启动无配置状态
    // 2. 验证显示目录选择器
    // 3. 模拟导航 (j/k)
    // 4. 模拟选择 (Enter)
    // 5. 验证配置已保存
    // 6. 验证加载仓库列表
}

#[tokio::test]
async fn test_directory_selection_invalid_path() {
    // 1. 选择无权限目录
    // 2. 验证显示错误
    // 3. 验证保持在选择器状态
}
```

### 4.2 搜索过滤集成测试

**文件**: `tests/integration/search_filtering.rs`

```rust
#[tokio::test]
async fn test_search_realtime_filter() {
    // 1. 加载 5 个仓库
    // 2. 进入搜索模式 (/)
    // 3. 输入 "re"
    // 4. 验证过滤结果 (debounce 后)
    // 5. 验证列表更新
}

#[tokio::test]
async fn test_search_no_results() {
    // 1. 输入 "xyz"
    // 2. 验证显示 "无匹配结果"
}

#[tokio::test]
async fn test_search_case_insensitive() {
    // 1. 输入 "REPO"
    // 2. 验证匹配 "repo"
}
```

### 4.3 键盘导航集成测试

**文件**: `tests/integration/keyboard_navigation.rs`

```rust
#[tokio::test]
async fn test_navigation_basic() {
    // 1. 加载 10 个仓库
    // 2. j 键向下移动
    // 3. 验证选中第 2 个
    // 4. k 键向上移动
    // 5. 验证回到第 1 个
}

#[tokio::test]
async fn test_navigation_wrap() {
    // 1. 在第 1 个按 k
    // 2. 验证保持第 1 个
    // 3. 到最后一个按 j
    // 4. 验证保持最后一个
}

#[tokio::test]
async fn test_navigation_jump() {
    // 1. 按 g 跳到顶部
    // 2. 按 G 跳到底部
}
```

### 4.4 仓库列表渲染集成测试

**文件**: `tests/integration/repo_list_rendering.rs`

```rust
#[tokio::test]
async fn test_list_rendering() {
    // 1. 创建 mock 终端
    // 2. 加载仓库
    // 3. 渲染 UI
    // 4. 验证缓冲区包含仓库名称
}

#[tokio::test]
async fn test_list_virtual_scrolling() {
    // 1. 创建 100 个仓库
    // 2. 验证仅渲染可见项
}

#[tokio::test]
async fn test_list_empty_state() {
    // 1. 空目录
    // 2. 验证显示 "未发现 Git 仓库"
}
```

---

## 5. 测试辅助工具

### 5.1 Mock 文件系统

**文件**: `tests/helpers/mock_fs.rs`

```rust
pub struct MockFs {
    temp_dir: TempDir,
}

impl MockFs {
    pub fn new() -> Self;
    pub fn create_repo(&self, name: &str) -> PathBuf;
    pub fn create_nested_repos(&self, parent: &str, repos: &[&str]) -> PathBuf;
    pub fn path(&self) -> &Path;
}
```

### 5.2 Mock 终端

**文件**: `tests/helpers/mock_terminal.rs`

```rust
pub struct MockTerminal {
    backend: TestBackend,
    terminal: Terminal<TestBackend>,
}

impl MockTerminal {
    pub fn new(width: u16, height: u16) -> Self;
    pub fn render(&mut self, app: &App);
    pub fn assert_contains(&self, text: &str);
    pub fn press_key(&mut self, key: KeyEvent) -> AppMsg;
}
```

### 5.3 应用构建器

**文件**: `tests/helpers/mod.rs`

```rust
pub fn create_test_app_with_repos(repos: Vec<Repository>) -> App;
pub fn create_test_app_empty() -> App;
pub async fn run_app_with_timeout(app: App, duration: Duration) -> App;
```

---

## 6. 测试执行计划

### 6.1 执行命令

```bash
# 全部测试
cargo test

# 单元测试 (内联)
cargo test --lib

# 集成测试
cargo test --test '*'

# 特定模块
cargo test config::validators
cargo test repo::discover
cargo test app::model
cargo test handler::keyboard

# 覆盖率 (需要 tarpaulin)
cargo tarpaulin --out Html --output-dir target/tarpaulin

# 基准测试
cargo test --release -- --nocapture
```

### 6.2 CI 集成

```yaml
# .github/workflows/test.yml
test:
  runs-on: [ubuntu-latest, macos-latest]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo test --all-features
    - run: cargo clippy -- -D warnings
    - run: cargo fmt --check
```

---

## 7. 测试数据

### 7.1 测试仓库结构

```
tests/fixtures/repos/
├── repo1/          # 普通仓库
│   └── .git/
├── repo2/          # 有修改的仓库
│   └── .git/
├── not-a-repo/     # 非仓库目录
├── file.txt        # 文件（应被忽略）
└── nested/
    └── inner-repo/ # 嵌套仓库（不应被发现）
        └── .git/
```

### 7.2 Mock 配置

```toml
# tests/fixtures/config/valid.toml
main_directory = "/tmp/test-repos"

[editors]
webstorm = "webstorm"
vscode = "code"

[ui]
theme = "dark"
```

---

## 8. 进度跟踪

| 模块 | 当前覆盖 | 目标 | 状态 |
|------|---------|------|------|
| config::validators | ~60% | 90% | 🟡 进行中 |
| repo::discover | ~70% | 90% | 🟡 进行中 |
| app::model | ~50% | 85% | 🟡 进行中 |
| handler::keyboard | ~40% | 85% | 🔴 待开始 |
| app::update | ~10% | 80% | 🔴 待开始 |
| action::types | 100% | 90% | ✅ 完成 |
| error | ~60% | 80% | 🟡 进行中 |

### 集成测试进度

| 测试 | 状态 |
|------|------|
| directory_selection.rs | 🔴 待创建 |
| search_filtering.rs | 🔴 待创建 |
| keyboard_navigation.rs | 🔴 待创建 |
| repo_list_rendering.rs | 🔴 待创建 |

---

## 9. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| UI 测试不稳定 | 中 | 使用 TestBackend，避免真实终端 |
| 异步测试复杂度 | 中 | 使用 tokio::test，单线程运行时 |
| 文件系统依赖 | 低 | 使用 tempfile，自动清理 |
| 覆盖率不达标 | 低 | 优先覆盖核心路径，边缘情况后续补充 |

---

## 10. 验收标准

### 10.1 单元测试

- [ ] 所有 P0 模块覆盖 ≥80%
- [ ] 测试通过 `cargo test --lib`
- [ ] 无警告 (`cargo test 2>&1 | grep -i warning` 为空)

### 10.2 集成测试

- [ ] 4 个核心场景测试通过
- [ ] 测试执行时间 <10 秒
- [ ] 无竞态条件 (运行 10 次结果一致)

### 10.3 代码质量

- [ ] `cargo clippy -- -D warnings` 通过
- [ ] `cargo fmt --check` 通过
- [ ] 测试代码有文档注释

---

## 附录

### A. 参考文档

- PRD v2 第7章: `docs/ghclone-prd-v2.md`
- Ratatui 测试文档: https://ratatui.rs/recipes/testing/testing/
- Rust 测试指南: https://doc.rust-lang.org/book/ch11-00-testing.html

### B. 相关命令速查

```bash
# 运行单个测试
cargo test test_apply_filter -- --nocapture

# 显示测试输出
cargo test -- --nocapture

# 仅运行失败测试
cargo test -- --failed

# 并发运行
cargo test -- --test-threads=4
```
