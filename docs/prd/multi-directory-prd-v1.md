# PRD: 多主目录管理功能 - v1

**文档版本**: v1  
**创建日期**: 2026-03-08  
**状态**: 待审查  
**关联**: 主目录管理  
**作者**: Product Manager  

---

## 1. 产品概述

### 1.1 背景

当前 repotui 仅支持配置一个主目录来扫描 Git 仓库。用户在管理多个工作目录（如个人项目、公司项目、开源贡献等）时，需要频繁切换主目录或使用多个配置文件，体验不佳。

### 1.2 目标

- 支持配置多个主目录，统一扫描并展示所有仓库
- 支持将任意位置的单个 Git 仓库添加到列表
- 使用 `@scope/package-name` 格式展示仓库，提升可识别性
- 重构目录选择器为通用组件，支持多种使用场景

### 1.3 用户价值

| 用户场景 | 当前痛点 | 解决后价值 |
|---------|---------|-----------|
| 同时管理个人和公司项目 | 只能二选一或频繁切换 | 同时展示，无需切换 |
| 管理分散在各处的项目 | 需要手动维护多个配置 | 统一界面，集中管理 |
| 临时处理外部仓库 | 需要临时更改主目录 | 可直接添加单个仓库 |

---

## 2. 功能需求

### 2.1 主目录管理界面

#### 2.1.1 界面描述

新增"主目录管理"界面，用于管理主目录列表。界面风格与仓库列表保持一致。

```
┌──────────────────────────────────────────────────────────────────────┐
│  repotui v0.1.0                           ┌───┬───┬───┐              │
│                                           │ 🌙│ 🔍│ ❓│              │
└──────────────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────────────┐
│  主目录管理 (3)                                       搜索:          │
├──────────────────────────────────────────────────────────────────────┤
│ ▶ ~/projects/personal                                                │
│   ~/projects/company                                                 │
│   ~/opensource                                                       │
├──────────────────────────────────────────────────────────────────────┤
│ ↑/↓ 导航 | Enter 操作菜单 | / 搜索 | a 添加 | Esc 返回              │
└──────────────────────────────────────────────────────────────────────┘
```

#### 2.1.2 数据来源

- 主目录列表存储在配置文件中
- 支持手动添加、删除
- 初始版本支持空列表，后续扫描时自动发现仓库

#### 2.1.3 操作支持

| 按键 | 功能 | 备注 |
|------|------|------|
| ↑/↓ | 导航选择 | 循环移动 |
| Enter | 打开操作菜单 | 仅支持删除操作 |
| / | 搜索过滤 | 支持模糊匹配 |
| a | 添加主目录 | 打开目录选择器 |
| Esc | 返回仓库列表 | - |

#### 2.1.4 操作菜单

```
┌──────────────────────────────────────────────────────────────────────┐
│  操作菜单                                                            │
├──────────────────────────────────────────────────────────────────────┤
│  d - 移除此目录                                                      │
│  Esc - 取消                                                        │
└──────────────────────────────────────────────────────────────────────┘
```

**注意**: 主目录管理界面不支持收藏夹、历史记录、批量操作功能。

### 2.2 仓库列表界面变更

#### 2.2.1 按键变更

| 按键 | 原功能 | 新功能 | 备注 |
|------|--------|--------|------|
| a | (无) | 添加单个仓库 | 打开目录选择器 |
| m | 更改主目录 | 进入主目录管理 | 功能升级 |

#### 2.2.2 数据源变更

```
仓库来源 = 主目录扫描结果 + 手动添加的单个仓库
```

- 主目录列表中的每个目录递归扫描（遵循现有搜索深度限制）
- 手动添加的单个仓库直接加入列表
- 自动去重（相同路径的仓库只保留一个）

#### 2.2.3 显示格式

使用 `@scope/package-name` 格式显示仓库名：

