# 多主目录管理功能测试策略文档

**文档版本**: 1.0  
**创建日期**: 2026-03-08  
**状态**: 待实施  
**关联**: 主目录管理  
**优先级**: P0

---

---

## 1. 测试策略概述

### 1.1 测试目标

本测试策略旨在验证多主目录管理功能的正确性、稳定性和性能，确保用户可以：

- 配置和管理多个主目录
- 添加和移除单个仓库
- 跨多个主目录搜索和过滤仓库
- 通过键盘快捷键流畅操作

### 1.2 测试范围

| 模块 | 测试范围 | 优先级 |
|------|---------|--------|
| 配置层 (`config`) | 多目录配置加载/保存、向后兼容 | P0 |
| 仓库发现 (`repo::discover`) | 多目录扫描、去重、合并 | P0 |
| 主目录管理 (`app::model`) | 添加/移除主目录、列表管理 | P0 |
| 目录选择器 (`ui::widgets::dir_chooser`) | 主目录/单个仓库模式 | P0 |
| 键盘处理 (`handler::keyboard`) | m/a/d 键操作 | P0 |
| UI 渲染 (`ui::render`) | 列表显示、状态栏、Scope 颜色 | P1 |

### 1.3 测试类型

```
                    ╱╲╱╲
                   ╱E2E╲           5-8 个场景
                  ╱─────╲         用户完整流程
                 ╱─────────╲
                ╱  集成测试  ╲     15-20 个测试用例
               ╱─────────────╲    模块间交互
              ╱─────────────────╲
             ╱    单元测试        ╲   80+ 测试用例
            ╱─────────────────────╲  覆盖率 ≥80%
```

| 测试类型 | 数量目标 | 覆盖率目标 |
|---------|---------|-----------|
| 单元测试 | 80+ | ≥ 80% |
| 集成测试 | 15-20 | 核心路径 100% |
| E2E 测试 | 5-8 | 用户场景覆盖 |

---

## 2. 测试用例清单

### 2.1 配置层测试

#### TC-CFG-001: 多主目录配置加载
- **前置条件**: 配置文件存在多个主目录
- **步骤**:
  1. 创建包含 `main_directories` 数组的配置文件
  2. 调用 `load_config()`
- **预期结果**: 成功加载所有主目录路径

#### TC-CFG-002: 旧配置向后兼容
- **前置条件**: 配置文件仅包含 `main_directory` 字段
- **步骤**:
  1. 创建旧格式配置文件
  2. 调用 `load_config()`
- **预期结果**: 自动迁移到新格式，`main_directories` 包含原路径

#### TC-CFG-003: 配置保存
- **前置条件**: 应用已配置多个主目录
- **步骤**:
  1. 添加多个主目录到配置
  2. 调用 `save_config()`
- **预期结果**: 配置文件中正确保存 `main_directories` 数组

#### TC-CFG-004: 空主目录列表验证
- **前置条件**: 配置文件存在但主目录列表为空
- **步骤**:
  1. 创建空 `main_directories` 配置
  2. 调用 `load_config()`
- **预期结果**: 返回验证错误，触发目录选择器

#### TC-CFG-005: 无效路径验证
- **前置条件**: 配置包含不存在的路径
- **步骤**:
  1. 创建包含不存在路径的配置
  2. 调用 `validate_config()`
- **预期结果**: 返回路径验证错误

### 2.2 主目录管理功能测试

#### TC-MD-001: 添加主目录
- **前置条件**: 应用处于 Running 状态
- **步骤**:
  1. 按 `m` 键进入主目录管理
  2. 按 `a` 键打开目录选择器
  3. 使用 `↑/↓` 键导航到目标目录
  4. 按 `Enter` 或 `Space` 进入目录（可选）
  5. 按 `Enter` 确认选择当前目录
  6. 验证主目录已添加
- **预期结果**: 新主目录添加到列表，重新扫描仓库，显示在主目录管理界面

#### TC-MD-002: 添加重复主目录
- **前置条件**: 主目录列表已包含某路径
- **步骤**:
  1. 尝试添加已存在的主目录
- **预期结果**: 显示错误提示，不重复添加

#### TC-MD-003: 移除主目录
- **前置条件**: 存在多个主目录
- **步骤**:
  1. 按 `m` 键进入主目录管理
  2. 使用 `↑/↓` 键选择要移除的主目录
  3. 按 `Enter` 键打开操作菜单
  4. 按 `d` 键选择"移除此目录"
  5. 在确认提示时按 `y` 确认移除
