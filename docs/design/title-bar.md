# 标题栏设计文档

**文档版本**: 1.0  
**创建日期**: 2026-03-08  
**状态**: 已实施

---

## 📋 目录

1. [设计概述](#设计概述)
2. [UI 设计](#ui 设计)
3. [功能规格](#功能规格)
4. [主题集成](#主题集成)
5. [技术实现](#技术实现)
6. [测试验证](#测试验证)

---

## 设计概述

### 核心目标

在界面顶部添加标题栏，用于显示当前视图状态，帮助用户快速了解所在位置。

### 设计原则

| 原则 | 说明 |
|------|------|
| **简洁** | 高度仅 1 行，不影响主要内容显示 |
| **清晰** | 标题文字清晰可读，使用主题色 |
| **一致** | 与现有 UI 风格保持一致 |
| **主题化** | 不同主题有不同的标题颜色效果 |

---

## UI 设计

### 布局结构

```
┌─────────────────────────────────────────────────────────────┐
│ ╭─ repotui — 全部视图 ────────────────────────────────────╮ │ 1 行 (标题栏)
├─┼───────────────────────────────────────────────────────────┼─┤
│ │ 🔍 Search: [...]                                    [1/5] │ │ 3 行 (搜索框)
│ ╰───────────────────────────────────────────────────────────╯ │
├───────────────────────────────────────────────────────────────┤
│ ─ Repositories ────────────────────────────────────────────╮ │
│ │ ▌ repo1                          main    ● dirty         │ │
│ │   repo2                          feat    ✓ clean         │ │ N 行 (列表)
│ │ ...                                                       │ │
│ ╰───────────────────────────────────────────────────────────╯ │
├───────────────────────────────────────────────────────────────┤
│ ~/repos  [5 repos]  •  ↑↓ navigate  / search  ENTER open     │ │ 2 行 (状态栏)
└───────────────────────────────────────────────────────────────┘
```

### 标题栏内容

| 视图模式 | 标题显示 | 快捷键 |
|----------|----------|--------|
| 全部视图 | `repotui — 全部视图` | - |
| 收藏夹 | `repotui — 收藏夹` | `Ctrl+f` |
| 最近视图 | `repotui — 最近视图` | `Ctrl+r` |
| 多选模式 | `repotui — 多选模式 (已选 n 个)` | `v` 进入 |

### 视觉样式

**边框**: 使用 `border_focused` 颜色  
**标题文字**: 使用 `title_fg` 和 `title_bg` 颜色  
**背景**: 与主题背景色一致

---

## 功能规格

### 视图模式显示

标题栏实时反映当前视图模式：

```rust
pub enum ViewMode {
    All,       // → "全部视图"
    Favorites, // → "收藏夹"
    Recent,    // → "最近视图"
}
```

### 多选模式状态

当进入多选模式时，标题栏显示选中数量：

```
repotui — 多选模式 (已选 3 个)
```

### 交互行为

- 标题栏为**只读显示**，不支持点击交互
- 视图切换通过快捷键完成
- 标题内容自动更新

---

## 主题集成

### 颜色字段

在 `ColorPalette` 中添加标题专用颜色：

```rust
pub struct ColorPalette {
    // ... 现有字段 ...
    pub title_fg: ColorRgb,   // 标题前景色
    pub title_bg: ColorRgb,   // 标题背景色
}
```

### 主题示例

#### Dark Theme
```rust
title_fg: ColorRgb { r: 248, g: 248, b: 242 }  // 亮白色
title_bg: ColorRgb { r: 9, g: 9, b: 11 }       // 深黑色
```

#### Light Theme
```rust
title_fg: ColorRgb { r: 9, g: 9, b: 11 }       // 深黑色
title_bg: ColorRgb { r: 255, g: 255, b: 255 }  // 纯白色
```

#### Nord Theme
```rust
title_fg: ColorRgb { r: 216, g: 222, b: 233 }  // 北极白
title_bg: ColorRgb { r: 47, g: 52, b: 64 }     // 北极深蓝
```

### 样式方法

```rust
impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.colors.title_fg.into())
            .bg(self.colors.title_bg.into())
            .add_modifier(Modifier::BOLD)
    }
}
```

---

## 技术实现

### 组件结构

```rust
pub struct TitleBar<'a> {
    view_mode: &'a ViewMode,
    theme: &'a Theme,
    selection_mode: bool,
    selected_count: usize,
}
```

### API

```rust
impl<'a> TitleBar<'a> {
    /// 创建标题栏
    pub fn new(view_mode: &'a ViewMode, theme: &'a Theme) -> Self;
    
    /// 设置多选信息
    pub fn selection_info(mut self, selected_count: usize) -> Self;
}
```

### 布局集成

修改 `render_main_ui` 函数：

```rust
let chunks = Layout::default()
    .constraints([
        Constraint::Length(1), // Title bar
        Constraint::Length(3), // Search box
        Constraint::Min(5),    // Repository list
        Constraint::Length(2), // Status bar
    ])
    .split(area);
```

### 最小终端尺寸

更新最小高度要求：

```rust
pub const MIN_TERMINAL_HEIGHT: u16 = 25; // 原 24 + 1 行标题栏
```

---

## 测试验证

### 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_title_bar_all_view() { /* ... */ }
    
    #[test]
    fn test_title_bar_favorites_view() { /* ... */ }
    
    #[test]
    fn test_title_bar_recent_view() { /* ... */ }
    
    #[test]
    fn test_title_bar_selection_mode() { /* ... */ }
}
```

### 验证清单

- [ ] 所有 8 个主题下标题栏正确显示
- [ ] 视图切换时标题内容正确更新
- [ ] 多选模式显示选中数量
- [ ] 终端尺寸检查更新
- [ ] 无布局溢出或截断
- [ ] 键盘快捷键正常工作

---

## 文件变更

### 新增文件

- `src/ui/widgets/title_bar.rs` - 标题栏组件

### 修改文件

- `src/ui/widgets/mod.rs` - 导出 TitleBar
- `src/ui/theme.rs` - 添加标题颜色字段和方法
- `src/ui/render.rs` - 集成标题栏布局
- `src/ui/layout.rs` - 更新布局常量和函数
- `src/constants.rs` - 更新最小高度
- `src/ui/themes/*.rs` - 8 个主题文件添加标题颜色

---

## 相关文档

- [CLAUDE.md](../CLAUDE.md) - 开发规范
- [THEME_SYSTEM_PLAN.md](./THEME_SYSTEM_PLAN.md) - 主题系统
- [KEYBOARD_SHORTCUTS.md](../KEYBOARD_SHORTCUTS.md) - 快捷键

---

**实施状态**: ✅ 完成  
**测试状态**: 待验证  
**下次审查**: Phase 完成后
