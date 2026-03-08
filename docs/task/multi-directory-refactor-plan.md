# 多主目录管理重构实施计划

**文档版本**: 1.0  
**创建日期**: 2026-03-08  
**状态**: 待实施  
**关联**: 主目录管理  
**优先级**: P0

---

## 1. 重构范围分析

### 1.1 影响范围列表

| 层级 | 模块 | 影响程度 | 说明 |
|------|------|----------|------|
| 配置层 | `config/types.rs` | **高** | 核心数据结构变更 |
| 配置层 | `config/load.rs` | **高** | 向后兼容处理 |
| 配置层 | `config/validators.rs` | **中** | 新增验证逻辑 |
| 数据层 | `repo/types.rs` | **高** | Repository 新增 source 字段 |
| 数据层 | `repo/discover.rs` | **高** | 多目录扫描逻辑 |
| 状态层 | `app/state.rs` | **高** | 新增 ManagingDirs 状态 |
| 消息层 | `app/msg.rs` | **高** | 新增 Cmd 和 AppMsg 类型 |
| 模型层 | `app/model.rs` | **中** | 新增多目录管理字段 |
| 更新层 | `app/update.rs` | **高** | 处理新增消息类型 |
| UI层 | `ui/widgets/dir_chooser.rs` | **高** | 支持两种选择器模式 |
| UI层 | `ui/render.rs` | **中** | 新增 ManagingDirs 渲染 |
| 处理层 | `handler/keyboard.rs` | **中** | 新增键盘映射 |

### 1.2 文件变更清单

#### 必须修改的文件（18个）

1. **src/config/types.rs** - 配置结构体重构
2. **src/config/load.rs** - 配置加载与迁移
3. **src/config/validators.rs** - 新增验证方法
4. **src/repo/types.rs** - Repository 新增字段和方法
5. **src/repo/discover.rs** - 多目录发现逻辑
6. **src/app/state.rs** - AppState 扩展
7. **src/app/msg.rs** - 新增消息和命令类型
8. **src/app/model.rs** - App 结构体扩展
9. **src/app/update.rs** - 新增消息处理
10. **src/ui/widgets/dir_chooser.rs** - 通用目录选择器
11. **src/ui/render.rs** - 新增 ManagingDirs 渲染
12. **src/handler/keyboard.rs** - 新增键盘处理
13. **src/ui/widgets/mod.rs** - 导出新增组件

#### 新增文件（3个）

14. **src/config/migration.rs** - 配置迁移逻辑
15. **src/repo/source.rs** - RepoSource 枚举定义
16. **docs/design/multi-directory-ui.md** - UI 设计文档

#### 测试文件（4个）

17. **tests/config_migration_test.rs** - 配置迁移测试
18. **tests/multi_directory_test.rs** - 多目录功能测试
19. **tests/repo_source_test.rs** - RepoSource 测试

### 1.3 依赖关系分析

```
                    ┌─────────────────────────────────────┐
                    │         配置层 (Config)              │
                    │  types.rs ← load.rs ← validators.rs │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │         数据层 (Repo)                │
                    │      types.rs ← discover.rs         │
                    └──────────────┬──────────────────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
┌───────▼───────┐         ┌────────▼─────────┐      ┌────────▼────────┐
│   Model       │◀────────│     State        │      │      Msg        │
│  (model.rs)   │         │   (state.rs)     │      │   (msg.rs)      │
└───────┬───────┘         └──────────────────┘      └────────┬────────┘
        │                                                    │
        └──────────────────┬─────────────────────────────────┘
                           │
               ┌───────────▼────────────┐
               │    Update (update.rs)   │
               └───────────┬─────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
┌───────▼───────┐  ┌───────▼───────┐  ┌───────▼───────┐
│   Render      │  │    Keyboard   │  │   DirChooser  │
│  (render.rs)  │  │ (keyboard.rs) │  │(dir_chooser.rs)│
└───────────────┘  └───────────────┘  └───────────────┘
```

---

## 2. 分阶段实施计划

### Phase 1: 配置层重构（预计2-3天）

#### 2.1.1 变更文件: `src/config/types.rs`

**新增类型定义:**

```rust
/// 仓库来源类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoSource {
    /// 来自主目录扫描
    MainDirectory { 
        /// 主目录索引
        dir_index: usize,
        /// 主目录路径（冗余存储用于显示）
        dir_path: PathBuf,
    },
    /// 独立添加的仓库
    Single,
}

/// 单个仓库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleRepoConfig {
    /// 仓库路径
    pub path: PathBuf,
    /// 显示名称（可选，默认使用目录名）
    pub display_name: Option<String>,
    /// 添加到收藏的时间戳
    pub added_at: Option<chrono::DateTime<chrono::Local>>,
}

/// 主目录配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainDirectoryConfig {
    /// 目录路径
    pub path: PathBuf,
    /// 显示名称（可选，默认使用目录名）
    pub display_name: Option<String>,
    /// 扫描深度（覆盖全局设置）
    pub max_depth: Option<usize>,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}
```

**修改 Config 结构体:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 配置版本（用于迁移）
    pub version: String,

    /// 主目录列表（新版本）
    #[serde(default)]
    pub main_directories: Vec<MainDirectoryConfig>,

    /// 独立仓库列表
    #[serde(default)]
    pub single_repositories: Vec<SingleRepoConfig>,

    /// 向后兼容：旧版单主目录字段
    #[serde(default)]
    pub main_directory: Option<PathBuf>,

    /// 其他字段保持不变...
    #[serde(default)]
    pub editors: EditorConfig,
    #[serde(default)]
    pub default_command: Option<String>,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub favorites: FavoritesConfig,
    #[serde(default)]
    pub recent: RecentConfig,
}
```

**新增方法:**

```rust
impl Config {
    /// 获取所有启用的主目录路径
    pub fn enabled_main_dirs(&self) -> Vec<&PathBuf> {
        self.main_directories
            .iter()
            .filter(|d| d.enabled)
            .map(|d| &d.path)
            .collect()
    }