- **预期结果**: 主目录从列表移除，相关仓库不再显示，返回主目录管理界面

#### TC-MD-004: 移除最后一个主目录
- **前置条件**: 仅有一个主目录
- **步骤**:
  1. 尝试移除唯一的主目录
- **预期结果**: 显示警告提示，要求至少保留一个主目录

#### TC-MD-005: 主目录列表导航
- **前置条件**: 存在多个主目录
- **步骤**:
  1. 按 `m` 键进入主目录管理
  2. 使用 ↑↓ 键导航
  3. 验证循环导航（第一个按 ↑ 到最后一个，反之亦然）
- **预期结果**: 导航正常，支持循环

#### TC-MD-006: 主目录搜索过滤
- **前置条件**: 存在多个主目录
- **步骤**:
  1. 在主目录管理界面输入搜索词
  2. 验证过滤结果
- **预期结果**: 正确过滤匹配的主目录

#### TC-MD-007: 并发操作测试
- **前置条件**: 正在执行主目录扫描
- **步骤**:
  1. 启动应用，开始扫描多个主目录
  2. 在扫描过程中按 `m` 键进入主目录管理
  3. 尝试添加新主目录或移除现有主目录
- **预期结果**: 操作被排队或拒绝，扫描完成后状态一致，无竞态条件

#### TC-MD-008: 取消添加主目录
- **前置条件**: 已进入目录选择器（添加主目录模式）
- **步骤**:
  1. 在主目录管理界面按 `a` 键打开选择器
  2. 导航到某个目录
  3. 按 `Esc` 键取消
- **预期结果**: 返回主目录管理界面，未添加新目录

#### TC-MD-009: 取消移除主目录
- **前置条件**: 在主目录管理界面，已选择主目录
- **步骤**:
  1. 选择要移除的主目录
  2. 按 `d` 键（或 `Enter` 打开操作菜单后选择移除）
  3. 在确认提示时按 `n` 或 `Esc` 取消
- **预期结果**: 主目录保留在列表中，未被移除

### 2.3 单个仓库添加测试

#### TC-SR-001: 添加有效 Git 仓库
- **前置条件**: 目录包含 `.git` 文件夹
- **步骤**:
  1. 按 `a` 键添加仓库
  2. 选择单个仓库模式
  3. 导航到有效仓库
  4. 确认添加
- **预期结果**: 仓库添加到列表，显示格式为 `@scope/repo-name`

#### TC-SR-002: 添加非 Git 目录
- **前置条件**: 目录不包含 `.git`
- **步骤**:
  1. 尝试添加非 Git 目录
- **预期结果**: 显示错误，要求选择有效的 Git 仓库

#### TC-SR-003: 添加已存在的仓库
- **前置条件**: 仓库已存在于列表中
- **步骤**:
  1. 尝试添加已存在的仓库
- **预期结果**: 显示提示，自动去重

#### TC-SR-004: 验证 Scope 提取
- **前置条件**: 仓库路径包含多层目录
- **步骤**:
  1. 添加路径如 `/home/user/projects/company/app` 的仓库
  2. 验证显示格式
- **预期结果**: 正确提取 scope 为 `company`，显示 `@company/app`

### 2.3.5 安全测试

#### TC-SEC-001: 路径遍历防护测试
- **前置条件**: 应用正常运行
- **步骤**:
  1. 尝试添加包含 `../` 或绝对路径 `/etc/passwd` 的主目录
  2. 验证应用行为
- **预期结果**: 拒绝添加，显示路径无效错误，不访问预期之外的目录

#### TC-SEC-002: 符号链接安全测试
- **前置条件**: 系统中存在指向敏感目录的符号链接
- **步骤**:
  1. 创建指向 `/etc` 或用户主目录外的符号链接
  2. 尝试将该链接添加为主目录或单个仓库
- **预期结果**: 根据配置 `allow_symlinks` 处理，默认拒绝访问符号链接目标

### 2.4 仓库发现测试

#### TC-RD-001: 多主目录扫描
- **前置条件**: 配置了两个主目录，各包含仓库
- **步骤**:
  1. 调用仓库发现功能
  2. 验证扫描结果
- **预期结果**: 发现所有主目录下的仓库

#### TC-RD-002: 跨目录去重
- **前置条件**: 两个主目录包含同名仓库
- **步骤**:
  1. 扫描多个主目录
  2. 验证结果中无重复
- **预期结果**: 自动去重，保留第一个发现的实例