```
┌──────────────────────────────────────────────────────────────────────┐
│  @personal/my-project                        main ✓                  │
│  @personal/dotfiles                          main ●                  │
│  @company/backend-api                        develop ●               │
│  @company/frontend-app                       feature/login ●         │
│  @stand/random-project                       main ✓                  │
└──────────────────────────────────────────────────────────────────────┘
```

**Scope 规则**：

| 仓库来源 | Scope 值 | 示例 |
|---------|---------|------|
| 主目录扫描 | 主目录文件夹名 | `~/projects/personal` → `@personal` |
| 手动添加 | `@stand` | `@stand/my-tool` |

**显示优先级**：
1. 显示格式：`@{scope}/{repo-name}`
2. 当仓库名与 scope 相同时，避免重复：`@{scope}/{scope}` → `@{scope}`

#### 2.2.4 状态栏增强

底部状态栏显示当前选中仓库的主目录路径：

```
┌──────────────────────────────────────────────────────────────────────┐
│  5/23 仓库 | 主目录: ~/projects/personal/my-project                  │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.3 目录选择器重构

#### 2.3.1 通用化设计

将现有的 `ChoosingDir` 状态重构为通用的目录选择器，支持两种模式：

| 模式 | 用途 | 确认后行为 |
|------|------|-----------|
| MainDirectory | 选择主目录 | 添加到主目录列表 |
| SingleRepo | 选择单个 Git 仓库 | 添加到仓库列表 |

#### 2.3.2 状态流转

```
主目录管理 --(a)--> 目录选择器(MainDirectory模式) --(确认)--> 主目录管理
仓库列表    --(a)--> 目录选择器(SingleRepo模式)   --(确认)--> 仓库列表
```

#### 2.3.3 Esc 行为

- 从主目录管理进入 → Esc 返回主目录管理
- 从仓库列表进入 → Esc 返回仓库列表

#### 2.3.4 Git 仓库验证（SingleRepo模式）

当选择器处于 SingleRepo 模式时：
- 选中目录包含 `.git` 子目录：允许确认
- 选中目录不包含 `.git`：显示警告，不允许确认

---

## 3. 用户故事

### 3.1 添加主目录

**作为** 开发者，我有多个项目目录需要管理  
**我希望** 能够在主目录管理界面添加新的主目录  
**以便** 统一查看所有项目的仓库

**验收标准**：
1. 按 `m` 键进入主目录管理界面
2. 按 `a` 键打开目录选择器
3. 选择目标目录后按 `Space` 或 `Enter` 确认
4. 新目录出现在主目录列表中
5. 返回仓库列表后，新目录下的仓库出现在列表中

### 3.2 移除主目录

**作为** 开发者，某个主目录不再使用  
**我希望** 能够从主目录列表中移除它  
**以便** 保持列表整洁

**验收标准**：
1. 在主目录管理界面选择要移除的目录
2. 按 `Enter` 打开操作菜单
3. 按 `d` 确认移除
4. 目录从列表中移除
5. 相关仓库不再出现在仓库列表中
6. **不移除**手动添加的单个仓库（即使路径在主目录下）

### 3.3 添加单个仓库

**作为** 开发者，我需要临时处理不在主目录下的仓库  
**我希望** 能够直接将单个仓库添加到列表  
**以便** 无需更改主目录配置

**验收标准**：
1. 在仓库列表界面按 `a` 键
2. 打开目录选择器（SingleRepo模式）
3. 导航到目标仓库目录
4. 确认时检查是否为有效 Git 仓库
5. 有效则添加到列表，显示为 `@stand/{repo-name}`

### 3.4 识别仓库来源

**作为** 开发者，我需要知道仓库来自哪个目录  
**我希望** 仓库名中包含来源标识  
**以便** 快速识别项目归属

**验收标准**：
1. 仓库显示格式为 `@scope/repo-name`
2. Scope 与主目录名一致
3. 单个添加的仓库显示为 `@stand`
4. 状态栏显示完整主目录路径

---

## 4. 验收标准

### 4.1 功能验收

| ID | 验收项 | 优先级 | 验证方式 |
|----|--------|--------|----------|
| AC-1 | 支持添加多个主目录 | P0 | 手动测试 |
| AC-2 | 支持移除主目录 | P0 | 手动测试 |
| AC-3 | 支持添加单个仓库 | P0 | 手动测试 |
| AC-4 | @scope/repo-name 格式显示 | P0 | 视觉检查 |
| AC-5 | 状态栏显示主目录路径 | P1 | 视觉检查 |
| AC-6 | 目录选择器支持两种模式 | P0 | 手动测试 |
| AC-7 | 移除主目录不影响手动添加的仓库 | P1 | 手动测试 |
| AC-8 | 支持搜索过滤主目录 | P1 | 手动测试 |

### 4.2 性能验收

| ID | 验收项 | 标准 |
|----|--------|------|
| AC-P1 | 主目录列表加载 | < 100ms |
| AC-P2 | 仓库扫描（5个主目录） | < 3s |
| AC-P3 | 搜索过滤响应 | < 50ms |

### 4.3 兼容性验收

| ID | 验收项 | 标准 |
|----|--------|------|
| AC-C1 | 配置文件向后兼容 | 支持读取 v1 配置 |
| AC-C2 | 迁移旧配置 | 自动将 main_directory 转为 main_directories[0] |
| AC-C3 | 最小终端尺寸 | 80x25 正常显示 |

---

## 5. 数据流与状态变更

### 5.1 数据流图

```
┌──────────────────────────────────────────────────────────────────────┐
│                          配置加载                                    │
└──────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────────┐
│  Config.main_directories: Vec<PathBuf>                               │
│  Config.single_repos: Vec<PathBuf>   (新增)                          │
└──────────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐       ┌─────────────────────────┐
│   主目录管理界面         │       │     仓库列表界面         │
│  - 显示 main_directories │       │  - 扫描所有主目录        │
│  - 支持增删             │       │  - 加载 single_repos     │
│  - 打开选择器           │       │  - 合并显示              │
└───────────┬─────────────┘       └───────────┬─────────────┘
            │                                 │
            └───────────┬─────────────────────┘
                        ▼