    /// 检查是否需要迁移（旧版本配置）
    pub fn needs_migration(&self) -> bool {
        self.main_directories.is_empty() && self.main_directory.is_some()
    }

    /// 执行配置迁移
    pub fn migrate(&mut self) {
        if let Some(old_dir) = self.main_directory.take() {
            if !old_dir.as_os_str().is_empty() {
                self.main_directories.push(MainDirectoryConfig {
                    path: old_dir,
                    display_name: None,
                    max_depth: None,
                    enabled: true,
                });
            }
        }
        self.version = crate::constants::CONFIG_VERSION.to_string();
    }
}
```

#### 2.1.2 变更文件: `src/config/load.rs`

**修改 `load_or_create_config` 函数:**

```rust
pub fn load_or_create_config() -> AppResult<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(AppError::Config(ConfigError::NotFound(config_path)));
    }

    let mut config = load_config()?;

    // 自动迁移旧版本配置
    if config.needs_migration() {
        log::info!("Migrating config from single to multi-directory format");
        config.migrate();
        // 保存迁移后的配置
        save_config(&config)?;
    }

    // 验证至少有配置或主目录
    if config.main_directories.is_empty() && config.single_repositories.is_empty() {
        return Err(AppError::Config(ConfigError::NotFound(config_path)));
    }

    Ok(config)
}
```

#### 2.1.3 新增文件: `src/config/migration.rs`

```rust
//! Configuration migration utilities

use crate::config::types::{Config, MainDirectoryConfig, SingleRepoConfig};
use crate::error::AppResult;
use std::path::PathBuf;

/// 配置版本历史
pub const VERSION_HISTORY: &[&str] = &["1.0", "1.1", "2.0"];

/// 迁移配置到最新版本
pub fn migrate_config(config: &mut Config) -> AppResult<()> {
    match config.version.as_str() {
        "1.0" | "1.1" => migrate_v1_to_v2(config),
        _ => Ok(()), // 已经是最新版本
    }
}

/// 从 v1.x 迁移到 v2.0
fn migrate_v1_to_v2(config: &mut Config) -> AppResult<()> {
    // 迁移旧的主目录字段
    if let Some(old_dir) = config.main_directory.take() {
        if !old_dir.as_os_str().is_empty() {
            config.main_directories.push(MainDirectoryConfig {
                path: old_dir,
                display_name: None,
                max_depth: None,
                enabled: true,
            });
        }
    }

    // 从 favorites 迁移独立仓库
    for fav_path in &config.favorites.repositories {
        let path = PathBuf::from(fav_path);
        // 检查是否已经在主目录中
        let in_main_dir = config.main_directories.iter().any(|d| {
            path.starts_with(&d.path)
        });
        
        if !in_main_dir {
            config.single_repositories.push(SingleRepoConfig {
                path,
                display_name: None,
                added_at: Some(chrono::Local::now()),
            });
        }
    }

    config.version = "2.0".to_string();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_v1_to_v2() {
        let mut config = Config {
            version: "1.0".to_string(),
            main_directory: Some(PathBuf::from("/home/user/projects")),
            main_directories: vec![],
            single_repositories: vec![],
            // ... 其他字段
        };

        migrate_v1_to_v2(&mut config).unwrap();

        assert_eq!(config.version, "2.0");
        assert_eq!(config.main_directories.len(), 1);
        assert!(config.main_directory.is_none());
    }
}
```

**测试要求:**
- 测试旧配置自动迁移
- 测试空配置处理
- 测试多版本兼容

**风险点:**
- 配置迁移失败可能导致数据丢失 → 迁移前自动备份
- 旧版本用户无法读取新配置 → 保持向后兼容读取

---

### Phase 2: 数据结构重构（预计2天）

#### 2.2.1 新增文件: `src/repo/source.rs`

```rust
//! Repository source types

use std::path::PathBuf;

/// 仓库来源类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepoSource {
    /// 来自主目录扫描
    MainDirectory {
        /// 主目录索引
        dir_index: usize,
        /// 主目录路径（用于生成显示名称）
        dir_path: PathBuf,
    },
    /// 独立添加的仓库
    Single,
}

impl RepoSource {
    /// 获取作用域名称（用于 display_name）
    pub fn scope(&self) -> String {
        match self {
            RepoSource::MainDirectory { dir_path, .. } => {
                dir_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            }
            RepoSource::Single => "single".to_string(),
        }
    }