#### TC-RD-003: 单个仓库与主目录合并
- **前置条件**: 配置了主目录和单个仓库
- **步骤**:
  1. 扫描主目录
  2. 合并单个仓库列表
- **预期结果**: 正确合并，无重复

#### TC-RD-004: 大量仓库性能
- **前置条件**: 主目录包含 1000+ 仓库
- **步骤**:
  1. 执行仓库发现
  2. 测量时间
- **预期结果**: 在 2 秒内完成

### 2.5 目录选择器测试

#### TC-DC-001: 主目录选择模式
- **前置条件**: 调用添加主目录
- **步骤**:
  1. 验证选择器标题和提示
  2. 选择目录
  3. 确认选择
- **预期结果**: 允许选择任何目录（不限于 Git 仓库）

#### TC-DC-002: 单个仓库选择模式
- **前置条件**: 调用添加单个仓库
- **步骤**:
  1. 验证选择器标题和提示
  2. 导航到 Git 仓库
  3. 确认选择
- **预期结果**: 仅允许选择有效的 Git 仓库

#### TC-DC-003: Esc 返回逻辑
- **前置条件**: 在目录选择器界面
- **步骤**:
  1. 从主目录管理界面按 `a` 键进入选择器（添加主目录模式）
  2. 按 `Esc` 键
- **预期结果**: 返回到主目录管理界面（不返回到仓库列表主界面）
- **说明**: 选择器的返回目标取决于进入时的状态：从主目录管理进入则返回主目录管理，从仓库列表进入则返回仓库列表

#### TC-DC-004: 路径验证
- **前置条件**: 选择器打开
- **步骤**:
  1. 尝试访问无权限目录
  2. 尝试访问不存在的路径
- **预期结果**: 显示适当的错误信息

### 2.6 UI 显示测试

#### TC-UI-001: 仓库列表显示格式
- **前置条件**: 仓库来自不同主目录
- **步骤**:
  1. 查看仓库列表
  2. 验证显示格式
- **预期结果**: 显示 `@scope/repo-name` 格式

#### TC-UI-002: 状态栏路径显示
- **前置条件**: 选中某个仓库
- **步骤**:
  1. 查看状态栏
- **预期结果**: 显示完整路径或相对路径

#### TC-UI-003: Scope 颜色区分
- **前置条件**: 仓库来自不同 scope
- **步骤**:
  1. 查看仓库列表
  2. 验证不同 scope 的颜色
- **预期结果**: 不同 scope 使用不同颜色

#### TC-UI-004: 空状态显示
- **前置条件**: 无主目录或所有主目录为空
- **步骤**:
  1. 查看主界面
- **预期结果**: 显示友好的空状态提示

### 2.7 键盘操作测试

#### TC-KB-001: m 键进入主目录管理
- **前置条件**: 应用处于 Running 状态
- **步骤**:
  1. 按 `m` 键
- **预期结果**: 进入主目录管理界面

#### TC-KB-002: a 键添加（主目录管理模式）
- **前置条件**: 在主目录管理界面
- **步骤**:
  1. 按 `a` 键
- **预期结果**: 打开目录选择器（主目录模式）

#### TC-KB-003: a 键添加（主界面）
- **前置条件**: 在主界面
- **步骤**:
  1. 按 `a` 键
- **预期结果**: 显示菜单选择添加主目录或单个仓库

#### TC-KB-004: d 键移除主目录
- **前置条件**: 在主目录管理界面，已选择主目录
- **步骤**:
  1. 按 `d` 键
- **预期结果**: 提示确认后移除选中的主目录

#### TC-KB-005: 快捷键冲突检查
- **前置条件**: 各功能界面
- **步骤**:
  1. 验证所有快捷键无冲突
- **预期结果**: 每个键在不同上下文中有明确的功能

---

## 3. 单元测试计划

### 3.1 `config::types` - 配置类型扩展

**文件**: `src/config/types.rs`

| 测试函数 | 场景 | 预期结果 |
|---------|------|---------|
| `test_config_with_multiple_directories` | 多主目录配置 | 正确解析 `main_directories` |
| `test_config_backward_compatibility` | 旧配置格式 | 自动迁移到新格式 |
| `test_config_main_directories_default` | 默认配置 | `main_directories` 为空 Vec |
| `test_config_serialize_multi_dir` | 序列化多目录 | TOML 包含数组格式 |
| `test_config_deserialize_multi_dir` | 反序列化多目录 | 正确解析数组 |

**边界条件**:
- 空数组 `main_directories = []`
- 单元素数组 `main_directories = ["/path"]`
- 包含空字符串的数组
- 包含重复路径的数组

