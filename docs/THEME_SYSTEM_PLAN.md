# 主题系统实施计划

**文档版本**: 1.0  
**创建日期**: 2026-03-07  
**状态**: 待实施  
**优先级**: P0

---

## 📋 目录

1. [需求概述](#需求概述)
2. [当前架构分析](#当前架构分析)
3. [目标架构设计](#目标架构设计)
4. [文件结构变更](#文件结构变更)
5. [实施步骤](#实施步骤)
6. [测试策略](#测试策略)
7. [验收标准](#验收标准)
8. [风险评估](#风险评估)

---

## 需求概述

### 核心需求

| 需求 | 描述 | 优先级 |
|------|------|--------|
| 终端独立 | 主题颜色不受终端配色影响，使用 RGB 真彩色 | P0 |
| 多主题支持 | 支持 7 个内置主题（dark, light, nord, dracula, gruvbox_dark, tokyo_night, catppuccin_mocha） | P0 |
| 运行时切换 | 可在软件运行期间无缝切换主题，配置自动保存 | P0 |
| 主题预览 | 切换时即时预览效果 | P1 |
| 自定义主题 | 允许用户通过配置文件自定义主题颜色 | P2 |

### 用户故事

1. **作为用户**，我希望按 `t` 键快速切换主题，以便在不同光线环境下获得舒适的视觉体验
2. **作为用户**，我希望在主题菜单中选择特定主题，而不是仅支持 dark/light 切换
3. **作为用户**，我希望主题切换后自动保存，下次启动时保持我的选择
4. **作为用户**，我希望主题切换是即时的，不需要重启应用

---

## 当前架构分析

### 现有主题架构

```
src/
├── ui/
│   └── theme.rs          # Theme 结构 + dark/light 实现
├── constants.rs          # 颜色常量定义 (dark/light)
└── config/
    └── types.rs          # UiConfig.theme (String)
```

### 当前实现分析

#### `src/ui/theme.rs` (296 行)

**核心结构**:
- `ColorRgb`: RGB 颜色表示
- `ColorPalette`: 12 色颜色面板
- `Theme`: 主题结构，含 name 和 colors

**当前方法**:
- `dark()`: 创建 dark 主题
- `light()`: 创建 light 主题
- `from_config(&str)`: 从配置加载（仅支持 dark/light）
- `toggle()`: 在 dark/light 之间切换
- 样式方法：`selected_style()`, `focused_border_style()`, 等

**局限性**:
1. ❌ 硬编码主题构造函数，不支持扩展
2. ❌ `toggle()` 仅支持 2 个主题切换
3. ❌ `from_config()` 不支持新主题名称

#### `src/constants.rs` (233 行)

**当前结构**:
```rust
pub mod ui {
    pub mod dark { /* 颜色常量 */ }
    pub mod light { /* 颜色常量 */ }
}
```

**问题**:
1. ❌ 颜色常量分散，新主题需要手动添加模块
2. ❌ 与 `theme.rs` 存在重复定义

#### `src/app/model.rs`

**主题相关字段**:
```rust
pub struct App {
    pub theme: Theme,
    // ...
}
```

#### `src/app/update.rs`

**主题切换逻辑** (第 378-400 行):
```rust
AppMsg::ThemeChanged => {
    app.theme = app.theme.toggle();  // ❌ 仅支持 2 主题
    if let Some(ref mut config) = app.config {
        config.ui.theme = app.theme.name.clone();
        config::save_config(config)?;
    }
}
```

#### `src/handler/keyboard.rs`

**快捷键触发** (第 299-302 行):
```rust
KeyCode::Char('t') => {
    let _ = app.msg_tx.try_send(AppMsg::ThemeChanged);
}
```

---

## 目标架构设计

### 架构原则

1. **单一数据源**: 所有主题颜色定义集中在一个地方
2. **可扩展性**: 新增主题无需修改核心逻辑
3. **零性能影响**: 主题切换不影响渲染性能
4. **向后兼容**: 保持现有配置格式兼容

### 主题注册表模式

```rust
// 主题工厂模式
pub struct ThemeRegistry {
    themes: HashMap<&'static str, fn() -> Theme>,
}

impl ThemeRegistry {
    pub fn get(&self, name: &str) -> Option<Theme>;
    pub fn list(&self) -> Vec<&'static str>;
}
```

### 7 个内置主题色板

| 主题 | 类型 | 主色调 | 特点 |
|------|------|--------|------|
| dark | 深色 | #58A6FF | 现有主题，蓝色系 |
| light | 浅色 | #096ADA | 现有主题，蓝色系 |
| nord | 冷色调 | #88C0D0 | 北欧极地风格，蓝绿色 |
| dracula | 深色 | #BD93F9 | 流行深色，紫色系 |
| gruvbox_dark | 深色 | #FE8019 | 复古暖色，橙棕色 |
| tokyo_night | 深色 | #7AA2F7 | 现代深色，蓝紫色 |
| catppuccin_mocha | 深色 | #89B4FA | 流行深色，柔和蓝色 |

### 完整色板定义（12 色）

每个主题需要定义以下颜色：

```rust
pub struct ColorPalette {
    pub primary: ColorRgb,        // 主色调
    pub secondary: ColorRgb,      // 辅助色
    pub success: ColorRgb,        // 成功/绿色
    pub warning: ColorRgb,        // 警告/黄色
    pub error: ColorRgb,          // 错误/红色
    pub background: ColorRgb,     // 背景色
    pub foreground: ColorRgb,     // 前景/文字色
    pub border: ColorRgb,         // 边框色
    pub selected_bg: ColorRgb,    // 选中背景
    pub selected_fg: ColorRgb,    // 选中前景
    pub text_muted: ColorRgb,     // 弱化文字
    pub border_focused: ColorRgb, // 聚焦边框
}
```

---

## 文件结构变更

### 新增文件

```
src/ui/
├── theme.rs              # 重构：Theme 核心结构
├── themes/               # 新增：主题定义目录
│   ├── mod.rs            # 主题注册表
│   ├── dark.rs           # Dark 主题定义
│   ├── light.rs          # Light 主题定义
│   ├── nord.rs           # Nord 主题定义
│   ├── dracula.rs        # Dracula 主题定义
│   ├── gruvbox_dark.rs   # Gruvbox Dark 主题定义
│   ├── tokyo_night.rs    # Tokyo Night 主题定义
│   └── catppuccin_mocha.rs # Catppuccin Mocha 主题定义
│
tests/
├── unit/
│   └── ui/
│       ├── theme_test.rs      # 主题单元测试
│       └── themes/            # 各主题测试
│           ├── nord_test.rs
│           └── ...
```

### 修改文件清单

| 文件 | 变更类型 | 变更内容 | 行数变化 |
|------|----------|----------|----------|
| `src/ui/theme.rs` | 重构 | 移除硬编码主题，改为通用结构 | ~150 行 → ~80 行 |
| `src/constants.rs` | 简化 | 移除颜色常量，仅保留配置常量 | ~233 行 → ~150 行 |
| `src/app/msg.rs` | 修改 | 扩展 `ThemeChanged` 消息 | +5 行 |
| `src/app/update.rs` | 修改 | 更新主题切换逻辑 | ~20 行变更 |
| `src/handler/keyboard.rs` | 修改 | `t` 键改为打开主题菜单 | ~10 行变更 |
| `src/ui/mod.rs` | 修改 | 导出新主题模块 | +5 行 |
| `src/ui/widgets/` | 新增 | `ThemeSelector` 组件 | ~150 行 |
| `src/config/types.rs` | 兼容 | 无变更（已支持字符串主题名） | 0 行 |
| `src/config/load.rs` | 验证 | 添加主题名验证 | +15 行 |
| `src/ui/widgets/mod.rs` | 修改 | 导出新组件 | +2 行 |

---

## 实施步骤

### Phase 1: 主题定义层（预计 4 小时）

#### Step 1.1: 创建主题模块结构

**文件**: `src/ui/themes/mod.rs`

```rust
//! Built-in theme definitions

mod dark;
mod light;
mod nord;
mod dracula;
mod gruvbox_dark;
mod tokyo_night;
mod catppuccin_mocha;

pub use dark::dark_theme;
pub use light::light_theme;
pub use nord::nord_theme;
pub use dracula::dracula_theme;
pub use gruvbox_dark::gruvbox_dark_theme;
pub use tokyo_night::tokyo_night_theme;
pub use catppuccin_mocha::catppuccin_mocha_theme;

use crate::ui::theme::Theme;

/// Available theme names
pub const THEME_NAMES: &[&str] = &[
    "dark",
    "light",
    "nord",
    "dracula",
    "gruvbox_dark",
    "tokyo_night",
    "catppuccin_mocha",
];

/// Get theme by name
pub fn get_theme(name: &str) -> Option<Theme> {
    match name {
        "dark" => Some(dark_theme()),
        "light" => Some(light_theme()),
        "nord" => Some(nord_theme()),
        "dracula" => Some(dracula_theme()),
        "gruvbox_dark" => Some(gruvbox_dark_theme()),
        "tokyo_night" => Some(tokyo_night_theme()),
        "catppuccin_mocha" => Some(catppuccin_mocha_theme()),
        _ => None,
    }
}

/// Get default theme name
pub fn default_theme_name() -> &'static str {
    "dark"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_themes() {
        for &name in THEME_NAMES {
            let theme = get_theme(name);
            assert!(theme.is_some(), "Theme {} should exist", name);
            assert_eq!(theme.unwrap().name, name);
        }
    }

    #[test]
    fn test_get_invalid_theme() {
        assert!(get_theme("invalid_theme").is_none());
    }
}
```

**验收标准**:
- [ ] 模块编译通过
- [ ] 单元测试通过
- [ ] `THEME_NAMES` 包含 7 个主题

#### Step 1.2: 实现 7 个主题颜色定义

**文件**: `src/ui/themes/dark.rs` (现有主题迁移)

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

pub fn dark_theme() -> Theme {
    Theme {
        name: "dark".to_string(),
        colors: ColorPalette {
            primary: ColorRgb { r: 88, g: 166, b: 255 },
            secondary: ColorRgb { r: 139, g: 92, b: 246 },
            success: ColorRgb { r: 63, g: 185, b: 80 },
            warning: ColorRgb { r: 210, g: 153, b: 34 },
            error: ColorRgb { r: 248, g: 81, b: 73 },
            background: ColorRgb { r: 9, g: 9, b: 11 },
            foreground: ColorRgb { r: 248, g: 248, b: 242 },
            border: ColorRgb { r: 63, g: 63, b: 70 },
            selected_bg: ColorRgb { r: 56, g: 139, b: 253 },
            selected_fg: ColorRgb { r: 255, g: 255, b: 255 },
            text_muted: ColorRgb { r: 107, g: 107, b: 107 },
            border_focused: ColorRgb { r: 56, g: 189, b: 248 },
        },
    }
}
```

**文件**: `src/ui/themes/light.rs`

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

pub fn light_theme() -> Theme {
    Theme {
        name: "light".to_string(),
        colors: ColorPalette {
            primary: ColorRgb { r: 9, g: 105, b: 218 },
            secondary: ColorRgb { r: 126, g: 34, b: 206 },
            success: ColorRgb { r: 26, g: 127, b: 55 },
            warning: ColorRgb { r: 154, g: 103, b: 0 },
            error: ColorRgb { r: 209, g: 36, b: 47 },
            background: ColorRgb { r: 255, g: 255, b: 255 },
            foreground: ColorRgb { r: 9, g: 9, b: 11 },
            border: ColorRgb { r: 209, g: 213, b: 219 },
            selected_bg: ColorRgb { r: 9, g: 105, b: 218 },
            selected_fg: ColorRgb { r: 255, g: 255, b: 255 },
            text_muted: ColorRgb { r: 156, g: 163, b: 175 },
            border_focused: ColorRgb { r: 37, g: 99, b: 235 },
        },
    }
}
```

**文件**: `src/ui/themes/nord.rs` (Nord 主题)

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

/// Nord Theme - Arctic North Blue
/// https://www.nordtheme.com/
pub fn nord_theme() -> Theme {
    Theme {
        name: "nord".to_string(),
        colors: ColorPalette {
            // Polar Night (背景)
            background: ColorRgb { r: 47, g: 52, b: 64 },
            // Snow Storm (前景)
            foreground: ColorRgb { r: 216, g: 222, b: 233 },
            // Frost (主色 - 蓝绿)
            primary: ColorRgb { r: 136, g: 192, b: 208 },
            secondary: ColorRgb { r: 129, g: 162, b: 190 },
            // Aurora (状态色)
            success: ColorRgb { r: 163, g: 190, b: 140 },
            warning: ColorRgb { r: 235, g: 203, b: 139 },
            error: ColorRgb { r: 191, g: 97, b: 106 },
            // Borders
            border: ColorRgb { r: 67, g: 76, b: 94 },
            selected_bg: ColorRgb { r: 67, g: 76, b: 94 },
            selected_fg: ColorRgb { r: 216, g: 222, b: 233 },
            text_muted: ColorRgb { r: 94, g: 109, b: 133 },
            border_focused: ColorRgb { r: 136, g: 192, b: 208 },
        },
    }
}
```

**文件**: `src/ui/themes/dracula.rs`

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

/// Dracula Theme - Popular dark theme
/// https://draculatheme.com/
pub fn dracula_theme() -> Theme {
    Theme {
        name: "dracula".to_string(),
        colors: ColorPalette {
            background: ColorRgb { r: 40, g: 42, b: 54 },
            foreground: ColorRgb { r: 248, g: 248, b: 242 },
            primary: ColorRgb { r: 139, g: 233, b: 253 },
            secondary: ColorRgb { r: 189, g: 147, b: 249 },
            success: ColorRgb { r: 80, g: 250, b: 123 },
            warning: ColorRgb { r: 241, g: 250, b: 140 },
            error: ColorRgb { r: 255, g: 85, b: 85 },
            border: ColorRgb { r: 62, g: 72, b: 136 },
            selected_bg: ColorRgb { r: 98, g: 114, b: 164 },
            selected_fg: ColorRgb { r: 248, g: 248, b: 242 },
            text_muted: ColorRgb { r: 98, g: 114, b: 164 },
            border_focused: ColorRgb { r: 189, g: 147, b: 249 },
        },
    }
}
```

**文件**: `src/ui/themes/gruvbox_dark.rs`

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

/// Gruvbox Dark Theme - Retro soft colors
/// https://github.com/morhetz/gruvbox
pub fn gruvbox_dark_theme() -> Theme {
    Theme {
        name: "gruvbox_dark".to_string(),
        colors: ColorPalette {
            background: ColorRgb { r: 40, g: 40, b: 40 },
            foreground: ColorRgb { r: 235, g: 219, b: 178 },
            primary: ColorRgb { r: 131, g: 165, b: 152 },
            secondary: ColorRgb { r: 211, g: 134, b: 155 },
            success: ColorRgb { r: 152, g: 195, b: 121 },
            warning: ColorRgb { r: 254, g: 128, b: 25 },
            error: ColorRgb { r: 204, g: 36, b: 29 },
            border: ColorRgb { r: 60, g: 58, b: 50 },
            selected_bg: ColorRgb { r: 102, g: 92, b: 84 },
            selected_fg: ColorRgb { r: 235, g: 219, b: 178 },
            text_muted: ColorRgb { r: 146, g: 131, b: 116 },
            border_focused: ColorRgb { r: 254, g: 128, b: 25 },
        },
    }
}
```

**文件**: `src/ui/themes/tokyo_night.rs`

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

/// Tokyo Night Theme - Modern dark theme
/// https://github.com/enkia/tokyo-night
pub fn tokyo_night_theme() -> Theme {
    Theme {
        name: "tokyo_night".to_string(),
        colors: ColorPalette {
            background: ColorRgb { r: 26, g: 27, b: 38 },
            foreground: ColorRgb { r: 192, g: 202, b: 245 },
            primary: ColorRgb { r: 122, g: 162, b: 247 },
            secondary: ColorRgb { r: 187, g: 154, b: 247 },
            success: ColorRgb { r: 158, g: 206, b: 106 },
            warning: ColorRgb { r: 224, g: 175, b: 104 },
            error: ColorRgb { r: 247, g: 118, b: 142 },
            border: ColorRgb { r: 41, g: 46, b: 66 },
            selected_bg: ColorRgb { r: 51, g: 59, b: 91 },
            selected_fg: ColorRgb { r: 192, g: 202, b: 245 },
            text_muted: ColorRgb { r: 86, g: 95, b: 137 },
            border_focused: ColorRgb { r: 122, g: 162, b: 247 },
        },
    }
}
```

**文件**: `src/ui/themes/catppuccin_mocha.rs`

```rust
use crate::ui::theme::{ColorRgb, Theme, ColorPalette};

/// Catppuccin Mocha Theme - Soothing dark theme
/// https://github.com/catppuccin/catppuccin
pub fn catppuccin_mocha_theme() -> Theme {
    Theme {
        name: "catppuccin_mocha".to_string(),
        colors: ColorPalette {
            background: ColorRgb { r: 30, g: 30, b: 46 },
            foreground: ColorRgb { r: 205, g: 214, b: 244 },
            primary: ColorRgb { r: 137, g: 180, b: 250 },
            secondary: ColorRgb { r: 203, g: 166, b: 247 },
            success: ColorRgb { r: 166, g: 227, b: 161 },
            warning: ColorRgb { r: 250, g: 179, b: 135 },
            error: ColorRgb { r: 243, g: 139, b: 168 },
            border: ColorRgb { r: 49, g: 50, b: 68 },
            selected_bg: ColorRgb { r: 88, g: 91, b: 112 },
            selected_fg: ColorRgb { r: 205, g: 214, b: 244 },
            text_muted: ColorRgb { r: 108, g: 112, b: 134 },
            border_focused: ColorRgb { r: 137, g: 180, b: 250 },
        },
    }
}
```

**验收标准**:
- [ ] 7 个主题文件全部创建
- [ ] 每个主题通过 `cargo test` 测试
- [ ] 颜色值符合官方主题规范
- [ ] 所有主题在 Dark/Light 环境下对比度可达 WCAG AA 标准

---

### Phase 2: 核心层重构（预计 3 小时）

#### Step 2.1: 重构 `src/ui/theme.rs`

**目标**: 移除硬编码主题，保留通用结构和样式方法

**修改后结构**:

```rust
//! UI theme configuration

use crate::constants;
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// RGB color representation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<ColorRgb> for Color {
    fn from(rgb: ColorRgb) -> Self {
        Color::Rgb(rgb.r, rgb.g, rgb.b)
    }
}

/// Color palette for a theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorPalette {
    pub primary: ColorRgb,
    pub secondary: ColorRgb,
    pub success: ColorRgb,
    pub warning: ColorRgb,
    pub error: ColorRgb,
    pub background: ColorRgb,
    pub foreground: ColorRgb,
    pub border: ColorRgb,
    pub selected_bg: ColorRgb,
    pub selected_fg: ColorRgb,
    pub text_muted: ColorRgb,
    pub border_focused: ColorRgb,
}

/// UI Theme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
}

impl Theme {
    /// Create theme from name
    pub fn new(name: &str) -> Self {
        crate::ui::themes::get_theme(name).unwrap_or_else(|| Self::dark())
    }

    /// Get dark theme (fallback)
    pub fn dark() -> Self {
        crate::ui::themes::dark_theme()
    }

    /// Get default theme
    pub fn default_theme() -> Self {
        Self::new(crate::ui::themes::default_theme_name())
    }

    // === Style Methods (保持不变) ===
    
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.colors.selected_fg.into())
            .bg(self.colors.selected_bg.into())
            .add_modifier(Modifier::BOLD)
    }

    pub fn focused_border_style(&self) -> Style {
        Style::default().fg(self.colors.border_focused.into())
    }

    pub fn normal_border_style(&self) -> Style {
        Style::default().fg(self.colors.border.into())
    }

    pub fn primary_text_style(&self) -> Style {
        Style::default().fg(self.colors.foreground.into())
    }

    pub fn secondary_text_style(&self) -> Style {
        Style::default().fg(self.colors.text_muted.into())
    }

    pub fn primary_style(&self) -> Style {
        Style::default().fg(self.colors.primary.into())
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.colors.success.into())
    }

    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.colors.warning.into())
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.colors.error.into())
    }

    /// Get next theme in rotation
    pub fn next(&self) -> Self {
        let themes = crate::ui::themes::THEME_NAMES;
        let current_idx = themes.iter().position(|&t| t == self.name).unwrap_or(0);
        let next_idx = (current_idx + 1) % themes.len();
        Self::new(themes[next_idx])
    }

    /// Get all available theme names
    pub fn available_themes() -> Vec<&'static str> {
        crate::ui::themes::THEME_NAMES.to_vec()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_theme()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_new() {
        let dark = Theme::new("dark");
        assert_eq!(dark.name, "dark");

        let nord = Theme::new("nord");
        assert_eq!(nord.name, "nord");

        let invalid = Theme::new("invalid");
        assert_eq!(invalid.name, "dark"); // Fallback
    }

    #[test]
    fn test_theme_next() {
        let dark = Theme::new("dark");
        let next = dark.next();
        assert_eq!(next.name, "light");

        // Test wrap around
        let last = Theme::new("catppuccin_mocha");
        let wrapped = last.next();
        assert_eq!(wrapped.name, "dark");
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert_eq!(themes.len(), 7);
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"nord"));
    }

    #[test]
    fn test_styles() {
        let theme = Theme::dark();
        let selected = theme.selected_style();
        assert!(selected.add_modifier == Modifier::BOLD);
    }
}
```

**验收标准**:
- [ ] 编译通过
- [ ] 所有现有测试通过
- [ ] 新增 `next()` 方法测试通过
- [ ] 向后兼容：`Theme::dark()` 仍可用

#### Step 2.2: 更新 `src/constants.rs`

**目标**: 清理颜色常量，保留配置常量

**修改**:

```rust
//! Application constants

// === 保留配置常量 ===
pub const APP_NAME: &str = "repotui";
pub const CONFIG_DIR_NAME: &str = "repotui";
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const CONFIG_VERSION: &str = "1.0";
// ... 其他配置常量

// === UI 配置 (移除颜色常量) ===
pub mod ui {
    /// Default theme
    pub const DEFAULT_THEME: &str = "dark";
    
    /// Show git status by default
    #[allow(dead_code)]
    pub const DEFAULT_SHOW_GIT_STATUS: bool = true;
    
    /// Show branch by default
    #[allow(dead_code)]
    pub const DEFAULT_SHOW_BRANCH: bool = true;
}

// === 保留安全配置 ===
pub mod security {
    pub const DEFAULT_ALLOW_SYMLINKS: bool = false;
    pub const DEFAULT_MAX_SEARCH_DEPTH: usize = 2;
}

// === 移除 ColorRgb 结构体 ===
// 已迁移到 theme.rs
```

**验收标准**:
- [ ] 编译通过
- [ ] 移除 `ui::dark` 和 `ui::light` 模块
- [ ] 保留 `DEFAULT_THEME` 常量

---

### Phase 3: 应用层集成（预计 3 小时）

#### Step 3.1: 更新消息类型

**文件**: `src/app/msg.rs`

**修改 `ThemeChanged` 消息**:

```rust
/// Theme changed
ThemeChanged,

/// Set specific theme
SetTheme(String),
```

**验收标准**:
- [ ] 消息类型编译通过
- [ ] 现有代码不受影响

#### Step 3.2: 更新 update 逻辑

**文件**: `src/app/update.rs`

**修改 `ThemeChanged` 处理**:

```rust
AppMsg::ThemeChanged => {
    // Cycle to next theme
    app.theme = app.theme.next();
    save_theme_config(app);
}

AppMsg::SetTheme(theme_name) => {
    // Set specific theme
    app.theme = Theme::new(&theme_name);
    save_theme_config(app);
}

// Helper function
fn save_theme_config(app: &mut App) {
    if let Some(ref mut config) = app.config {
        config.ui.theme = app.theme.name.clone();
        
        match config::save_config(config) {
            Ok(()) => {
                app.loading_message = Some(format!("Theme: {}", app.theme.name));
            }
            Err(e) => {
                app.error_message = Some(format!(
                    "Failed to save theme: {}. Theme will reset on restart.",
                    e
                ));
            }
        }
    }
}
```

**验收标准**:
- [ ] 主题切换逻辑正确
- [ ] 配置保存成功显示提示
- [ ] 配置保存失败有错误提示

#### Step 3.3: 更新键盘处理

**文件**: `src/handler/keyboard.rs`

**修改 `t` 键行为**（从切换改为打开主题菜单）:

```rust
// 在 Running 状态
KeyCode::Char('t') => {
    // Open theme selector instead of direct toggle
    let _ = app.msg_tx.try_send(AppMsg::OpenThemeSelector);
}
```

**新增消息处理**:

```rust
// 在 update.rs 中添加
AppMsg::OpenThemeSelector => {
    app.state = AppState::SelectingTheme;
}

AppMsg::CloseThemeSelector => {
    app.state = AppState::Running;
}

AppMsg::SelectTheme(theme_name) => {
    app.theme = Theme::new(&theme_name);
    save_theme_config(app);
    app.state = AppState::Running;
}
```

**验收标准**:
- [ ] `t` 键打开主题选择器
- [ ] 方向键可在主题间导航
- [ ] Enter 确认选择

---

### Phase 4: UI 组件开发（预计 4 小时）

#### Step 4.1: 创建主题选择器组件

**文件**: `src/ui/widgets/theme_selector.rs`

```rust
//! Theme selector widget

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

use crate::ui::theme::Theme;

/// Theme selector widget
pub struct ThemeSelector<'a> {
    themes: Vec<&'a str>,
    state: &'a mut ListState,
    theme_preview: Theme,
}

impl<'a> ThemeSelector<'a> {
    pub fn new(themes: Vec<&'a str>, state: &'a mut ListState, current_theme: &Theme) -> Self {
        // Initialize state if not already selected
        if state.selected().is_none() && !themes.is_empty() {
            let current_idx = themes.iter().position(|&t| t == current_theme.name).unwrap_or(0);
            state.select(Some(current_idx));
        }

        let theme_preview = current_theme.clone();
        Self {
            themes,
            state,
            theme_preview,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Clear area
        frame.render_widget(Clear, area);

        // Create centered popup
        let popup_area = centered_rect(60, 50, area);

        // Main block
        let block = Block::default()
            .title(" Select Theme ")
            .borders(Borders::ALL)
            .border_style(self.theme_preview.focused_border_style())
            .style(
                Style::default()
                    .bg(self.theme_preview.colors.background.into())
                    .fg(self.theme_preview.colors.foreground.into()),
            );

        // Split into preview and list
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(5), // Preview
                Constraint::Min(5),    // List
                Constraint::Length(3), // Help
            ])
            .split(popup_area.inner(block.inner_margin()));

        // Render block
        frame.render_widget(block, popup_area);

        // Render preview
        self.render_preview(frame, chunks[0]);

        // Render theme list
        self.render_list(frame, chunks[1]);

        // Render help
        self.render_help(frame, chunks[2]);
    }

    fn render_preview(&self, frame: &mut Frame, area: Rect) {
        let preview_text = vec![
            Line::from(Span::styled(
                "Theme Preview",
                Style::default()
                    .fg(self.theme_preview.colors.primary.into())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Success | Warning | Error",
                Style::default().fg(self.theme_preview.colors.success.into()),
            )),
        ];

        frame.render_widget(
            ratatui::widgets::Paragraph::new(preview_text)
                .style(Style::default().bg(self.theme_preview.colors.background.into())),
            area,
        );
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .themes
            .iter()
            .map(|&name| {
                let is_selected = Some(self.state.selected().unwrap_or(0))
                    == self
                        .themes
                        .iter()
                        .position(|&t| t == name)
                        .map(|i| i);

                let style = if is_selected {
                    self.theme_preview.selected_style()
                } else {
                    self.theme_preview.primary_text_style()
                };

                ListItem::new(Line::from(Span::styled(
                    format!("  {}", name.replace('_', " ").to_uppercase()),
                    style,
                )))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(" Available Themes "))
            .highlight_style(self.theme_preview.selected_style());

        frame.render_stateful_widget(list, area, self.state);
    }

    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = Line::from(vec![
            Span::styled("↑/↓ ", Style::default().fg(self.theme_preview.colors.primary.into())),
            Span::raw("Navigate  "),
            Span::styled("Enter", Style::default().fg(self.theme_preview.colors.primary.into())),
            Span::raw(" Select  "),
            Span::styled("Esc", Style::default().fg(self.theme_preview.colors.primary.into())),
            Span::raw(" Cancel"),
        ]);

        frame.render_widget(
            ratatui::widgets::Paragraph::new(help_text)
                .style(Style::default().fg(self.theme_preview.colors.text_muted.into())),
            area,
        );
    }

    /// Navigate to next theme
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.themes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Navigate to previous theme
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.themes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Get selected theme name
    pub fn selected(&self) -> Option<String> {
        self.state
            .selected()
            .and_then(|i| self.themes.get(i).map(|&s| s.to_string()))
    }
}

/// Helper: centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

**验收标准**:
- [ ] 组件渲染正确
- [ ] 主题预览区域显示
- [ ] 导航功能正常
- [ ] 选择功能正常

#### Step 4.2: 更新 AppState

**文件**: `src/app/state.rs`

**新增状态**:

```rust
/// Application state
#[derive(Debug, Clone)]
pub enum AppState {
    // ... existing variants
    /// Selecting theme
    SelectingTheme,
}
```

**验收标准**:
- [ ] 编译通过
- [ ] 状态优先级正确

#### Step 4.3: 集成到主循环

**文件**: `src/lib.rs` 或 `src/ui/render.rs`

**在渲染循环中添加**:

```rust
match &app.state {
    AppState::SelectingTheme => {
        // Render theme selector
        let mut theme_state = ListState::default();
        // Initialize with current selection
        let current_idx = Theme::available_themes()
            .iter()
            .position(|&t| t == app.theme.name)
            .unwrap_or(0);
        theme_state.select(Some(current_idx));

        let mut selector = ThemeSelector::new(
            Theme::available_themes(),
            &mut theme_state,
            &app.theme,
        );
        selector.render(f, area);
    }
    // ... other states
}
```

**验收标准**:
- [ ] 主题选择器正确渲染
- [ ] 主题即时预览生效

---

### Phase 5: 测试与优化（预计 2 小时）

#### Step 5.1: 单元测试

**文件**: `tests/unit/ui/theme_test.rs`

```rust
#[cfg(test)]
mod theme_tests {
    use repotui::ui::theme::Theme;
    use repotui::ui::themes;

    #[test]
    fn test_all_themes_load() {
        for &name in themes::THEME_NAMES {
            let theme = Theme::new(name);
            assert_eq!(theme.name, name);
            // Verify all colors are set
            assert!(theme.colors.background.r > 0 || theme.colors.background.g > 0 || theme.colors.background.b > 0);
        }
    }

    #[test]
    fn test_theme_rotation() {
        let mut current = Theme::dark();
        for &expected in themes::THEME_NAMES.iter().cycle().take(14) {
            assert_eq!(current.name, expected);
            current = current.next();
        }
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::new("nord");
        let serialized = toml::to_string(&theme).unwrap();
        let deserialized: Theme = toml::from_str(&serialized).unwrap();
        assert_eq!(theme.name, deserialized.name);
    }
}
```

#### Step 5.2: 集成测试

**文件**: `tests/integration/theme_switching.rs`

```rust
#[tokio::test]
async fn test_theme_switching_in_runtime() {
    // Create app
    // Simulate 't' key press
    // Verify theme changed
    // Verify config saved
}
```

#### Step 5.3: 性能测试

**测试项**:
- [ ] 主题切换延迟 < 16ms (60fps)
- [ ] 内存占用无显著增加
- [ ] 渲染帧率稳定在 60fps

---

## 测试策略

### 测试金字塔

```
          E2E (3 场景)
         /
     集成 (10 用例)
    /
  单元 (50+ 用例)
```

### 单元测试 (覆盖率目标：≥90%)

| 模块 | 测试内容 | 用例数 |
|------|----------|--------|
| `theme.rs` | Theme 创建/序列化/样式方法 | 15 |
| `themes/*.rs` | 7 个主题颜色定义 | 14 |
| `theme_selector.rs` | 组件渲染/导航/选择 | 12 |
| `update.rs` | 主题切换逻辑 | 9 |

### 集成测试

| 场景 | 测试内容 |
|------|----------|
| 配置加载 | 从 config.toml 加载主题名 |
| 主题切换 | 运行时切换并保存 |
| 配置保存 | 切换后 config.toml 更新 |
| 错误处理 | 无效主题名回退到 dark |

### E2E 测试

| 场景 | 步骤 | 预期结果 |
|------|------|----------|
| 主题循环切换 | 连续按`t`7 次 | 主题循环一圈回到 dark |
| 选择器导航 | 打开选择器→导航→选择 | 主题正确切换 |
| 重启保持 | 切换主题→重启应用 | 主题保持不变 |

### 手动测试清单

- [ ] 所有 7 个主题在终端中渲染正确
- [ ] 文字对比度在可接受范围
- [ ] 主题切换无闪烁
- [ ] 配置保存/加载正确
- [ ] 终端 resize 后主题保持

---

## 验收标准

### 功能验收

- [ ] **7 个内置主题**: dark, light, nord, dracula, gruvbox_dark, tokyo_night, catppuccin_mocha
- [ ] **终端独立**: 使用 RGB 真彩色，不受终端配色影响
- [ ] **运行时切换**: 按`t`键打开主题选择器，方向键导航，Enter 确认
- [ ] **即时预览**: 切换时立即看到新主题效果
- [ ] **配置保存**: 主题选择自动保存到 config.toml
- [ ] **重启保持**: 重启应用后主题保持不变

### 技术验收

- [ ] **编译通过**: `cargo build` 无错误无警告
- [ ] **测试通过**: `cargo test` 所有测试通过（≥50 个单元测试）
- [ ] **代码质量**: `cargo clippy -- -D warnings` 无警告
- [ ] **格式化**: `cargo fmt` 格式化通过
- [ ] **向后兼容**: 现有 dark/light 切换仍可用
- [ ] **无性能回退**: 主题切换延迟 < 16ms

### 用户体验验收

- [ ] **主题选择器美观**: 居中弹窗，带预览区域
- [ ] **帮助提示清晰**: 显示导航/选择/取消说明
- [ ] **切换反馈**: 切换后显示 "Theme: xxx" 提示
- [ ] **错误处理**: 无效主题名回退到 dark，不崩溃

---

## 风险评估

### 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 终端不支持真彩色 | 中 | 高 | 检测终端能力，降级到 256 色 |
| 主题颜色对比度不足 | 低 | 中 | 使用 WCAG 工具预检所有主题 |
| 配置文件不兼容 | 低 | 高 | 保持向后兼容，旧配置自动迁移 |

### 进度风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 主题定义工作量大 | 中 | 低 | 分批实施，优先 4 个主流主题 |
| UI 组件复杂度高 | 低 | 中 | 复用现有 widget 模式 |

### 应对措施

1. **真彩色检测**: 添加环境变量检测
```rust
fn supports_truecolor() -> bool {
    std::env::var("COLORTERM")
        .map(|v| v == "truecolor" || v == "24bit")
        .unwrap_or(false)
}
```

2. **对比度验证**: 使用 WCAG 公式预验证
```rust
fn validate_contrast(fg: ColorRgb, bg: ColorRgb) -> f32 {
    // WCAG contrast ratio calculation
}
```

---

## 附录

### 主题颜色参考

#### Nord 官方配色
- Polar Night: #2E3440, #3B4252, #434C5E, #4C566A
- Snow Storm: #D8DEE9, #E5E9F0, #ECEFF4
- Frost: #8FBCBB, #88C0D0, #81A1C1, #5E81AC
- Aurora: #BF616A, #D08770, #EBCB8B, #A3BE8C, #B48EAD

#### Dracula 官方配色
- Background: #282A36
- Foreground: #F8F8F2
- Current Line: #44475A
- Comment: #6272A4
- Cyan: #8BE9FD
- Green: #50FA7B
- Orange: #FFB86C
- Pink: #FF79C6
- Purple: #BD93F9
- Red: #FF5555
- Yellow: #F1FA8C

#### Gruvbox Dark 官方配色
- Background: #282828
- Foreground: #EBDBB2
- Red: #CC241D
- Green: #98971A
- Yellow: #D79921
- Blue: #458588
- Purple: #B16286
- Aqua: #689D6A
- Orange: #D65D0E

#### Tokyo Night 官方配色
- Background: #1A1B26
- Foreground: #C0CAF5
- Blue: #7AA2F7
- Magenta: #BB9AF7
- Cyan: #7DCFFF
- Green: #9ECE6A
- Yellow: #E0AF68
- Red: #F7768E

#### Catppuccin Mocha 官方配色
- Background: #1E1E2E
- Foreground: #CDD6F4
- Blue: #89B4FA
- Lavender: #B4BEFE
- Sapphire: #74C7EC
- Sky: #89DCEB
- Teal: #94E2D5
- Green: #A6E3A1
- Yellow: #F9E2AF
- Peach: #FAB387
- Red: #F38BA8
- Maroon: #EBA0AC
- Pink: #F5C2E7
- Mauve: #CBA6F7

---

**文档结束**

最后更新：2026-03-07  
维护者：repotui Team