┌──────────────────────────────────────────────────────────────────────┐
│                      目录选择器 (通用组件)                            │
│  DirectoryChooserMode::MainDirectory 或 ::SingleRepo                │
└──────────────────────────────────────────────────────────────────────┘
```

### 5.2 状态机变更

```
现有状态机:
┌──────────┐   m键    ┌─────────────┐
│ Running  │────────▶│ ChoosingDir │
└──────────┘         └─────────────┘

新状态机:
┌──────────┐   m键    ┌─────────────────┐
│ Running  │────────▶│ ManagingDirs    │ (新增)
└──────────┘         └────────┬────────┘
                              │ a键
                              ▼
                    ┌─────────────────┐
                    │ DirectoryChooser│ (重构)
                    │  - mode: Mode   │
                    └─────────────────┘
                              │
                              │ a键 (Running状态)
                              ▼
                    ┌─────────────────┐
                    │ DirectoryChooser│
                    │  - mode: Single │
                    └─────────────────┘
```

### 5.3 核心状态变更

#### 新增 AppState 变体

```rust
/// 主目录管理状态
ManagingDirs {
    /// 主目录列表
    directories: Vec<PathBuf>,
    /// 过滤后索引
    filtered_indices: Vec<usize>,
    /// 选中索引
    selected_index: usize,
    /// 搜索查询
    search_query: String,
    /// 滚动偏移
    scroll_offset: usize,
}

/// 目录选择器模式
enum DirectoryChooserMode {
    /// 选择主目录
    MainDirectory,
    /// 选择单个 Git 仓库
    SingleRepo,
}