### 3.2 `config::validators` - 配置验证扩展

**文件**: `src/config/validators.rs`

| 测试函数 | 场景 | 预期结果 |
|---------|------|---------|
| `test_validate_multiple_directories_all_valid` | 所有路径有效 | Ok |
| `test_validate_multiple_directories_one_invalid` | 一个路径无效 | Err |
| `test_validate_multiple_directories_empty` | 空列表 | Err(EmptyMainDirectories) |
| `test_validate_multiple_directories_duplicates` | 重复路径 | Err(DuplicatePaths) |
| `test_validate_directory_not_in_home_multi` | 路径在 home 外 | Err |

**新增验证函数**:
```rust
fn validate_main_directories(dirs: &[PathBuf]) -> Result<(), ConfigError>
fn check_duplicate_paths(dirs: &[PathBuf]) -> bool
```

### 3.3 `repo::discover` - 多目录发现

**文件**: `src/repo/discover.rs`

| 测试函数 | 场景 | 预期结果 |
|---------|------|---------|
| `test_discover_multiple_directories` | 两个主目录 | 合并所有仓库 |
| `test_discover_with_deduplication` | 目录间有重复 | 去重后返回 |
| `test_discover_empty_directories` | 所有目录为空 | 空 Vec |
| `test_discover_single_and_multi_merge` | 单个仓库 + 主目录 | 正确合并 |
| `test_discover_preserves_scope` | 不同 scope | 保留路径信息 |

**新增函数**:
```rust
pub fn discover_repositories_multi(
    main_dirs: &[PathBuf],
    single_repos: &[PathBuf]
) -> Result<Vec<Repository>, RepoError>

fn deduplicate_repositories(repos: Vec<Repository>) -> Vec<Repository>
fn merge_repositories(
    from_dirs: Vec<Repository>,
    from_single: Vec<Repository>
) -> Vec<Repository>
```

### 3.4 `app::model` - 主目录管理

**文件**: `src/app/model.rs`

| 测试函数 | 场景 | 预期结果 |
|---------|------|---------|
| `test_add_main_directory` | 添加新主目录 | 列表更新，触发扫描 |
| `test_add_duplicate_main_directory` | 添加重复路径 | 忽略，返回错误 |
| `test_remove_main_directory` | 移除存在的路径 | 列表更新，移除相关仓库 |
| `test_remove_last_main_directory` | 移除最后一个 | 拒绝操作 |
| `test_main_directories_navigation` | 导航主目录列表 | 索引正确更新 |
| `test_add_single_repo` | 添加单个仓库 | 添加到单独列表 |
| `test_remove_single_repo` | 移除单个仓库 | 从列表移除 |
| `test_get_repo_scope` | 获取仓库 scope | 正确提取父目录名 |

**新增方法**:
```rust
impl App {
    pub fn add_main_directory(&mut self, path: PathBuf) -> Result<(), AppError>
    pub fn remove_main_directory(&mut self, index: usize) -> Result<(), AppError>
    pub fn add_single_repo(&mut self, path: PathBuf) -> Result<(), AppError>
    pub fn remove_single_repo(&mut self, index: usize) -> Result<(), AppError>
    pub fn get_main_directories(&self) -> &[PathBuf]
    pub fn get_single_repos(&self) -> &[PathBuf]
}
```

### 3.5 `ui::widgets::dir_chooser` - 目录选择器扩展

**文件**: `src/ui/widgets/dir_chooser.rs`

| 测试函数 | 场景 | 预期结果 |
|---------|------|---------|
| `test_chooser_mode_main_directory` | 主目录模式 | 允许选择任何目录 |
| `test_chooser_mode_single_repo` | 单个仓库模式 | 仅允许 Git 仓库 |
| `test_chooser_validate_git_repo` | 验证 Git 仓库 | 正确识别 `.git` |
| `test_chooser_filter_entries_multi` | 过滤条目 | 根据模式过滤 |

### 3.6 `handler::keyboard` - 键盘处理扩展

**文件**: `src/handler/keyboard.rs`

| 测试函数 | 场景 | 预期结果 |
|---------|------|---------|
| `test_handle_m_key_running` | 按 `m` 键 | 进入主目录管理 |
| `test_handle_m_key_managing_dirs` | 在主目录管理按 `m` | 返回主界面 |
| `test_handle_a_key_running` | 按 `a` 键 | 显示添加菜单 |
| `test_handle_a_key_managing` | 在主目录管理按 `a` | 添加主目录 |
| `test_handle_d_key_managing` | 按 `d` 键 | 移除选中的主目录 |
| `test_handle_esc_in_chooser` | 在选择器按 Esc | 返回主目录管理 |