    /// 检查是否来自特定主目录
    pub fn is_from_main_dir(&self, dir_index: usize) -> bool {
        matches!(self, RepoSource::MainDirectory { dir_index: idx, .. } if *idx == dir_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_source_scope() {
        let source = RepoSource::MainDirectory {
            dir_index: 0,
            dir_path: PathBuf::from("/home/user/work"),
        };
        assert_eq!(source.scope(), "work");

        let single = RepoSource::Single;
        assert_eq!(single.scope(), "single");
    }
}
```

#### 2.2.2 变更文件: `src/repo/types.rs`

**修改 Repository 结构体:**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// 仓库名称
    pub name: String,

    /// 仓库路径
    pub path: PathBuf,

    /// 最后修改时间
    pub last_modified: Option<SystemTime>,

    /// 是否有未提交更改
    pub is_dirty: bool,

    /// 当前分支
    pub branch: Option<String>,

    /// 是否是 Git 仓库
    pub is_git_repo: bool,

    /// 仓库来源（新增）
    pub source: RepoSource,
}

impl Repository {
    /// 创建新仓库（带来源）
    pub fn from_path_with_source(path: PathBuf, source: RepoSource) -> Self {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let last_modified = path.metadata().ok().and_then(|m| m.modified().ok());
        let is_git_repo = path.join(".git").exists();

        Self {
            name,
            path,
            last_modified,
            is_dirty: false,
            branch: None,
            is_git_repo,
            source,
        }
    }

    /// 生成显示名称: @scope/repo-name
    pub fn display_name(&self) -> String {
        format!("@{}/{}", self.source.scope(), self.name)
    }

    /// 获取完整显示信息
    pub fn display_info(&self) -> String {
        if self.is_git_repo {
            match &self.branch {
                Some(branch) => format!("{} [{}]{}", 
                    self.display_name(),
                    branch,
                    if self.is_dirty { " *" } else { "" }
                ),
                None => self.display_name(),
            }
        } else {
            format!("{} (not git)", self.display_name())
        }
    }
}
```

#### 2.2.3 变更文件: `src/repo/discover.rs`

**修改扫描逻辑:**

```rust
/// 从多个主目录发现仓库
pub fn discover_repositories_multi(
    main_dirs: &[(usize, &PathBuf, Option<usize>)],
    single_repos: &[PathBuf],
    config: &ScanConfig,
) -> Result<Vec<Repository>, RepoError> {
    let mut all_repos = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 扫描主目录
    for (dir_index, dir_path, max_depth) in main_dirs {
        let repos = discover_in_directory(dir_path, *max_depth, config)?;
        for repo_path in repos {
            if seen_paths.insert(repo_path.clone()) {
                let source = RepoSource::MainDirectory {
                    dir_index: *dir_index,
                    dir_path: (*dir_path).clone(),
                };
                all_repos.push(Repository::from_path_with_source(repo_path, source));
            }
        }
    }

    // 添加独立仓库
    for repo_path in single_repos {
        if seen_paths.insert(repo_path.clone()) && repo_path.exists() {
            let source = RepoSource::Single;
            all_repos.push(Repository::from_path_with_source(repo_path.clone(), source));
        }
    }

    // 按最后修改时间排序
    all_repos.sort_by(|a, b| {
        b.last_modified.cmp(&a.last_modified)
    });

    Ok(all_repos)
}
```

**测试要求:**
- 测试多目录扫描（去重）
- 测试独立仓库添加
- 测试 RepoSource 生成正确的作用域

**风险点:**
- 同名仓库冲突 → 使用路径去重
- 性能问题 → 并行扫描各个主目录

---

### Phase 3: 状态机重构（预计2-3天）

#### 2.3.1 变更文件: `src/app/state.rs`

**新增 DirectoryChooserMode 枚举:**

```rust
/// 目录选择器模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryChooserMode {
    /// 选择主目录（首次启动或管理主目录）
    SelectMainDirectory {
        /// 是否允许多选
        allow_multiple: bool,
        /// 编辑模式（替换现有或添加新）
        edit_mode: bool,
    },
    /// 添加独立仓库
    AddSingleRepository,
}

impl Default for DirectoryChooserMode {
    fn default() -> Self {
        DirectoryChooserMode::SelectMainDirectory {
            allow_multiple: false,
            edit_mode: false,
        }
    }
}
```

**修改 AppState 枚举:**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// 正常运行状态
    Running,

    /// 选择目录（通用，支持两种模式）
    ChoosingDir {
        /// 当前路径
        path: PathBuf,
        /// 目录条目
        entries: Vec<String>,
        /// 选中索引
        selected_index: usize,
        /// 滚动偏移
        scroll_offset: usize,
        /// 选择器模式（新增）
        mode: DirectoryChooserMode,
    },

    /// 管理主目录列表（新增状态）
    ManagingDirs {
        /// 主目录列表状态
        list_state: ratatui::widgets::ListState,
        /// 当前选中的目录索引
        selected_dir_index: usize,
        /// 编辑中的目录信息（如果有）
        editing: Option<MainDirEdit>,
    },

    /// 显示操作菜单
    ShowingActions { repo: Repository },

    /// 显示帮助面板
    ShowingHelp { scroll_offset: usize },

    /// 加载中
    Loading { message: String },

    /// 错误状态
    Error { message: String },

    /// 退出
    Quit,

    /// 选择主题
    SelectingTheme {
        theme_list_state: ratatui::widgets::ListState,
        preview_theme: crate::ui::theme::Theme,
    },
}

/// 主目录编辑状态
#[derive(Debug, Clone)]
pub struct MainDirEdit {
    /// 编辑的目录索引（None 表示新增）
    pub index: Option<usize>,
    /// 当前编辑的路径
    pub path: PathBuf,
    /// 显示名称
    pub display_name: String,
    /// 是否启用
    pub enabled: bool,
}
```

**修改优先级方法:**

```rust
impl AppState {
    pub fn priority(&self) -> u8 {
        match self {
            AppState::ShowingActions { .. } => 5,
            AppState::ShowingHelp { .. } => 4,
            AppState::ManagingDirs { .. } => 4, // 与管理目录同优先级
            AppState::ChoosingDir { .. } => 3,
            AppState::SelectingTheme { .. } => 3,
            AppState::Running => 1,
            AppState::Loading { .. } | AppState::Error { .. } => 0,
            AppState::Quit => 0,
        }
    }

    pub fn is_modal(&self) -> bool {
        matches!(
            self,
            AppState::ShowingActions { .. }
                | AppState::ShowingHelp { .. }
                | AppState::ChoosingDir { .. }
                | AppState::SelectingTheme { .. }
                | AppState::ManagingDirs { .. } // 新增
        )
    }
}
```

#### 2.3.2 变更文件: `src/app/msg.rs`

**新增 Cmd 类型:**

```rust
#[derive(Debug, Clone)]
pub enum Cmd {
    // 现有命令...
    LoadConfig,
    LoadRepositories(PathBuf),
    CheckGitStatus(usize, PathBuf),
    ExecuteAction(Action, Repository),
    ExecuteBatchAction(Action, Vec<Repository>),
    ScanDirectory(PathBuf),

    // 新增命令
    /// 从多个主目录加载仓库
    LoadRepositoriesMulti(Vec<(PathBuf, Option<usize>)>),
    
    /// 保存配置
    SaveConfig(crate::config::Config),
    