/// 重构后的 ChoosingDir
ChoosingDir {
    path: PathBuf,
    entries: Vec<String>,
    selected_index: usize,
    scroll_offset: usize,
    /// 新增：选择器模式
    mode: DirectoryChooserMode,
    /// 新增：返回目标
    return_to: Box<AppState>,
}
```

#### 状态优先级更新

```rust
impl AppState {
    pub fn priority(&self) -> u8 {
        match self {
            AppState::ShowingActions { .. } => 5,
            AppState::ShowingHelp { .. } => 4,
            AppState::ManagingDirs { .. } => 3,     // 新增
            AppState::ChoosingDir { .. } => 3,
            AppState::SelectingTheme { .. } => 3,
            AppState::Running => 1,
            AppState::Loading { .. } | AppState::Error { .. } => 0,
            AppState::Quit => 0,
        }
    }
}
```

### 5.4 新增消息类型

```rust
pub enum AppMsg {
    // === 主目录管理 ===
    /// 显示主目录管理界面
    ShowMainDirectoryManager,
    /// 关闭主目录管理界面
    CloseMainDirectoryManager,
    /// 添加主目录
    AddMainDirectory(PathBuf),
    /// 移除主目录
    RemoveMainDirectory(usize),
    /// 主目录导航
    MainDirNavDown,
    MainDirNavUp,
    /// 主目录搜索
    MainDirSearchInput(char),
    MainDirSearchBackspace,
    MainDirSearchClear,
    
    // === 单个仓库管理 ===
    /// 添加单个仓库
    AddSingleRepository(PathBuf),
    /// 移除单个仓库
    RemoveSingleRepository(PathBuf),
    
    // === 目录选择器 ===
    /// 显示目录选择器（指定模式）
    ShowDirectoryChooserWithMode {
        mode: DirectoryChooserMode,
        return_to: Box<AppState>,
    },
    /// 目录已验证（SingleRepo模式）
    DirectoryValidated(PathBuf),
    /// 目录验证失败
    DirectoryValidationFailed(String),
}
```

---

## 6. 错误处理

### 6.1 错误场景

| 场景 | 错误类型 | 处理方式 |
|------|---------|---------|
| 添加重复主目录 | 用户错误 | 显示提示："目录已存在" |
| 添加无效路径 | 用户错误 | 显示提示："无效路径" |
| 添加非 Git 目录为单仓库 | 用户错误 | 显示警告："非 Git 仓库" |
| 移除不存在的目录 | 系统错误 | 静默忽略，刷新列表 |
| 扫描主目录失败 | 系统错误 | 显示错误，继续其他目录 |
| 配置文件写入失败 | 系统错误 | 显示错误，保持内存状态 |

### 6.2 错误界面

```
┌──────────────────────────────────────────────────────────────────────┐
│  ⚠ 错误                                                             │
├──────────────────────────────────────────────────────────────────────┤
│  无法添加目录: /path/to/dir                                          │
│  原因: 目录已存在于主目录列表中                                      │
├──────────────────────────────────────────────────────────────────────┤
│  按 Enter 或 Esc 关闭                                               │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 7. 配置变更说明

### 7.1 配置结构变更

#### 当前配置 (v1)

```toml
version = "1.0"
main_directory = "/home/user/projects"
```

#### 新配置 (v2)

```toml
version = "2.0"

# 主目录列表（替代单个 main_directory）
main_directories = [
    "/home/user/projects/personal",
    "/home/user/projects/company",
    "/home/user/opensource"
]

# 手动添加的单个仓库
[[single_repositories]]
path = "/opt/tools/my-tool"
added_at = "2026-03-08T10:30:00Z"

[[single_repositories]]
path = "/tmp/experiment"
added_at = "2026-03-08T11:00:00Z"
```

### 7.2 向后兼容性

#### 配置迁移策略

```rust
/// 配置加载时自动迁移
impl Config {
    pub fn load_with_migration() -> Result<Self, ConfigError> {
        let config = Self::load()?;
        
        // 如果存在旧版 main_directory，迁移到 main_directories
        if !config.main_directory.as_os_str().is_empty() {
            config.main_directories.push(config.main_directory.clone());
            config.main_directory = PathBuf::new(); // 清空旧字段
            config.version = "2.0".to_string();
            config.save()?;
        }
        
        Ok(config)
    }
}
```