---

## 4. 集成测试计划

### 4.1 配置加载与仓库发现集成

**文件**: `tests/integration/multi_config_loading.rs`

```rust
#[tokio::test]
async fn test_multi_directory_config_loading() {
    // 1. 创建临时配置目录
    // 2. 写入多主目录配置
    // 3. 加载配置
    // 4. 验证所有主目录被正确加载
    // 5. 触发仓库发现
    // 6. 验证所有仓库被发现
}

#[tokio::test]
async fn test_backward_compatibility_migration() {
    // 1. 创建旧格式配置
    // 2. 加载配置
    // 3. 验证自动迁移
    // 4. 验证配置已保存为新格式
}

#[tokio::test]
async fn test_config_save_and_reload() {
    // 1. 创建应用实例
    // 2. 添加多个主目录
    // 3. 保存配置
    // 4. 重新加载
    // 5. 验证配置持久化
}
```

### 4.2 主目录管理流程集成

**文件**: `tests/integration/main_directory_management.rs`

```rust
#[tokio::test]
async fn test_add_main_directory_flow() {
    // 1. 启动应用
    // 2. 按 'm' 进入主目录管理
    // 3. 按 'a' 添加
    // 4. 选择目录
    // 5. 验证主目录已添加
    // 6. 验证仓库已扫描
}

#[tokio::test]
async fn test_remove_main_directory_flow() {
    // 1. 配置两个主目录
    // 2. 进入主目录管理
    // 3. 选择第二个主目录
    // 4. 按 'd' 移除
    // 5. 验证主目录已移除
    // 6. 验证相关仓库已移除
}

#[tokio::test]
async fn test_prevent_remove_last_directory() {
    // 1. 配置单个主目录
    // 2. 尝试移除
    // 3. 验证操作被拒绝
    // 4. 验证错误提示
}
```

### 4.3 单个仓库管理集成

**文件**: `tests/integration/single_repo_management.rs`

```rust
#[tokio::test]
async fn test_add_single_repo_flow() {
    // 1. 启动应用
    // 2. 按 'a' 显示菜单
    // 3. 选择"添加单个仓库"
    // 4. 导航到 Git 仓库
    // 5. 确认添加
    // 6. 验证仓库已添加并显示正确格式
}

#[tokio::test]
async fn test_single_repo_validation() {
    // 1. 尝试添加非 Git 目录
    // 2. 验证错误提示
    // 3. 验证无法确认
}

#[tokio::test]
async fn test_scope_extraction() {
    // 1. 添加不同 scope 的仓库
    // 2. 验证显示格式为 @scope/name
}
```

### 4.4 仓库发现与去重集成

**文件**: `tests/integration/repo_discovery_dedup.rs`

```rust
#[tokio::test]
async fn test_multi_directory_discovery() {
    // 1. 创建两个主目录，各含仓库
    // 2. 执行仓库发现
    // 3. 验证所有仓库被发现
}

#[tokio::test]
async fn test_cross_directory_deduplication() {
    // 1. 创建两个主目录含同名仓库
    // 2. 执行仓库发现
    // 3. 验证结果中去重
}

#[tokio::test]
async fn test_single_repo_merge() {
    // 1. 配置主目录和单个仓库
    // 2. 执行发现
    // 3. 验证正确合并
}
```

### 4.5 键盘操作集成

**文件**: `tests/integration/keyboard_multi_dir.rs`

```rust
#[tokio::test]
async fn test_m_key_toggle_management() {
    // 1. 在主界面按 'm'
    // 2. 验证进入主目录管理
    // 3. 再按 'm'
    // 4. 验证返回主界面
}

#[tokio::test]
async fn test_a_key_context_menu() {
    // 1. 在主界面按 'a'
    // 2. 验证显示菜单
    // 3. 选择选项
    // 4. 验证正确响应
}

#[tokio::test]
async fn test_directory_list_navigation() {
    // 1. 进入主目录管理
    // 2. 测试 ↑↓ 导航
    // 3. 测试循环（第一个 ↑ 到最后）
}
```

### 4.6 UI 渲染集成

**文件**: `tests/integration/ui_multi_dir.rs`

