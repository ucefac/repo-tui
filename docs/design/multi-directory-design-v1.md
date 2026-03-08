# 主目录管理功能设计文档

**文档版本**: 1.0  
**创建日期**: 2026-03-08  
**状态**: 待实施  
**关联**: 主目录管理  
**优先级**: P0

---

## 📋 目录

1. [功能概述](#功能概述)
2. [界面布局规范](#界面布局规范)
3. [视觉规范](#视觉规范)
4. [交互规范](#交互规范)
5. [组件规范](#组件规范)
6. [状态流转图](#状态流转图)

---

## 功能概述

### 核心功能

| 功能 | 说明 | 优先级 |
|------|------|--------|
| 主目录管理 | 添加、移除、切换多个主目录 | P0 |
| 目录选择器 | 重构为通用组件，支持两种模式 | P0 |
| 仓库分组显示 | 按主目录分组，使用 scope 颜色区分 | P0 |

### 用户故事

1. **作为用户**，我希望管理多个主目录，以便在不同项目集合间快速切换
2. **作为用户**，我希望目录选择器支持选择单个 git 仓库，以便直接打开特定项目
3. **作为用户**，我希望通过 scope 颜色区分不同主目录的仓库，以便快速识别来源

---

## 界面布局规范

### 1. 主目录管理界面

```
┌─────────────────────────────────────────────────────────────┐
│ repotui — 主目录管理                                          │  ← 标题栏
├─────────────────────────────────────────────────────────────┤
│ [🔍 search...                              ]                │  ← 搜索框
├─────────────────────────────────────────────────────────────┤
│  📂 /home/user/Projects                    (12 repos)  ▌    │  ← 选中项高亮
│  📂 /home/user/work                         (8 repos)       │
│  📂 /home/user/experiments                  (5 repos)       │
│  📂 /home/user/open-source                  (3 repos)       │
│                                                              │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│ a add  d remove  Enter switch  Esc return                    │  ← 底部帮助栏
└─────────────────────────────────────────────────────────────┘
```

**布局结构**:

| 区域 | 高度 | 内容 |
|------|------|------|
| 标题栏 | 1行 | `repotui — 主目录管理` |
| 搜索框 | 1行 | 支持实时过滤，与仓库列表界面一致 |
| 主目录列表 | 自适应 | 显示所有主目录路径和仓库数量 |
| 底部帮助栏 | 1行 | 操作提示 |

**列表项格式**:
```
📂 {path}    ({count} repos)    [active indicator]
```

- `📂`: 文件夹图标，使用 `text_muted` 颜色
- `{path}`: 主目录路径，使用 `foreground` 颜色
- `({count} repos)`: 仓库数量，使用 `text_muted` 颜色，右对齐
- `[active indicator]`: 当前激活目录标记 `▌`，使用 `primary` 颜色

### 2. 目录选择器（通用组件）

#### 模式 A: 选择主目录

```
┌─────────────────────────────────────────────────────────────┐
│  📁 添加主目录                                                │  ← 标题
├─────────────────────────────────────────────────────────────┤
│  📂 /home/user/Projects                                       │  ← 当前路径
├─────────────────────────────────────────────────────────────┤
│  ┌─ Directories (8/15) ─────────────────────────────────┐   │
│  │                                                       │   │
│  │ ▌ 📁 project-a                                       │   │  ← 选中
│  │   📁 project-b                                       │   │
│  │   📁 project-c                                       │   │
│  │   📁 project-d                                       │   │
│  │                                                       │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ↑↓ navigate   ← back   → enter   SPACE confirm   Esc cancel │  ← 帮助
└─────────────────────────────────────────────────────────────┘
```

#### 模式 B: 选择单个 Git 仓库

```
┌─────────────────────────────────────────────────────────────┐
│  📁 选择 Git 仓库                                             │  ← 标题
├─────────────────────────────────────────────────────────────┤
│  📂 /home/user/Projects                                       │  ← 当前路径
├─────────────────────────────────────────────────────────────┤
│  ┌─ Git Repositories (5/5) ─────────────────────────────┐   │
│  │                                                       │   │
│  │ ▌ 📁 my-app                    (main)  ● Modified    │   │  ← 选中
│  │   📁 api-server                (dev)   ● Clean       │   │
│  │   📁 frontend                  (main)  ● Modified    │   │
│  │   📁 utils                     (v1.2)  ● Clean       │   │
│  │   📁 docs                      (main)  ● Modified    │   │
│  │                                                       │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  ↑↓ navigate   ← back   → enter   SPACE open   Esc cancel   │  ← 帮助
└─────────────────────────────────────────────────────────────┘
```

**布局结构**:

| 区域 | 高度 | 内容 |
|------|------|------|
| 标题 | 1行 | 根据模式变化：`添加主目录` / `选择 Git 仓库` |
| 当前路径 | 1行 | 显示当前浏览的目录路径 |
| 目录列表 | 自适应 | 显示子目录（模式A）或 git 仓库（模式B） |
| 帮助栏 | 1行 | 操作提示 |

### 3. 仓库列表显示格式（变更）

```
┌─────────────────────────────────────────────────────────────┐
│ repotui — 全部视图                                            │
├─────────────────────────────────────────────────────────────┤
│ [🔍 search...                              ]                │
├─────────────────────────────────────────────────────────────┤
│  ┌─ Repositories (28/28) ───────────────────────────────┐   │
│  │                                                       │   │
│  │ ▌ @Projects/my-app              (main)  ● Modified   │   │  ← Projects 颜色
│  │   @Projects/api-server          (dev)   ● Clean      │   │
│  │   @work/backend                 (main)  ● Modified   │   │  ← work 颜色
│  │   @work/frontend                (main)  ● Clean      │   │
│  │   @experiments/rust-game        (main)  ● Modified   │   │  ← experiments 颜色
│  │   @stand/single-repo            (main)  ● Clean      │   │  ← stand 颜色
│  │                                                       │   │
│  └───────────────────────────────────────────────────────┘   │
│                                                              │
│  📂 3 directories | 🗂️ 28 repos                              │
└─────────────────────────────────────────────────────────────┘
```

**显示格式**:

```
@scope/repo-name    (branch)  ● status
```

- `@scope`: 主目录标识，使用 scope 特定颜色
- `/repo-name`: 仓库名称，使用 `foreground` 颜色
- `(branch)`: 分支名，使用 `secondary` 颜色
- `●`: 状态圆点，绿色（Clean）/ 红色（Modified）
- `status`: 状态文本，使用对应颜色

**Scope 颜色分配**:

| Scope | 颜色 | 来源 |
|-------|------|------|
| `@stand` | `text_muted` | 单个添加的仓库 |
| 主目录1 | `primary` | 第一个主目录 |
| 主目录2 | `secondary` | 第二个主目录 |
| 主目录3 | `success` | 第三个主目录 |
| 主目录4+ | `warning` / `error` | 循环使用 |

---

## 视觉规范

### 颜色使用

基于现有主题系统（`src/ui/theme.rs`）：

| 用途 | 颜色字段 | 说明 |
|------|----------|------|
| 选中背景 | `selected_bg` | 列表项选中时的背景色 |
| 选中文本 | `selected_fg` | 列表项选中时的前景色（通常为白色） |
| 主要强调 | `primary` | 标题、选中标记、scope 颜色1 |
| 次要强调 | `secondary` | 分支名、scope 颜色2 |
| 成功/干净 | `success` | Clean 状态、scope 颜色3 |
| 警告/修改 | `warning` | Modified 状态提示 |
| 错误 | `error` | 错误信息、删除确认 |
| 普通文本 | `foreground` | 仓库名、路径 |
| 弱化文本 | `text_muted` | 数量统计、图标 |
| 边框 | `border` | 列表框边框 |
| 聚焦边框 | `border_focused` | 选中项边框 |

### 列表项样式

**普通状态**:
```
  📂 /home/user/Projects                    (12 repos)
```
- 图标：`text_muted`
- 路径：`foreground`
- 数量：`text_muted`，右对齐

**选中状态**:
```
▌ 📂 /home/user/Projects                    (12 repos)
```
- 选中标记：`▌`，使用 `primary` 颜色
- 背景：`selected_bg`
- 前景：`selected_fg`（加粗）

### 帮助提示样式

**格式**:
```
{key} {description}   {key} {description}
```

**样式**:
- 按键：`primary` 颜色
- 描述：`text_muted` 颜色
- 分隔：`3个空格`

### 反馈消息样式

**成功提示**:
```
✅ 主目录已添加: /home/user/Projects
```
- 图标：`✅` 使用 `success` 颜色
- 文本：`foreground` 颜色

**确认提示**:
```
⚠️  确认移除主目录 /home/user/Projects? (y/n)
```
- 图标：`⚠️` 使用 `warning` 颜色
- 确认选项：`y` 使用 `success`，`n` 使用 `error`

---

## 交互规范

### 键盘导航

#### 主目录管理界面

| 按键 | 功能 | 说明 |
|------|------|------|
| `↑/↓` | 上/下移动 | **循环滚动**，在列表顶部按上键跳到底部 |
| `Space` | 添加主目录 | 打开目录选择器（模式A） |
| `d` | 移除主目录 | 显示确认提示 |
| `→` / `Enter` | 切换到该目录 | 切换当前主目录并返回仓库列表 |
| `Esc` | 返回 | 返回仓库列表 |

#### 目录选择器（通用）

| 按键 | 功能 | 说明 |
|------|------|------|
| `↑/↓` | 上/下移动 | **循环滚动** |
| `←/h` | 返回上级 | 目录导航 |
| `→/l` | 进入目录 | 进入选中的子目录 |
| `Space` | 确认/打开 | 模式A：选择当前目录；模式B：打开仓库 |
| `Esc/q` | 取消 | 关闭选择器 |

### 状态切换流程

#### 添加主目录流程

```
仓库列表
    │
    │ 按 `m`
    ▼
主目录管理界面
    │
    │ 按 `Space`
    ▼
目录选择器（模式A：添加主目录）
    │
    │ 选择目录 + Space
    ▼
返回主目录管理界面（显示成功提示）
    │
    │ 按 `Esc`
    ▼
仓库列表（自动刷新）
```

#### 移除主目录流程

```
主目录管理界面
    │
    │ 按 `d`
    ▼
显示确认提示：确认移除 /path/to/dir? (y/n)
    │
    ├─ 按 `y` → 移除 + 显示成功提示
    │
    └─ 按 `n` → 取消
```

#### 切换主目录流程

```
主目录管理界面
    │
    │ 选择目标目录 + Enter/→
    ▼
仓库列表（显示该目录下的仓库）
```

#### 选择单个 Git 仓库流程

```
仓库列表
    │
    │ 按 `o`（open single）
    ▼
目录选择器（模式B：选择 Git 仓库）
    │
    │ 导航到目标仓库 + Space
    ▼
打开操作菜单（针对该仓库）
    │
    │ 选择操作 / Esc
    ▼
返回仓库列表
```

### 操作反馈

| 操作 | 反馈类型 | 反馈内容 |
|------|----------|----------|
| 添加成功 | 成功提示 | `✅ 主目录已添加: {path}` |
| 添加失败 | 错误提示 | `❌ 无法添加: {reason}` |
| 移除确认 | 确认提示 | `⚠️ 确认移除 {path}? (y/n)` |
| 移除成功 | 成功提示 | `✅ 主目录已移除` |
| 切换成功 | 状态更新 | 标题栏更新，列表刷新 |
| 打开仓库 | 操作菜单 | 显示仓库操作选项 |

---

## 组件规范

### 主目录列表项

```rust
pub struct MainDirItem {
    /// 主目录路径
    pub path: PathBuf,
    /// 显示名称（路径的最后一部分或完整路径）
    pub display_name: String,
    /// 仓库数量
    pub repo_count: usize,
    /// 是否是当前激活的目录
    pub is_active: bool,
    /// 颜色索引（用于 scope 颜色）
    pub color_index: usize,
}
```

**显示格式**:
```
{active_indicator} {icon} {path:<width$} {count:>10}
```

- `active_indicator`: `▌` 或两个空格
- `icon`: `📂`
- `path`: 左对齐，自动截断中间部分
- `count`: 右对齐，格式为 `(n repos)`

### 目录选择器组件

```rust
pub struct DirChooser<'a> {
    /// 当前目录路径
    pub current_path: &'a Path,
    /// 目录条目
    pub entries: &'a [DirEntry],
    /// 选中索引
    pub selected_index: usize,
    /// 模式
    pub mode: DirChooserMode,
    /// 主题
    pub theme: &'a Theme,
}

pub enum DirChooserMode {
    /// 选择主目录
    SelectMainDir,
    /// 选择单个 Git 仓库
    SelectGitRepo,
}
```

**条目结构**:
```rust
pub struct DirEntry {
    /// 条目名称
    pub name: String,
    /// 是否是 Git 仓库
    pub is_git_repo: bool,
    /// Git 分支（如果是仓库）
    pub branch: Option<String>,
    /// 是否有修改（如果是仓库）
    pub is_dirty: Option<bool>,
}
```

### 仓库显示格式

```rust
pub struct RepoDisplay {
    /// Scope（主目录标识）
    pub scope: String,
    /// 仓库名称
    pub name: String,
    /// 分支名
    pub branch: Option<String>,
    /// 是否有修改
    pub is_dirty: bool,
    /// Scope 颜色
    pub scope_color: Color,
}
```

**格式化函数**:
```rust
fn format_repo_item(repo: &RepoDisplay, theme: &Theme) -> Line<'static> {
    // @scope/repo-name    (branch)  ● status
}
```

### Scope 颜色映射

```rust
impl Theme {
    pub fn scope_color(&self, index: usize) -> Color {
        let colors = [
            self.colors.primary,
            self.colors.secondary,
            self.colors.success,
            self.colors.warning,
            self.colors.error,
        ];
        colors[index % colors.len()].into()
    }
}
```

---

## 状态流转图

### 完整状态流转

```
                         ┌─────────────────────────────────────┐
                         │                                     │
                         ▼                                     │
┌─────────┐    m     ┌──────────┐    Space    ┌────────────┐   │
│ Running │ ───────▶ │ Managing │ ───────────▶│ Choosing   │   │
│(仓库列表)│◀─────── │ (主目录  │◀────────────│ Dir A      │   │
└────┬────┘   Esc    │  管理)   │    Esc      │(选择主目录)│   │
     │               └────┬─────┘             └────────────┘   │
     │                    │ d                                 │
     │                    ▼                                   │
     │               ┌──────────┐                            │
     │               │ Confirm  │◀───── y ───────────────────┤
     │               │  Remove  │      n                     │
     │               └──────────┘                            │
     │                                                       │
     │ o                                                     │
     ▼                                                       │
┌────────────┐    Space/Enter    ┌───────────┐              │
│ Choosing   │ ─────────────────▶│ Showing   │              │
│ Dir B      │                   │ Actions   │              │
│(选择git仓库)│◀──────────────────│ (操作菜单)│              │
└────────────┘        Esc        └───────────┘              │
     ▲                                                      │
     │                                                      │
     └──────────────────────────────────────────────────────┘
```

### 关键状态定义

```rust
pub enum AppState {
    /// 正常状态 - 显示仓库列表
    Running,
    
    /// 主目录管理界面
    ManagingMainDirs {
        /// 主目录列表
        dirs: Vec<MainDir>,
        /// 选中索引
        selected_index: usize,
        /// 是否显示确认对话框
        confirm_remove: Option<usize>,
    },
    
    /// 目录选择器（重构）
    ChoosingDir {
        path: PathBuf,
        entries: Vec<DirEntry>,
        selected_index: usize,
        scroll_offset: usize,
        mode: DirChooserMode,
    },
    
    // ... 其他状态
}
```

### 状态优先级

```
ShowingActions (5) > ShowingHelp (4) > ManagingMainDirs (3) > 
ChoosingDir (3) > SelectingTheme (3) > Running (1)
```

---

## 数据结构

### 配置扩展

```rust
pub struct Config {
    // 现有字段...
    
    /// 主目录列表（新增）
    pub main_directories: Vec<MainDirConfig>,
    
    /// 当前激活的主目录索引（新增）
    pub active_directory_index: usize,
}

pub struct MainDirConfig {
    /// 目录路径
    pub path: PathBuf,
    /// 是否启用
    pub enabled: bool,
    /// 自定义显示名称（可选）
    pub display_name: Option<String>,
}
```

### 仓库结构扩展

```rust
pub struct Repository {
    // 现有字段...
    
    /// 所属主目录索引（新增）
    pub main_dir_index: usize,
    
    /// Scope 名称（从主目录路径派生）
    pub scope: String,
}
```

---

## 验收标准

### 功能验收

- [ ] 支持添加多个主目录
- [ ] 支持移除主目录（带确认）
- [ ] 支持切换当前主目录
- [ ] 目录选择器支持两种模式
- [ ] 仓库列表显示 `@scope/repo-name` 格式
- [ ] 不同 scope 使用不同颜色

### 视觉验收

- [ ] 界面风格与现有组件一致
- [ ] 选中状态清晰可见
- [ ] scope 颜色区分明显
- [ ] 帮助提示格式统一

### 交互验收

- [ ] 键盘导航支持循环滚动
- [ ] 所有操作有反馈
- [ ] 删除操作有确认
- [ ] 状态切换流畅

---

## 相关文档

- [UI 设计规范](./ui-guidelines.md)
- [键盘快捷键](./keyboard-shortcuts.md)
- [主题系统](./theme-system.md)

---

**最后更新**: 2026-03-08  
**维护者**: repotui Team