#### 序列化兼容性

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    
    /// 向后兼容：旧版单主目录
    #[serde(default)]
    pub main_directory: PathBuf,
    
    /// 新版：主目录列表
    #[serde(default)]
    pub main_directories: Vec<PathBuf>,
    
    /// 手动添加的单个仓库
    #[serde(default)]
    pub single_repositories: Vec<SingleRepository>,
    
    // ... 其他字段
}

/// 单个仓库记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleRepository {
    pub path: PathBuf,
    pub added_at: chrono::DateTime<chrono::Utc>,
}
```

---

## 8. 技术实现建议

### 8.1 配置结构变更建议

```rust
// src/config/types.rs

/// 主目录条目（带元数据）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainDirectory {
    pub path: PathBuf,
    pub alias: Option<String>,  // 可选别名，用于自定义 scope
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// 单个仓库条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleRepository {
    pub path: PathBuf,
    pub added_at: chrono::DateTime<chrono::Utc>,
    pub alias: Option<String>,  // 可选：覆盖 @stand
}

/// 扩展 Config 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    
    // 向后兼容
    #[serde(default)]
    pub main_directory: PathBuf,
    
    // 主目录列表
    #[serde(default)]
    pub main_directories: Vec<MainDirectory>,
    
    // 单个仓库列表
    #[serde(default)]
    pub single_repositories: Vec<SingleRepository>,
    
    // ... 其他字段
}

impl Config {
    /// 获取所有主目录路径
    pub fn get_main_directories(&self) -> Vec<PathBuf> {
        if !self.main_directory.as_os_str().is_empty() {
            // 向后兼容
            vec![self.main_directory.clone()]
        } else {
            self.main_directories.iter().map(|d| d.path.clone()).collect()
        }
    }
    
    /// 添加主目录
    pub fn add_main_directory(&mut self, path: PathBuf) -> Result<(), ConfigError> {
        // 检查重复
        if self.main_directories.iter().any(|d| d.path == path) {
            return Err(ConfigError::DuplicateDirectory);
        }
        
        self.main_directories.push(MainDirectory {
            path,
            alias: None,
            added_at: chrono::Utc::now(),
        });
        
        self.save()
    }
    
    /// 移除主目录
    pub fn remove_main_directory(&mut self, index: usize) -> Result<(), ConfigError> {
        if index < self.main_directories.len() {
            self.main_directories.remove(index);
            self.save()
        } else {
            Err(ConfigError::InvalidIndex)
        }
    }
}
```

### 8.2 Repository 结构扩展

```rust
// src/repo/types.rs

/// 仓库来源类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoSource {
    /// 来自主目录扫描
    MainDirectory { 
        /// 主目录索引
        dir_index: usize,
        /// Scope 名称（主目录文件夹名或别名）
        scope: String,
    },
    /// 手动添加的单个仓库
    Standalone { 
        /// 可选自定义 scope
        alias: Option<String>,
    },
}

/// 扩展 Repository 结构
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    pub name: String,
    pub path: PathBuf,
    pub last_modified: Option<SystemTime>,
    pub is_dirty: bool,
    pub branch: Option<String>,
    pub is_git_repo: bool,
    /// 新增：仓库来源
    pub source: RepoSource,
}

impl Repository {
    /// 获取显示名称（@scope/repo-name）
    pub fn display_name(&self) -> String {
        let scope = match &self.source {
            RepoSource::MainDirectory { scope, .. } => scope.clone(),
            RepoSource::Standalone { alias } => {
                alias.clone().unwrap_or_else(|| "stand".to_string())
            }
        };
        
        format!("@{}/{}", scope, self.name)
    }
    
    /// 获取主目录路径（如果是来自主目录）
    pub fn main_directory(&self) -> Option<&Path> {
        match &self.source {
            RepoSource::MainDirectory { .. } => {
                // 从 path 向上查找到主目录
                // 这需要额外的主目录列表引用
                None
            }
            RepoSource::Standalone { .. } => None,
        }
    }
}
```

### 8.3 状态机变更建议

```rust
// src/app/state.rs