    /// 验证目录路径
    ValidateDirectory(PathBuf),
}
```

**新增 AppMsg 类型:**

```rust
#[derive(Debug, Clone)]
pub enum AppMsg {
    // 现有消息...
    SearchInput(char),
    SearchBackspace,
    // ... 等等

    // === 目录管理相关（新增）===
    /// 显示主目录管理界面
    ShowMainDirectoryManager,
    
    /// 关闭主目录管理
    CloseMainDirectoryManager,
    
    /// 添加主目录
    AddMainDirectory(PathBuf),
    
    /// 移除主目录
    RemoveMainDirectory(usize),
    
    /// 切换主目录启用状态
    ToggleMainDirectoryEnabled(usize),
    
    /// 更新主目录显示名称
    UpdateMainDirectoryName(usize, String),
    
    /// 在管理器中导航
    MainDirNavUp,
    MainDirNavDown,
    
    /// 编辑主目录（打开编辑模式）
    EditMainDirectory(usize),
    
    /// 确认编辑
    ConfirmEditMainDirectory,
    
    /// 取消编辑
    CancelEditMainDirectory,

    // === 独立仓库相关（新增）===
    /// 显示添加独立仓库选择器
    ShowAddSingleRepoChooser,
    
    /// 添加独立仓库
    AddSingleRepository(PathBuf),
    
    /// 移除独立仓库
    RemoveSingleRepository(PathBuf),

    // === 目录选择器增强 ===
    /// 显示选择器（带模式）
    ShowDirectoryChooserWithMode(DirectoryChooserMode),
    
    /// 多个目录被选择（多选模式）
    DirectoriesSelected(Vec<String>),
}
```

#### 2.3.3 变更文件: `src/app/model.rs`

**新增字段:**

```rust
pub struct App {
    // 现有字段...
    pub config: Option<Config>,
    pub main_dir: Option<PathBuf>, // 保留向后兼容，但标记为 deprecated
    pub repositories: Vec<Repository>,
    pub filtered_indices: Vec<usize>,
    pub search_query: String,
    pub pending_search: Option<String>,
    pub search_active: bool,
    pub list_state: ListState,
    pub scroll_offset: usize,
    pub state: AppState,
    pub loading: bool,
    pub loading_message: Option<String>,
    pub error_message: Option<String>,
    pub selected_repo: Option<Repository>,
    pub msg_tx: mpsc::Sender<AppMsg>,
    pub path_bar_area: Option<ratatui::layout::Rect>,
    pub git_cache: Arc<StatusCache>,
    pub git_scheduler: Option<Arc<GitStatusScheduler>>,
    pub theme: Theme,
    pub favorites: FavoritesStore,
    pub recent: RecentStore,
    pub view_mode: ViewMode,
    pub selection_mode: bool,
    pub selected_indices: HashSet<usize>,

    // 新增字段
    /// 主目录列表（从配置加载）
    pub main_directories: Vec<MainDirectoryInfo>,
    
    /// 独立仓库列表
    pub single_repositories: Vec<PathBuf>,
    
    /// 当前选中的主目录索引（用于过滤）
    pub active_main_dir_index: Option<usize>,
}

/// 主目录信息（运行时）
#[derive(Debug, Clone)]
pub struct MainDirectoryInfo {
    pub path: PathBuf,
    pub display_name: String,
    pub enabled: bool,
    pub repo_count: usize,
}
```

**修改 new() 方法:**

```rust
impl App {
    pub fn new(msg_tx: mpsc::Sender<AppMsg>) -> Self {
        // ... 现有初始化代码
        
        Self {
            // ... 现有字段
            main_directories: Vec::new(),
            single_repositories: Vec::new(),
            active_main_dir_index: None,
        }
    }
}
```

#### 2.3.4 变更文件: `src/app/update.rs`

**修改 ConfigLoaded 处理:**

```rust
AppMsg::ConfigLoaded(result) => {
    match *result {
        Ok(config) => {
            // 加载主目录信息
            app.main_directories = config.main_directories.iter().enumerate().map(|(i, d)| {
                MainDirectoryInfo {
                    path: d.path.clone(),
                    display_name: d.display_name.clone()
                        .unwrap_or_else(|| {
                            d.path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string()
                        }),
                    enabled: d.enabled,
                    repo_count: 0, // 稍后更新
                }
            }).collect();

            // 加载独立仓库
            app.single_repositories = config.single_repositories.iter()
                .map(|r| r.path.clone())
                .collect();

            app.config = Some(config.clone());
            // ... 其他加载逻辑

            // 从多个主目录加载仓库
            let main_dirs: Vec<_> = config.main_directories.iter()
                .filter(|d| d.enabled)
                .map(|d| (d.path.clone(), d.max_depth))
                .collect();
            
            runtime.dispatch(crate::app::msg::Cmd::LoadRepositoriesMulti(main_dirs));
        }
        // ... 错误处理
    }
}
```

**新增消息处理:**

```rust
// 主目录管理
AppMsg::ShowMainDirectoryManager => {
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(0));
    
    app.state = AppState::ManagingDirs {
        list_state,
        selected_dir_index: 0,
        editing: None,
    };
}

AppMsg::CloseMainDirectoryManager => {
    app.state = AppState::Running;
}

AppMsg::AddMainDirectory(path) => {
    if let Some(ref mut config) = app.config {
        let display_name = path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        
        config.main_directories.push(MainDirectoryConfig {
            path: path.clone(),
            display_name,
            max_depth: None,
            enabled: true,
        });
        
        // 保存配置并刷新
        let _ = config::save_config(config);
        runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
    }
}

// 独立仓库管理
AppMsg::ShowAddSingleRepoChooser => {
    app.state = AppState::ChoosingDir {
        path: dirs::home_dir().unwrap_or_default(),
        entries: Vec::new(),
        selected_index: 0,
        scroll_offset: 0,
        mode: DirectoryChooserMode::AddSingleRepository,
    };
    runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
        dirs::home_dir().unwrap_or_default(),
    ));
}