```rust
#[tokio::test]
async fn test_repo_list_format_with_scope() {
    // 1. 创建 Mock 终端
    // 2. 加载多 scope 仓库
    // 3. 渲染 UI
    // 4. 验证显示格式 @scope/name
}

#[tokio::test]
async fn test_scope_color_differentiation() {
    // 1. 加载不同 scope 仓库
    // 2. 渲染 UI
    // 3. 验证不同 scope 颜色不同
}

#[tokio::test]
async fn test_status_bar_path_display() {
    // 1. 选择仓库
    // 2. 验证状态栏显示正确路径
}
```

---

## 5. E2E 测试计划

### 5.1 完整用户场景

#### E2E-001: 首次使用配置多主目录
```gherkin
场景: 新用户配置多个主目录
  给定 用户首次启动应用
  当 用户完成初始配置
  并且 添加第二个主目录
  并且 添加单个仓库
  那么 所有仓库应在列表中显示
  并且 重启后配置应保留
```

**测试实现**:
```rust
#[tokio::test]
async fn e2e_first_time_setup_multi_directory() {
    // 1. 启动无配置应用
    // 2. 选择第一个主目录
    // 3. 验证仓库列表显示
    // 4. 按 'm' 进入管理
    // 5. 按 'a' 添加第二个主目录
    // 6. 验证新仓库出现
    // 7. 按 'a' 添加单个仓库
    // 8. 验证格式正确
    // 9. 退出并重启
    // 10. 验证配置持久化
}
```

#### E2E-002: 日常多目录工作流程
```gherkin
场景: 日常开发工作流程
  给定 用户已配置多个主目录
  当 用户搜索跨目录的仓库
  并且 用户收藏某仓库
  并且 用户切换视图
  那么 搜索应跨所有目录
  并且 收藏应正确显示
```

**测试实现**:
```rust
#[tokio::test]
async fn e2e_daily_workflow() {
    // 1. 启动已配置多目录的应用
    // 2. 搜索仓库（验证跨目录）
    // 3. 按 'f' 收藏
    // 4. 按 Ctrl+f 切换到收藏视图
    // 5. 验证显示正确
    // 6. 按 Ctrl+r 切换到最近视图
    // 7. 打开仓库操作
}
```

#### E2E-003: 主目录维护
```gherkin
场景: 维护主目录列表
  给定 用户有多个主目录
  当 用户移除一个主目录
  并且 添加新的主目录
  那么 被移除的仓库应消失
  并且 新目录的仓库应出现
```

### 5.2 键盘操作流程

#### E2E-KB-001: 完整键盘操作流程

| 步骤 | 按键 | 预期状态 |
|-----|------|---------|
| 1 | `m` | 进入主目录管理 |
| 2 | `↓` | 选中第二个主目录 |
| 3 | `d` | 提示确认移除 |
| 4 | `Enter` | 确认，主目录移除 |
| 5 | `a` | 打开目录选择器 |
| 6 | `↓`/`Enter` | 选择目录 |
| 7 | `Space` | 确认添加 |
| 8 | `Esc` | 返回主界面 |
| 9 | `a` | 显示添加菜单 |
| 10 | `↓`/`Enter` | 选择"添加单个仓库" |
| 11 | 导航 | 选择 Git 仓库 |
| 12 | `Enter` | 确认添加 |

### 5.3 验收标准

| 场景 | 验收标准 |
|-----|---------|
| 多主目录配置 | 可以配置 ≥3 个主目录，重启后保留 |
| 添加主目录 | 添加后主目录立即可见，仓库自动扫描 |
| 移除主目录 | 移除后主目录和相关仓库消失 |
| 添加单个仓库 | 添加后显示格式为 @scope/repo-name |
| 跨目录搜索 | 搜索词匹配所有目录的仓库 |
| 去重 | 同名仓库只显示一次 |
| 性能 | 1000 仓库加载时间 < 2 秒 |

---

## 6. 测试数据准备

### 6.1 模拟目录结构

```
tests/fixtures/multi_dir/
├── home/
│   └── user/
│       ├── projects/
│       │   ├── company1/
│       │   │   ├── app1/          # Git repo
│       │   │   │   └── .git/
│       │   │   ├── app2/          # Git repo
│       │   │   │   └── .git/
│       │   │   └── docs/          # Non-git
│       │   └── company2/
│       │       ├── service1/      # Git repo
│       │       │   └── .git/
│       │       └── service2/      # Git repo
│       │           └── .git/
│       └── personal/
│           ├── dotfiles/          # Git repo
│           │   └── .git/
│           └── experiments/       # Git repo
│               └── .git/
└── work/
    └── repos/
        ├── shared-lib/            # Git repo (与 projects/company1/app1 同名)
        │   └── .git/
        └── tools/                 # Git repo
            └── .git/
```