/// 目录选择器模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryChooserMode {
    /// 选择主目录
    MainDirectory,
    /// 选择单个 Git 仓库
    SingleRepo,
}

/// 应用状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    // 现有状态...
    
    /// 新增：主目录管理
    ManagingDirs {
        /// 主目录列表
        directories: Vec<PathBuf>,
        /// 过滤后索引
        filtered_indices: Vec<usize>,
        /// 选中索引
        selected_index: usize,
        /// 搜索查询
        search_query: String,
        /// 滚动偏移
        scroll_offset: usize,
    },
    
    /// 重构：目录选择
    ChoosingDir {
        path: PathBuf,
        entries: Vec<String>,
        selected_index: usize,
        scroll_offset: usize,
        /// 新增：选择器模式
        mode: DirectoryChooserMode,
        /// 新增：返回目标状态
        return_to: ReturnTarget,
    },
    
    // ... 其他状态
}

/// 返回目标
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnTarget {
    /// 返回运行状态（仓库列表）
    Running,
    /// 返回主目录管理
    ManagingDirs {
        directories: Vec<PathBuf>,
        filtered_indices: Vec<usize>,
        selected_index: usize,
        search_query: String,
        scroll_offset: usize,
    },
}
```

### 8.4 键盘处理器变更

```rust
// src/handler/keyboard.rs

pub fn handle_key_event(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    // ... 现有代码 ...
    
    match &app.state {
        // 新增状态处理
        AppState::ManagingDirs { .. } => {
            handle_managing_dirs_keys(key, app, runtime);
        }
        // 修改现有状态处理
        AppState::ChoosingDir { mode, .. } => {
            handle_chooser_keys(key, app, runtime, mode.clone());
        }
        // ... 其他状态 ...
    }
}

/// 处理主目录管理界面的按键
fn handle_managing_dirs_keys(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    match key.code {
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CloseMainDirectoryManager);
        }
        KeyCode::Char('a') => {
            // 添加主目录：打开选择器（MainDirectory模式）
            let _ = app.msg_tx.try_send(AppMsg::ShowDirectoryChooserWithMode {
                mode: DirectoryChooserMode::MainDirectory,
                return_to: Box::new(app.state.clone()),
            });
        }
        KeyCode::Enter => {
            // 打开操作菜单
            // ...
        }
        KeyCode::Up => {
            let _ = app.msg_tx.try_send(AppMsg::MainDirNavUp);
        }
        KeyCode::Down => {
            let _ = app.msg_tx.try_send(AppMsg::MainDirNavDown);
        }
        KeyCode::Char('/') => {
            // 进入搜索模式
            // ...
        }
        _ => {}
    }
}

/// 处理仓库列表界面的按键（修改）
fn handle_running_keys(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    match key.code {
        // ... 现有代码 ...
        
        // m键：进入主目录管理
        KeyCode::Char('m') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowMainDirectoryManager);
        }
        
        // a键：添加单个仓库
        KeyCode::Char('a') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowDirectoryChooserWithMode {
                mode: DirectoryChooserMode::SingleRepo,
                return_to: Box::new(app.state.clone()),
            });
        }
        
        // ... 其他按键 ...
    }
}

/// 处理目录选择器的按键（修改）
fn handle_chooser_keys(
    key: KeyEvent, 
    app: &mut App, 
    runtime: &Runtime,
    mode: DirectoryChooserMode,
) {
    match key.code {
        KeyCode::Esc => {
            // 根据 return_to 返回
            let _ = app.msg_tx.try_send(AppMsg::Cancel);
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            // 根据模式执行不同验证
            match mode {
                DirectoryChooserMode::MainDirectory => {
                    // 直接确认，无需额外验证
                    let _ = app.msg_tx.try_send(AppMsg::DirectorySelected(path));
                }
                DirectoryChooserMode::SingleRepo => {
                    // 验证是否为 Git 仓库
                    let _ = app.msg_tx.try_send(AppMsg::ValidateDirectory(path));
                }
            }
        }
        // ... 其他按键 ...
    }
}
```

### 8.5 UI 渲染变更建议

```rust
// src/ui/render.rs