AppMsg::AddSingleRepository(path) => {
    if let Some(ref mut config) = app.config {
        // 检查是否已存在
        let exists = config.single_repositories.iter()
            .any(|r| r.path == path);
        
        if !exists {
            config.single_repositories.push(SingleRepoConfig {
                path,
                display_name: None,
                added_at: Some(chrono::Local::now()),
            });
            
            let _ = config::save_config(config);
            runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
        }
    }
}
```

**测试要求:**
- 测试状态转换正确
- 测试主目录增删改
- 测试独立仓库管理

**风险点:**
- 状态转换复杂 → 添加全面的单元测试
- 配置保存失败 → 添加错误处理和恢复机制

---

### Phase 4: UI 层重构（预计3-4天）

#### 2.4.1 变更文件: `src/ui/widgets/dir_chooser.rs`

**重构为通用组件:**

```rust
//! 通用目录选择器组件
//!
//! 支持两种模式:
//! 1. SelectMainDirectory - 选择主目录
//! 2. AddSingleRepository - 添加独立仓库

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use std::path::Path;

use crate::app::state::DirectoryChooserMode;
use crate::ui::theme::Theme;

/// 目录选择器状态
#[derive(Debug, Clone)]
pub struct DirectoryChooserState {
    /// 当前路径
    pub current_path: PathBuf,
    /// 目录条目
    pub entries: Vec<String>,
    /// 选中索引
    pub selected_index: usize,
    /// 滚动偏移
    pub scroll_offset: usize,
    /// 选择器模式
    pub mode: DirectoryChooserMode,
}

impl DirectoryChooserState {
    pub fn new(initial_path: PathBuf, mode: DirectoryChooserMode) -> Self {
        Self {
            current_path: initial_path,
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            mode,
        }
    }

    /// 根据模式获取标题
    pub fn title(&self) -> &'static str {
        match &self.mode {
            DirectoryChooserMode::SelectMainDirectory { .. } => "Select Main Directory",
            DirectoryChooserMode::AddSingleRepository => "Add Single Repository",
        }
    }

    /// 根据模式获取帮助文本
    pub fn help_text(&self) -> &'static str {
        match &self.mode {
            DirectoryChooserMode::SelectMainDirectory { allow_multiple, .. } => {
                if *allow_multiple {
                    "↑↓ navigate   ← back   → enter   SPACE toggle select   Enter confirm   Esc cancel"
                } else {
                    "↑↓ navigate   ← back   → enter   SPACE select   Esc cancel"
                }
            }
            DirectoryChooserMode::AddSingleRepository => {
                "↑↓ navigate   ← back   → enter   SPACE select repo   Esc cancel"
            }
        }
    }
}

/// 目录选择器组件
pub struct DirectoryChooser<'a> {
    pub state: &'a DirectoryChooserState,
    pub theme: &'a Theme,
    pub visible_height: u16,
    pub git_repo_count: Option<usize>,
    /// 多选模式下已选择的目录（可选）
    pub selected_paths: Option<&'a std::collections::HashSet<PathBuf>>,
}

impl<'a> DirectoryChooser<'a> {
    pub fn new(state: &'a DirectoryChooserState, theme: &'a Theme) -> Self {
        Self {
            state,
            theme,
            visible_height: 10,
            git_repo_count: None,
            selected_paths: None,
        }
    }

    pub fn visible_height(mut self, height: u16) -> Self {
        self.visible_height = height;
        self
    }

    pub fn git_repo_count(mut self, count: usize) -> Self {
        self.git_repo_count = Some(count);
        self
    }

    pub fn selected_paths(mut self, paths: &'a std::collections::HashSet<PathBuf>) -> Self {
        self.selected_paths = Some(paths);
        self
    }
}

impl<'a> Widget for DirectoryChooser<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 根据模式调整布局
        let show_selection_indicator = matches!(
            self.state.mode,
            DirectoryChooserMode::SelectMainDirectory { allow_multiple: true, .. }
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Length(3), // 当前路径
                Constraint::Length(2), // 统计
                Constraint::Min(5),    // 目录列表
                Constraint::Length(1), // 间距
                Constraint::Length(1), // 帮助文本
            ])
            .split(area);

        // 渲染标题
        self.render_title(chunks[0], buf);
        
        // 渲染当前路径
        self.render_current_path(chunks[1], buf);
        
        // 渲染统计
        self.render_stats(chunks[2], buf);
        
        // 渲染目录列表（支持选择指示器）
        self.render_directory_list(chunks[3], buf, show_selection_indicator);
        
        // 渲染帮助
        self.render_help(chunks[5], buf);
    }
}

impl<'a> DirectoryChooser<'a> {
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let icon = match &self.state.mode {
            DirectoryChooserMode::SelectMainDirectory { .. } => "📁",
            DirectoryChooserMode::AddSingleRepository => "📦",
        };
        
        let text = format!("{} {}", icon, self.state.title());
        
        let title = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(self.theme.colors.primary.into())
                    .add_modifier(Modifier::BOLD),
            );
        title.render(area, buf);
    }

    fn render_directory_list(
        &self, 
        area: Rect, 
        buf: &mut Buffer,
        show_selection: bool,
    ) {
        // 实现类似原版的列表渲染
        // 但在 show_selection 为 true 时显示复选框
        // ...
    }

    // ... 其他渲染方法
}
```

#### 2.4.2 新增文件: `src/ui/widgets/main_dir_manager.rs`

```rust
//! 主目录管理器组件

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Clear};

use crate::app::model::MainDirectoryInfo;
use crate::ui::theme::Theme;

/// 主目录管理器
pub struct MainDirManager<'a> {
    pub directories: &'a [MainDirectoryInfo],
    pub selected_index: usize,
    pub theme: &'a Theme,
    pub editing_index: Option<usize>,
    pub editing_name: &'a str,
}