### 6.2 配置文件示例

**多主目录配置** (`tests/fixtures/config/multi_dir.toml`):
```toml
version = "2.0"
main_directories = [
    "/home/user/projects",
    "/home/user/personal",
    "/work/repos"
]
single_repos = [
    "/opt/external/repo"
]

[editors]
webstorm = "webstorm"
vscode = "code"

[ui]
theme = "dark"
show_git_status = true
show_branch = true

[security]
allow_symlinks = false
max_search_depth = 2
```

**旧配置格式** (`tests/fixtures/config/old_format.toml`):
```toml
version = "1.0"
main_directory = "/home/user/projects"

[editors]
vscode = "code"
```

**迁移后配置** (`tests/fixtures/config/migrated.toml`):
```toml
version = "2.0"
main_directories = ["/home/user/projects"]
single_repos = []
# ... 其他字段
```

### 6.3 Git 仓库模拟

**使用 MockFs 创建**:
```rust
use tests::helpers::mock_fs::MockFs;

fn setup_multi_directory_fixture() -> MockFs {
    let mock = MockFs::new();
    
    // 主目录 1
    mock.create_nested_repos("projects/company1", &["app1", "app2"]);
    mock.create_non_repo_dir("projects/company1/docs");
    mock.create_nested_repos("projects/company2", &["service1", "service2"]);
    
    // 主目录 2
    mock.create_nested_repos("personal", &["dotfiles", "experiments"]);
    
    // 主目录 3
    mock.create_nested_repos("work/repos", &["shared-lib", "tools"]);
    
    // 单个仓库（在 mock fs 外）
    // 使用绝对路径模拟
    
    mock
}
```

---

## 7. 性能测试

### 7.1 大目录扫描性能

**文件**: `benches/discovery_benchmark.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_discover_100_repos(c: &mut Criterion) {
    c.bench_function("discover_100_repos", |b| {
        let temp_dir = setup_temp_repos(100);
        b.iter(|| {
            discover_repositories(black_box(temp_dir.path()))
        });
    });
}

fn bench_discover_1000_repos(c: &mut Criterion) {
    c.bench_function("discover_1000_repos", |b| {
        let temp_dir = setup_temp_repos(1000);
        b.iter(|| {
            discover_repositories_multi(
                black_box(&[temp_dir.path().join("dir1"), temp_dir.path().join("dir2")]),
                black_box(&[])
            )
        });
    });
}

fn bench_multi_directory_merge(c: &mut Criterion) {
    c.bench_function("merge_5_dirs_with_dedup", |b| {
        let repos = generate_test_repos(500);
        b.iter(|| {
            deduplicate_repositories(black_box(repos.clone()))
        });
    });
}
```

**性能目标**:

| 场景 | 目标时间 | 最大时间 |
|-----|---------|---------|
| 扫描 100 个仓库 | < 100ms | 200ms |
| 扫描 1000 个仓库 | < 1s | 2s |
| 合并 5 个目录（去重） | < 50ms | 100ms |
| 加载配置（5 主目录） | < 10ms | 50ms |

### 7.2 内存使用测试

```rust
#[test]
fn test_memory_usage_large_scan() {
    // 使用 10000 仓库测试内存使用
    // 验证无内存泄漏
}
```

---

## 8. 测试执行计划

### 8.1 测试阶段

| 阶段 | 内容 | 时间 | 负责人 |
|-----|------|------|--------|
| Phase 1 | 单元测试开发 | 2 天 | 开发工程师 |
| Phase 2 | 集成测试开发 | 2 天 | 测试工程师 |
| Phase 3 | E2E 测试开发 | 1 天 | 测试工程师 |
| Phase 4 | 性能测试 | 1 天 | 性能工程师 |
| Phase 5 | 回归测试 | 1 天 | 全体 |

### 8.2 优先级矩阵

| 测试 | P0 | P1 | P2 |
|-----|----|----|----|
| 配置加载/保存 | ✅ | | |
| 向后兼容 | ✅ | | |
| 添加主目录 | ✅ | | |
| 移除主目录 | ✅ | | |
| 单个仓库添加 | ✅ | | |
| 多目录扫描 | ✅ | | |
| 去重逻辑 | ✅ | | |
| 键盘操作 | ✅ | | |
| UI 显示格式 | | ✅ | |
| Scope 颜色 | | ✅ | |
| 性能测试 | | ✅ | |
| 边界条件 | | | ✅ |

### 8.3 通过标准