/// 渲染主界面（添加状态栏信息）
fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    // ... 现有代码 ...
    
    // 渲染状态栏
    render_status_bar(frame, app, status_area);
}

/// 渲染状态栏（新增）
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(repo) = app.selected_repository() {
        let main_dir = get_main_directory_for_repo(repo, &app.config);
        format!("{}/{} | 主目录: {}", 
            app.filtered_count(), 
            app.repositories.len(),
            main_dir.display()
        )
    } else {
        format!("{}/{} 仓库", app.filtered_count(), app.repositories.len())
    };
    
    let paragraph = Paragraph::new(status)
        .style(app.theme.status_bar_style());
    
    frame.render_widget(paragraph, area);
}

/// 渲染主目录管理界面（新增）
fn render_managing_dirs(frame: &mut Frame, app: &App, area: Rect) {
    // 类似仓库列表的渲染逻辑
    // 但数据是 main_directories 而非 repositories
}

/// 渲染目录选择器（重构）
fn render_directory_chooser(frame: &mut Frame, app: &App, area: Rect, mode: &DirectoryChooserMode) {
    // 现有渲染逻辑
    // 根据 mode 显示不同提示
    let hint = match mode {
        DirectoryChooserMode::MainDirectory => "选择主目录",
        DirectoryChooserMode::SingleRepo => "选择 Git 仓库",
    };
    
    // ... 渲染代码 ...
}
```

---

## 9. 测试策略

### 9.1 单元测试

| 模块 | 测试内容 | 覆盖率目标 |
|------|---------|-----------|
| config/types | 配置迁移、序列化/反序列化 | 90% |
| repo/types | display_name 生成、source 匹配 | 85% |
| app/state | 新状态优先级、is_modal | 90% |
| handler/keyboard | 新按键处理逻辑 | 80% |

### 9.2 集成测试

| 场景 | 测试步骤 |
|------|---------|
| 配置迁移 | 加载 v1 配置 → 验证自动迁移 → 检查 v2 格式 |
| 添加主目录 | 打开管理 → 添加目录 → 验证列表 → 检查仓库加载 |
| 添加单个仓库 | 按 a → 选择非 git 目录 → 验证拒绝 → 选择 git 目录 → 验证添加 |
| 显示格式 | 添加多个来源仓库 → 验证 @scope/repo-name 格式 |
| 状态栏 | 选择不同仓库 → 验证状态栏显示正确主目录 |

### 9.3 E2E 测试

| 场景 | 测试步骤 |
|------|---------|
| 完整工作流 | 启动 → 添加主目录 → 添加单仓库 → 验证显示 → 移除主目录 → 验证 |
| 向后兼容 | 使用旧配置启动 → 验证迁移 → 验证功能正常 |
| 边界情况 | 空主目录列表 → 无效路径 → 重复添加 → 大量主目录(20+) |

---

## 10. 开发计划

### 10.1 阶段划分

| 阶段 | 任务 | 预估工时 |
|------|------|---------|
| Phase 1 | 配置结构变更 + 向后兼容 | 4h |
| Phase 2 | Repository 结构扩展 + 显示格式 | 3h |
| Phase 3 | 目录选择器重构 | 4h |
| Phase 4 | 主目录管理界面实现 | 6h |
| Phase 5 | 仓库列表界面变更 | 3h |
| Phase 6 | 状态栏增强 | 2h |
| Phase 7 | 测试与调试 | 6h |
| **总计** | | **28h** |

### 10.2 依赖关系

```
Phase 1 (配置) ──┬──▶ Phase 2 (Repository) ──┬──▶ Phase 3 (选择器)
                │                            │
                └──▶ Phase 4 (管理界面) ◀────┘
                              │
                              ▼
                    Phase 5 (列表界面) ──▶ Phase 6 (状态栏)
                              │
                              ▼
                          Phase 7 (测试)