impl<'a> MainDirManager<'a> {
    pub fn new(
        directories: &'a [MainDirectoryInfo],
        selected_index: usize,
        theme: &'a Theme,
    ) -> Self {
        Self {
            directories,
            selected_index,
            theme,
            editing_index: None,
            editing_name: "",
        }
    }

    pub fn editing(mut self, index: usize, name: &'a str) -> Self {
        self.editing_index = Some(index);
        self.editing_name = name;
        self
    }
}

impl<'a> Widget for MainDirManager<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 清除背景
        Clear.render(area, buf);

        // 创建布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 标题
                Constraint::Min(5),    // 目录列表
                Constraint::Length(3), // 操作提示
            ])
            .margin(2)
            .split(area);

        // 渲染标题
        self.render_title(chunks[0], buf);
        
        // 渲染目录列表
        self.render_directory_list(chunks[1], buf);
        
        // 渲染操作提示
        self.render_help(chunks[2], buf);
    }
}

impl<'a> MainDirManager<'a> {
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let title = Paragraph::new("🏠 Manage Main Directories")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(self.theme.colors.primary.into())
                    .add_modifier(Modifier::BOLD),
            );
        title.render(area, buf);
    }

    fn render_directory_list(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self.directories.iter().enumerate().map(|(i, dir)| {
            let is_selected = i == self.selected_index;
            let is_editing = self.editing_index == Some(i);
            
            // 构建显示文本
            let enabled_icon = if dir.enabled { "✓" } else { "✗" };
            let name = if is_editing {
                format!("► {} (editing: {})", enabled_icon, self.editing_name)
            } else {
                format!("{} {} ({} repos)", enabled_icon, dir.display_name, dir.repo_count)
            };
            
            let style = if is_selected {
                Style::default()
                    .bg(self.theme.colors.selected_bg.into())
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.colors.foreground.into())
            };
            
            ListItem::new(name).style(style)
        }).collect();

        let block = Block::default()
            .title(format!(" Directories ({}) ", self.directories.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.colors.border.into()));

        let list = List::new(items).block(block);
        list.render(area, buf);
    }

    fn render_help(&self, area: Rect, buf: &mut Buffer) {
        let help_text = if self.editing_index.is_some() {
            "Enter confirm   Esc cancel   Type to edit name"
        } else {
            "↑↓ navigate   a add   d delete   e edit   SPACE toggle   s save   Esc close"
        };

        let paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(self.theme.colors.text_muted.into()));

        paragraph.render(area, buf);
    }
}

/// 创建居中弹窗区域
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    // 同 dir_chooser.rs 的实现
}
```

#### 2.4.3 变更文件: `src/ui/render.rs`

**新增 ManagingDirs 渲染:**

```rust
fn render_state(&self, app: &App, frame: &mut Frame, area: Rect) {
    match &app.state {
        AppState::Running => self.render_repository_list(app, frame, area),
        AppState::ChoosingDir { mode, .. } => {
            // 使用通用目录选择器
            self.render_directory_chooser(app, frame, area, mode);
        }
        AppState::ManagingDirs { .. } => {
            // 渲染主目录管理器
            self.render_main_dir_manager(app, frame, area);
        }
        // ... 其他状态
    }
}

fn render_directory_chooser(
    &self, 
    app: &App, 
    frame: &mut Frame, 
    area: Rect,
    mode: &DirectoryChooserMode,
) {
    let popup_area = centered_rect(70, 80, area);
    
    // 从 app.state 中提取状态
    if let AppState::ChoosingDir { 
        path, 
        entries, 
        selected_index, 
        scroll_offset,
        mode,
    } = &app.state {
        let state = DirectoryChooserState {
            current_path: path.clone(),
            entries: entries.clone(),
            selected_index: *selected_index,
            scroll_offset: *scroll_offset,
            mode: mode.clone(),
        };
        
        let chooser = DirectoryChooser::new(&state, &app.theme)
            .visible_height(popup_area.height.saturating_sub(10));
        
        frame.render_widget(chooser, popup_area);
    }
}