#### 单元测试通过标准
- [ ] 所有 P0 模块覆盖率 ≥ 80%
- [ ] `cargo test --lib` 100% 通过
- [ ] 无编译警告
- [ ] Clippy 检查通过

#### 集成测试通过标准
- [ ] 15+ 集成测试用例通过
- [ ] 核心路径覆盖 100%
- [ ] 平均执行时间 < 5 秒
- [ ] 无竞态条件（运行 10 次一致）

#### E2E 测试通过标准
- [ ] 5+ E2E 场景通过
- [ ] 所有验收标准满足
- [ ] 键盘操作流程完整
- [ ] 内存使用正常

#### 性能测试通过标准
- [ ] 1000 仓库扫描 < 2 秒
- [ ] 内存使用 < 100MB
- [ ] 无内存泄漏

---

## 9. 测试辅助工具

### 9.1 MockFs 扩展

**文件**: `tests/helpers/mock_fs.rs`

```rust
impl MockFs {
    /// 创建多主目录结构
    pub fn create_multi_directory_structure(&self) -> Vec<PathBuf>;
    
    /// 创建带 scope 的仓库
    pub fn create_repo_with_scope(&self, scope: &str, name: &str) -> PathBuf;
    
    /// 获取所有创建的仓库路径
    pub fn get_all_repo_paths(&self) -> Vec<PathBuf>;
    
    /// 创建重复仓库（用于测试去重）
    pub fn create_duplicate_repos(&self, name: &str, count: usize) -> Vec<PathBuf>;
}
```

### 9.2 配置测试工具

**文件**: `tests/helpers/config_helper.rs`

```rust
pub fn create_temp_config_with_dirs(dirs: &[&str]) -> (TempDir, PathBuf);
pub fn create_old_format_config() -> (TempDir, PathBuf);
pub fn read_config_version(path: &Path) -> String;
pub fn assert_config_valid(path: &Path);
```

### 9.3 UI 测试工具

**文件**: `tests/helpers/ui_assertions.rs`

```rust
pub fn assert_repo_displayed(buffer: &Buffer, name: &str, scope: &str);
pub fn assert_scope_color(buffer: &Buffer, scope: &str, color: Color);
pub fn assert_path_in_status_bar(buffer: &Buffer, path: &str);
pub fn assert_main_directories_listed(buffer: &Buffer, dirs: &[&str]);
```

---

## 10. 风险与缓解

| 风险 | 影响 | 可能性 | 缓解措施 |
|-----|------|--------|---------|
| 配置迁移失败 | 高 | 中 | 备份旧配置，提供回滚机制 |
| 大规模目录扫描慢 | 中 | 高 | 实现异步扫描，进度指示 |
| 去重逻辑错误 | 高 | 低 | 充分的边界测试，路径规范化 |
| 键盘快捷键冲突 | 中 | 中 | 完整的热键映射文档，冲突检查 |
| 向后兼容问题 | 高 | 中 | 旧配置测试，灰度发布 |

---

## 11. 附录

### 11.1 参考文档

- [Phase 1 测试计划](./test-plan-phase1.md)
- [CLAUDE.md](../../CLAUDE.md) - 项目开发规范
- [Config 类型](../../src/config/types.rs)
- [Repo 发现](../../src/repo/discover.rs)

### 11.2 相关命令

```bash
# 运行所有测试
cargo test

# 运行多目录相关测试
cargo test multi_dir
cargo test main_directory
cargo test single_repo

# 运行集成测试
cargo test --test '*'

# 覆盖率报告
cargo tarpaulin --out Html --output-dir target/tarpaulin

# 性能测试
cargo bench

# 检查测试
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --check
```

### 11.3 测试检查清单

#### 开发前检查
- [ ] 阅读相关代码和文档
- [ ] 理解功能需求
- [ ] 识别边界条件
- [ ] 设计测试数据

#### 开发中检查
- [ ] 遵循 AAA 模式 (Arrange-Act-Assert)
- [ ] 使用 tempfile 创建临时资源
- [ ] 添加测试文档注释
- [ ] 保持测试独立性

#### 开发后检查
- [ ] 运行 `cargo test` 通过
- [ ] 检查测试覆盖率
- [ ] 运行 `cargo clippy` 无警告
- [ ] 更新此文档状态

---

**文档版本历史**:

| 版本 | 日期 | 修改者 | 变更内容 |
|-----|------|--------|---------|
| 1.0 | 2026-03-08 | Tester | 初始版本 |

**最后更新**: 2026-03-08