```

---

## 11. 附录

### 11.1 界面原型详细描述

#### 主目录管理界面

```
┌──────────────────────────────────────────────────────────────────────┐
│  repotui v0.1.0                           ┌───┬───┬───┐              │
│                                           │ 🌙│ 🔍│ ❓│              │
├──────────────────────────────────────────────────────────────────────┤
│  主目录管理 (3)                                       搜索:          │
├──────────────────────────────────────────────────────────────────────┤
│ ▶ ~/projects/personal                                                │
│   ~/projects/company                                                 │
│   ~/opensource                                                       │
├──────────────────────────────────────────────────────────────────────┤
│ ↑/↓ 导航 | Enter 操作菜单 | / 搜索 | a 添加 | Esc 返回              │
└──────────────────────────────────────────────────────────────────────┘
```

**元素说明**：
- 标题栏：显示 "主目录管理 (N)"，N 为目录数量
- 搜索框：支持实时过滤
- 列表项：显示完整路径，当前选中项高亮
- 底部快捷键提示

#### 操作菜单

```
┌──────────────────────────────────────────────────────────────────────┐
│  操作菜单                              当前: ~/projects/personal     │
├──────────────────────────────────────────────────────────────────────┤
│  d - 移除此目录                                                      │
│  Esc - 取消                                                        │
└──────────────────────────────────────────────────────────────────────┘
```

#### 目录选择器（SingleRepo 模式）

```
┌──────────────────────────────────────────────────────────────────────┐
│  选择 Git 仓库 (SingleRepo)                                          │
├──────────────────────────────────────────────────────────────────────┤
│  路径: /home/user/projects/test-repo                                 │
├──────────────────────────────────────────────────────────────────────┤
│ ▶ ..                                                                 │
│   .git/                                                              │
│   src/                                                               │
│   Cargo.toml                                                         │
│   README.md                                                          │
├──────────────────────────────────────────────────────────────────────┤
│  ←/→ 导航 | Space/Enter 选择 | Esc 返回                              │
│  ✅ 有效 Git 仓库                                                    │
└──────────────────────────────────────────────────────────────────────┘
```

#### 目录选择器（非 Git 仓库警告）

```
┌──────────────────────────────────────────────────────────────────────┐
│  选择 Git 仓库 (SingleRepo)                                          │
├──────────────────────────────────────────────────────────────────────┤
│  路径: /home/user/documents                                          │
├──────────────────────────────────────────────────────────────────────┤
│ ▶ ..                                                                 │
│   work/                                                              │
│   personal/                                                          │
├──────────────────────────────────────────────────────────────────────┤
│  ←/→ 导航 | Space/Enter 选择 | Esc 返回                              │
│  ⚠️ 非 Git 仓库，无法添加                                            │
└──────────────────────────────────────────────────────────────────────┘
```

### 11.2 快捷键汇总

#### 主目录管理界面

| 按键 | 功能 |
|------|------|
| ↑/↓ | 导航选择 |
| Enter | 打开操作菜单 |
| d（菜单中） | 确认移除 |
| / | 进入搜索模式 |
| a | 添加主目录（打开选择器） |
| Esc | 返回仓库列表 |

#### 仓库列表界面（变更）

| 按键 | 原功能 | 新功能 |
|------|--------|--------|
| a | 无 | 添加单个仓库 |
| m | 更改主目录 | 主目录管理 |

#### 目录选择器

| 按键 | 功能 |
|------|------|
| ←/→ | 返回上级 / 进入目录 |
| ↑/↓ | 选择条目 |
| Home/End | 跳到首项/末项 |
| Space/Enter | 确认选择 |
| Esc | 返回上一界面 |

---

**文档结束**

---

## 变更日志

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-03-08 | v1 | 初始版本创建 |