fn render_main_dir_manager(&self, app: &App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 70, area);
    
    if let AppState::ManagingDirs { 
        selected_dir_index, 
        editing,
        ..
    } = &app.state {
        let manager = MainDirManager::new(
            &app.main_directories,
            *selected_dir_index,
            &app.theme,
        );
        
        // 如果有编辑状态，传递编辑信息
        let manager = if let Some(ref edit) = editing {
            manager.editing(edit.index.unwrap_or(0), &edit.display_name)
        } else {
            manager
        };
        
        frame.render_widget(manager, popup_area);
    }
}
```

**测试要求:**
- 测试两种模式下的目录选择器
- 测试主目录管理器渲染
- 测试键盘导航

**风险点:**
- UI 复杂度增加 → 使用组件化设计
- 布局问题 → 添加响应式布局测试

---

### Phase 5: 键盘处理重构（预计2天）

#### 2.5.1 变更文件: `src/handler/keyboard.rs`

**修改状态处理逻辑:**

```rust
pub fn handle_key_event(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    // Ctrl+C 全局处理
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        let _ = app.msg_tx.try_send(AppMsg::Quit);
        return;
    }

    // 状态优先级处理
    match &app.state {
        AppState::ShowingActions { .. } => {
            handle_action_menu_keys(key, app, runtime);
        }
        AppState::ShowingHelp { .. } => {
            handle_help_keys(key, app);
        }
        AppState::ChoosingDir { mode, .. } => {
            // 根据模式选择处理函数
            match mode {
                DirectoryChooserMode::SelectMainDirectory { .. } => {
                    handle_main_dir_chooser_keys(key, app, runtime);
                }
                DirectoryChooserMode::AddSingleRepository => {
                    handle_single_repo_chooser_keys(key, app, runtime);
                }
            }
        }
        AppState::ManagingDirs { editing, .. } => {
            if editing.is_some() {
                handle_main_dir_edit_keys(key, app);
            } else {
                handle_main_dir_manager_keys(key, app, runtime);
            }
        }
        // ... 其他状态
    }
}
```

**新增主目录管理器键盘处理:**

```rust
fn handle_main_dir_manager_keys(key: KeyEvent, app: &mut App, runtime: &Runtime) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            let _ = app.msg_tx.try_send(AppMsg::CloseMainDirectoryManager);
        }
        KeyCode::Char('a') => {
            // 添加新主目录 - 打开选择器
            app.state = AppState::ChoosingDir {
                path: dirs::home_dir().unwrap_or_default(),
                entries: Vec::new(),
                selected_index: 0,
                scroll_offset: 0,
                mode: DirectoryChooserMode::SelectMainDirectory {
                    allow_multiple: false,
                    edit_mode: false,
                },
            };
            runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                dirs::home_dir().unwrap_or_default(),
            ));
        }
        KeyCode::Char('d') => {
            // 删除选中的主目录
            if let AppState::ManagingDirs { selected_dir_index, .. } = &app.state {
                let _ = app.msg_tx.try_send(AppMsg::RemoveMainDirectory(*selected_dir_index));
            }
        }
        KeyCode::Char('e') => {
            // 编辑选中的主目录
            if let AppState::ManagingDirs { selected_dir_index, .. } = &app.state {
                let _ = app.msg_tx.try_send(AppMsg::EditMainDirectory(*selected_dir_index));
            }
        }
        KeyCode::Char(' ') => {
            // 切换启用状态
            if let AppState::ManagingDirs { selected_dir_index, .. } = &app.state {
                let _ = app.msg_tx.try_send(AppMsg::ToggleMainDirectoryEnabled(*selected_dir_index));
            }
        }
        KeyCode::Char('s') => {
            // 保存配置（刷新）
            runtime.dispatch(crate::app::msg::Cmd::LoadConfig);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let _ = app.msg_tx.try_send(AppMsg::MainDirNavUp);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let _ = app.msg_tx.try_send(AppMsg::MainDirNavDown);
        }
        _ => {}
    }
}

fn handle_main_dir_edit_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => {
            let _ = app.msg_tx.try_send(AppMsg::CancelEditMainDirectory);
        }
        KeyCode::Enter => {
            let _ = app.msg_tx.try_send(AppMsg::ConfirmEditMainDirectory);
        }
        KeyCode::Char(c) => {
            // 编辑名称
            if let AppState::ManagingDirs { 
                editing: Some(ref mut edit),
                ..
            } = &mut app.state {
                edit.display_name.push(c);
            }
        }
        KeyCode::Backspace => {
            if let AppState::ManagingDirs { 
                editing: Some(ref mut edit),
                ..
            } = &mut app.state {
                edit.display_name.pop();
            }
        }
        _ => {}
    }
}
```

**修改运行状态键盘处理:**

```rust
fn handle_running_keys(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        // ... 现有导航快捷键

        // 修改: m 键现在进入主目录管理
        KeyCode::Char('m') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowMainDirectoryManager);
        }

        // 新增: a 键打开目录选择器（添加仓库）
        KeyCode::Char('a') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowDirectoryChooser);
        }

        // 新增: A 键（大写）添加独立仓库
        KeyCode::Char('A') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowAddSingleRepoChooser);
        }

        // ... 其他快捷键
    }
}
```

**测试要求:**
- 测试所有新增键盘映射
- 测试状态切换
- 测试编辑模式

**风险点:**
- 快捷键冲突 → 更新帮助文档
- 键盘处理复杂 → 使用状态机模式

---

### Phase 6: 集成测试（预计2-3天）

#### 2.6.1 测试策略

**单元测试:**
- 配置迁移测试
- RepoSource 功能测试
- 消息处理测试

**集成测试:**
- 多目录扫描流程
- 主目录增删改流程
- 独立仓库管理流程
- 配置保存/加载完整流程

**E2E 测试:**
- 首次启动配置向导
- 主目录管理 UI 流程
- 添加独立仓库流程

#### 2.6.2 测试文件: `tests/multi_directory_test.rs`

```rust
//! 多目录管理集成测试

use repotui::config::{Config, MainDirectoryConfig};
use repotui::repo::{discover_repositories_multi, RepoSource};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_discover_from_multiple_dirs() {
    // 创建临时目录结构
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join("work");
    let personal_dir = temp.path().join("personal");
    
    // 创建仓库
    create_mock_repo(&work_dir.join("project1"));
    create_mock_repo(&work_dir.join("project2"));
    create_mock_repo(&personal_dir.join("blog"));
    
    // 扫描
    let main_dirs = vec![
        (0, &work_dir, Some(2usize)),
        (1, &personal_dir, Some(2usize)),
    ];
    
    let repos = discover_repositories_multi(&main_dirs, &[], &Default::default()).unwrap();
    
    assert_eq!(repos.len(), 3);
    
    // 验证来源
    let work_repos: Vec<_> = repos.iter()
        .filter(|r| r.source.scope() == "work")
        .collect();
    assert_eq!(work_repos.len(), 2);
    
    // 验证显示名称
    assert!(repos.iter().any(|r| r.display_name() == "@work/project1"));
    assert!(repos.iter().any(|r| r.display_name() == "@personal/blog"));
}

#[test]
fn test_single_repository_management() {
    let temp = TempDir::new().unwrap();
    let single_repo = temp.path().join("standalone");
    create_mock_repo(&single_repo);
    
    let main_dirs: Vec<(usize, &PathBuf, Option<usize>)> = vec![];
    let single_repos = vec![single_repo.clone()];
    
    let repos = discover_repositories_multi(&main_dirs, &single_repos, &Default::default()).unwrap();
    
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].source.scope(), "single");
    assert_eq!(repos[0].display_name(), "@single/standalone");
}

fn create_mock_repo(path: &PathBuf) {
    std::fs::create_dir_all(path.join(".git")).unwrap();
}
```

#### 2.6.3 测试文件: `tests/config_migration_test.rs`

```rust
//! 配置迁移测试

use repotui::config::{Config, load_config, save_config};
use repotui::config::migration::migrate_config;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_v1_to_v2_migration() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.toml");
    
    // 创建 v1 配置
    let v1_config = r#"
version = "1.0"
main_directory = "/home/user/projects"

[editors]
vscode = "/usr/bin/code"
"#;
    
    std::fs::write(&config_path, v1_config).unwrap();
    
    // 加载并迁移
    let mut config: Config = toml::from_str(v1_config).unwrap();
    migrate_config(&mut config).unwrap();
    
    // 验证迁移结果
    assert_eq!(config.version, "2.0");
    assert!(config.main_directory.is_none());
    assert_eq!(config.main_directories.len(), 1);
    assert_eq!(config.main_directories[0].path, PathBuf::from("/home/user/projects"));
    assert!(config.main_directories[0].enabled);
}

#[test]
fn test_empty_main_directory_migration() {
    let v1_config = r#"
version = "1.0"
main_directory = ""
"#;
    
    let mut config: Config = toml::from_str(v1_config).unwrap();
    migrate_config(&mut config).unwrap();
    
    // 空路径不应添加到主目录列表
    assert!(config.main_directories.is_empty());
}
```

---

## 3. 向后兼容性方案

### 3.1 配置迁移逻辑

**自动迁移流程:**

1. **启动时检测** - 检查 `config.version` 字段
2. **版本识别** - 如果版本 < "2.0" 或缺少 `main_directories` 字段
3. **执行迁移** - 调用 `Config::migrate()` 方法
4. **备份旧配置** - 保存 `.toml.backup.{timestamp}`
5. **保存新配置** - 写入新的配置格式

**迁移场景处理:**

| 场景 | 处理方式 |
|------|----------|
| 只有 `main_directory` | 迁移为单个主目录 |
| 已有 `main_directories` | 跳过迁移 |
| 两者都存在 | 合并去重 |
| `favorites` 中有独立仓库 | 迁移到 `single_repositories` |
| 配置损坏 | 创建新配置，提示用户 |

### 3.2 版本检测

```rust
// src/constants.rs
pub const CONFIG_VERSION: &str = "2.0";
pub const MIN_SUPPORTED_VERSION: &str = "1.0";

// src/config/load.rs
pub fn load_config() -> AppResult<Config> {
    let config = load_raw_config()?;
    
    // 检查版本兼容性
    if !is_version_supported(&config.version) {
        return Err(ConfigError::UnsupportedVersion(config.version).into());
    }
    
    // 自动迁移
    if config.needs_migration() {
        log::info!("Auto-migrating config from version {}", config.version);
        let mut config = config;
        config.migrate();
        save_config(&config)?;
    }
    
    Ok(config)
}
```

---

## 4. 风险评估与应对

### 4.1 可能的问题

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| 配置迁移失败 | 中 | 高 | 自动备份，失败时回滚到备份 |
| 性能下降（多目录扫描） | 中 | 中 | 并行扫描，添加缓存 |
| 同名仓库冲突 | 中 | 中 | 路径去重，显示完整路径 |
| UI 复杂性增加 | 高 | 低 | 组件化设计，清晰文档 |
| 快捷键冲突 | 低 | 低 | 更新帮助文档，提供配置 |
| 向后兼容性问题 | 中 | 高 | 全面测试，渐进式发布 |

### 4.2 回滚方案

**配置回滚:**

```bash
# 如果迁移失败，用户可以手动恢复
cp ~/.config/repotui/config.toml.backup.20240308_120000 \
   ~/.config/repotui/config.toml
```

**代码回滚:**
- 使用 git tag 标记发布版本
- 每个 Phase 独立分支
- 保留旧版本代码路径（feature flag）

**功能开关:**

```rust
// src/constants.rs
pub const ENABLE_MULTI_DIRECTORY: bool = true;

// 使用条件编译或运行时检查
#[cfg(feature = "multi-dir")]
// 新代码

#[cfg(not(feature = "multi-dir"))]
// 旧代码
```

### 4.3 发布策略

**Alpha 阶段:**
- 内部测试，主要关注配置迁移
- 功能开关默认关闭

**Beta 阶段:**
- 小范围用户测试
- 功能开关默认开启，可关闭

**正式发布:**
- 功能开关移除
- 旧代码路径删除

---

## 5. 实施时间线

```
Week 1-2:
├── Phase 1: 配置层重构 (2-3天)
├── Phase 2: 数据结构重构 (2天)
└── 代码审查 + 测试 (1-2天)

Week 3:
├── Phase 3: 状态机重构 (2-3天)
├── Phase 4: UI 层重构 (3-4天)
└── 集成测试 (并行)

Week 4:
├── Phase 5: 键盘处理重构 (2天)
├── Phase 6: 集成测试 (2-3天)
├── 性能优化 (1天)
└── 文档更新 (1天)

总计: 4 周
```

---

## 6. 附录

### 6.1 快捷键映射表

| 快捷键 | 原功能 | 新功能 |
|--------|--------|--------|
| `m` | 打开目录选择器 | 打开主目录管理器 |
| `a` | - | 打开目录选择器 |
| `A` | - | 添加独立仓库 |

### 6.2 配置示例

**v2.0 配置格式:**

```toml
version = "2.0"

[[main_directories]]
path = "/home/user/work"
display_name = "Work"
enabled = true
max_depth = 2

[[main_directories]]
path = "/home/user/personal"
display_name = "Personal"
enabled = true

[[single_repositories]]
path = "/opt/projects/special"
display_name = "Special Project"
added_at = "2024-03-08T10:30:00+08:00"

[editors]
vscode = "/usr/bin/code"

[ui]
theme = "dark"
```

---

**文档版本:** 1.0  
**创建日期:** 2024-03-08  
**最后更新:** 2024-03-08
