# 随机主题功能实施计划

## 📋 需求描述

在主题选择器列表中添加一个 "🎲 Random (随机)" 选项，用户选择后随机应用一个主题。

### 用户确认的需求

1. **位置**: 第一个选项（列表顶部）
2. **预览行为**: 选中 random 时，预览区域显示一个随机主题的實際效果
3. **配置保存**: 保存为 `theme = "random"`，启动时随机选择一个主题
4. **显示文本**: 使用 "🎲 Random (随机)"

---

## 🎯 功能规格

### 用户体验流程

```
1. 用户按 't' 打开主题选择器
2. 默认选中 "🎲 Random (随机)" 选项
3. 预览区域显示一个随机主题的彩色预览
4. 用户可以：
   - 按 Enter 确认 → 应用当前预览的随机主题，配置保存为 "random"
   - 按 j/k 导航到其他主题 → 预览对应主题
   - 按 q 取消 → 保持原主题不变
5. 下次启动时：
   - 如果配置是 "random" → 再次随机选择一个主题
   - 如果配置是具体主题名 → 应用该主题
```

### UI/UX 设计

**主题选择器界面**:

```
╭────────── 🎨 Theme Selector ──────────╮
│                                        │
│ ╭─────── Preview: dark ───────╮       │
│ │ Theme: dark                 │       │
│ │ [Primary] [Success] ...     │       │
│ │ Selected: RGB(...)          │       │
│ ╰─────────────────────────────╯       │
│                                        │
│ ▶ 🎲 Random (随机)                     │
│   dark                                 │
│   light                                │
│   nord                                 │
│   dracula                              │
│   gruvbox_dark                         │
│   tokyo_night                          │
│   catppuccin_mocha                     │
│                                        │
│ [j/k] Navigate  [Enter] Select  [q] Cancel │
╰────────────────────────────────────────┘
```

**预览区域说明**:
- 位置：主题选择器中部（约 7 行高）
- 内容：
  - 主题名称
  - 颜色样本（Primary/Success/Warning/Error 的背景色展示）
  - RGB 值
  - 当前主题对比
- Random 行为：选中 "🎲 Random" 时，预览区域显示一个**具体随机主题的预览效果**

---

## 🏗️ 技术方案

### 核心设计决策

#### 1. Random 作为"元选项"

- "🎲 Random (随机)" 不是一个真实主题，而是一个"元选项"
- 选择 random 后，实际存储的是一个**具体的主题实例**
- 配置文件中保存 `"random"` 字符串作为标记

#### 2. 随机性来源

- 使用 `rand` crate (版本 0.8)
- 每次选择 random 时，从 7 个真实主题中随机选择一个
- 每次启动时如果配置是 "random"，重新随机

#### 3. 预览逻辑

- 选中 random 选项时，生成一个随机主题用于预览
- **关键**: 预览的主题和最终应用的主题是**同一个**（避免"所见非所得"）
- 实现方式：在 `AppState::SelectingTheme` 中存储 `preview_theme`，选中 random 时使用该预览主题

#### 4. 配置持久化

```toml
# 配置示例
[ui]
theme = "random"  # 保存为 "random" 字符串
```

启动时：
```rust
if config.ui.theme == "random" {
    app.theme = get_random_theme();  // 随机选择一个
} else {
    app.theme = Theme::new(&config.ui.theme);
}
```

---

## 📝 实施步骤

### 步骤 1：添加依赖

**文件**: `Cargo.toml`

```toml
[dependencies]
# ... 现有依赖
rand = "0.8"
```

**理由**: `rand` crate 提供高质量的随机数生成，Rust 生态标准选择。

---

### 步骤 2：更新主题定义

**文件**: `src/ui/themes/mod.rs`

#### 修改 2.1: 在 `THEME_NAMES` 开头添加 "random"

```rust
/// Available theme names
pub const THEME_NAMES: &[&str] = &[
    "🎲 Random (随机)",  // 新增：第一个位置
    "dark",
    "light",
    "nord",
    "dracula",
    "gruvbox_dark",
    "tokyo_night",
    "catppuccin_mocha",
];
```

**影响**:
- 主题选择器列表第一个选项是 random
- 索引 0 对应 random，索引 1-7 对应真实主题

#### 修改 2.2: 添加随机主题函数

在 `get_theme()` 函数后添加：

```rust
use rand::seq::SliceRandom;

/// Get a random theme (excluding the "random" option itself)
pub fn get_random_theme() -> Theme {
    let mut rng = rand::thread_rng();
    // Exclude "random" option (first element)
    let real_themes = &THEME_NAMES[1..];
    
    real_themes
        .choose(&mut rng)
        .and_then(|&name| get_theme(name))
        .unwrap_or_else(dark_theme)
}
```

**测试覆盖**:
```rust
#[test]
fn test_get_random_theme() {
    let theme = get_random_theme();
    assert!(theme.is_some());
    assert_ne!(theme.unwrap().name, "🎲 Random (随机)");
    assert_ne!(theme.unwrap().name, "random");
}

#[test]
fn test_get_random_theme_variety() {
    // 多次调用应该可能得到不同主题
    let themes: Vec<_> = (0..20)
        .map(|_| get_random_theme().name)
        .collect();
    // 20 次随机应该至少覆盖 2 个不同主题
    assert!(themes.iter().collect::<std::collections::HashSet<_>>().len() >= 2);
}
```

---

### 步骤 3：更新 Theme 创建逻辑

**文件**: `src/ui/theme.rs`

#### 修改 3.1: `Theme::new()` 处理 random

```rust
impl Theme {
    /// Create theme from name
    pub fn new(name: &str) -> Self {
        // Handle "🎲 Random (随机)" option
        if name.contains("Random") || name == "random" {
            return crate::ui::themes::get_random_theme();
        }
        
        crate::ui::themes::get_theme(name).unwrap_or_else(Self::dark)
    }
    
    // ... 其他方法
}
```

**理由**: 统一处理 random 选项，所有调用 `Theme::new()` 的地方都能正确处理。

---

### 步骤 4：更新 AppState

**文件**: `src/app/state.rs`

#### 修改 4.1: `AppState::SelectingTheme` 添加 preview_theme 字段

```rust
pub enum AppState {
    // ... 其他状态
    
    SelectingTheme {
        theme_list_state: ratatui::widgets::ListState,
        preview_theme: Theme,  // 新增：存储预览主题（保证所见即所得）
    },
    
    // ...
}
```

#### 修改 4.2: 添加 `preview_theme()` 访问器方法

```rust
impl AppState {
    // ... 现有方法
    
    /// Get preview theme reference (for theme selector rendering)
    pub fn preview_theme(&self) -> Option<&Theme> {
        if let AppState::SelectingTheme { preview_theme, .. } = self {
            Some(preview_theme)
        } else {
            None
        }
    }
}
```

---

### 步骤 5：更新主题选择器打开逻辑

**文件**: `src/app/update.rs`

#### 修改 5.1: `AppMsg::OpenThemeSelector` 初始化预览主题

```rust
AppMsg::OpenThemeSelector => {
    let mut theme_list_state = ratatui::widgets::ListState::default();
    theme_list_state.select(Some(0));  // 默认选中 random
    
    // 生成初始预览主题（随机主题）
    let preview_theme = crate::ui::themes::get_random_theme();
    
    app.state = AppState::SelectingTheme { 
        theme_list_state,
        preview_theme,
    };
}
```

---

### 步骤 6：更新导航逻辑

**文件**: `src/app/update.rs`

#### 修改 6.1: `AppMsg::ThemeNavDown` 更新预览主题

```rust
AppMsg::ThemeNavDown => {
    if let AppState::SelectingTheme { theme_list_state, preview_theme } = &mut app.state {
        let themes = crate::ui::themes::THEME_NAMES;
        if themes.is_empty() {
            return;
        }
        let current = theme_list_state.selected().unwrap_or(0);
        let len = themes.len();
        let next = (current + 1) % len;
        theme_list_state.select(Some(next));
        
        // 如果选中 random，更新预览主题为新的随机主题
        if next == 0 {
            *preview_theme = crate::ui::themes::get_random_theme();
        }
    }
}
```

#### 修改 6.2: `AppMsg::ThemeNavUp` 更新预览主题

```rust
AppMsg::ThemeNavUp => {
    if let AppState::SelectingTheme { theme_list_state, preview_theme } = &mut app.state {
        let themes = crate::ui::themes::THEME_NAMES;
        if themes.is_empty() {
            return;
        }
        let current = theme_list_state.selected().unwrap_or(0);
        let len = themes.len();
        let prev = if current == 0 { len - 1 } else { current - 1 };
        theme_list_state.select(Some(prev));
        
        // 如果选中 random，更新预览主题为新的随机主题
        if prev == 0 {
            *preview_theme = crate::ui::themes::get_random_theme();
        }
    }
}
```

---

### 步骤 7：更新主题选择逻辑

**文件**: `src/app/update.rs`

#### 修改 7.1: `AppMsg::SelectTheme` 使用预览主题

```rust
AppMsg::SelectTheme(theme_name) => {
    // Handle random theme - use the preview theme (guarantees "what you see is what you get")
    let final_theme = if theme_name.contains("Random") {
        if let AppState::SelectingTheme { preview_theme, .. } = &app.state {
            preview_theme.clone()
        } else {
            // Fallback (shouldn't happen)
            crate::ui::themes::get_random_theme()
        }
    } else {
        Theme::new(&theme_name)
    };
    
    app.theme = final_theme;
    
    if let Some(ref mut config) = app.config {
        // Save "random" in config if random was selected
        config.ui.theme = if theme_name.contains("Random") {
            "random".to_string()
        } else {
            theme_name
        };
        
        // 保存配置
        match config::save_config(config) {
            Ok(()) => {
                app.loading_message = Some(format!("Theme '{}' saved", config.ui.theme));
            }
            Err(e) => {
                app.error_message = Some(format!("Failed to save theme: {}", e));
            }
        }
    }
    
    app.state = AppState::Running;
}
```

**关键点**: 
- 使用 `AppState` 中存储的 `preview_theme`，确保"所见即所得"
- 配置文件中保存 "random" 字符串

---

### 步骤 8：更新主题选择器渲染

**文件**: `src/ui/render.rs`

#### 修改 8.1: `render_theme_selector()` 使用 AppState 中的预览主题

```rust
fn render_theme_selector(
    frame: &mut Frame,
    area: Rect,
    app: &mut App,  // 修改：传入 &mut App 而不是单独的 theme_list_state
    theme: &Theme,
) {
    use crate::ui::themes::THEME_NAMES;

    let popup_area = centered_rect(60, 55, area);
    frame.render_widget(Clear, popup_area);

    let current_theme = theme.clone();

    // Get preview theme from app state
    let preview_theme = if let Some(preview) = app.state.preview_theme() {
        preview.clone()
    } else {
        // Fallback (shouldn't happen)
        Theme::dark()
    };

    // Get theme list state
    if let Some(theme_list_state) = app.state.theme_list_state_mut() {
        let selected_index = theme_list_state.selected().unwrap_or(0);
        let selector = ThemeSelector::new(THEME_NAMES, selected_index, &current_theme, preview_theme)
            .title("🎨 Select Theme");

        frame.render_widget(selector, popup_area);
    }
}
```

**注意**: 需要修改函数签名，从传入 `theme_list_state` 改为传入 `&mut App`。

---

### 步骤 9：更新键盘处理

**文件**: `src/handler/keyboard.rs`

#### 修改 9.1: `get_selected_theme_name()` 处理 random

```rust
fn get_selected_theme_name(app: &App) -> Option<String> {
    use crate::ui::themes::THEME_NAMES;
    
    if let AppState::SelectingTheme { theme_list_state, .. } = &app.state {
        if let Some(selected) = theme_list_state.selected() {
            if selected < THEME_NAMES.len() {
                let name = THEME_NAMES[selected];
                // Return "random" for the random option
                if name.contains("Random") {
                    return Some("random".to_string());
                }
                return Some(name.to_string());
            }
        }
    }
    None
}
```

---

### 步骤 10：更新配置加载

**文件**: `src/app/update.rs`

#### 修改 10.1: `AppMsg::ConfigLoaded` 处理 random 配置

找到配置加载后的主题应用位置（约 92-118 行），在 `Ok(config)` 分支添加：

```rust
AppMsg::ConfigLoaded(result) => {
    match result {
        Ok(config) => {
            app.main_dir = Some(config.main_directory.clone());
            app.config = Some(config.clone());

            // 处理 random 配置 - 启动时随机选择一个主题
            app.theme = if config.ui.theme == "random" {
                crate::ui::themes::get_random_theme()
            } else {
                Theme::new(&config.ui.theme)
            };

            // Start loading repositories
            runtime.dispatch(crate::app::msg::Cmd::LoadRepositories(
                config.main_directory,
            ));
        }
        // ... 错误处理
    }
    app.loading = false;
    app.loading_message = None;
}
```

**理由**: 启动时如果配置是 "random"，需要随机选择一个具体主题应用。

---

## 🧪 测试计划

### 单元测试

#### `src/ui/themes/mod.rs` 测试

```rust
#[test]
fn test_get_random_theme() {
    let theme = get_random_theme();
    assert!(theme.is_some());
    assert_ne!(theme.unwrap().name, "🎲 Random (随机)");
    assert_ne!(theme.unwrap().name, "random");
}

#[test]
fn test_get_random_theme_variety() {
    let themes: Vec<_> = (0..20)
        .map(|_| get_random_theme().name)
        .collect();
    // 20 次随机应该至少覆盖 2 个不同主题
    assert!(themes.iter().collect::<std::collections::HashSet<_>>().len() >= 2);
}
```

#### `src/ui/theme.rs` 测试

```rust
#[test]
fn test_theme_new_random() {
    let theme = Theme::new("random");
    assert_ne!(theme.name, "random");
    assert_ne!(theme.name, "🎲 Random (随机)");
    
    let theme2 = Theme::new("🎲 Random (随机)");
    assert_ne!(theme2.name, "random");
}
```

---

### 集成测试

**文件**: `tests/theme_random.rs` (新建)

```rust
//! Random theme functionality tests

use repotui::app::model::App;
use repotui::app::msg::AppMsg;
use repotui::app::state::AppState;
use repotui::app::update;
use repotui::runtime::executor::Runtime;
use tokio::sync::mpsc;

#[tokio::test(flavor = "multi_thread")]
async fn test_select_random_theme() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    // Open theme selector
    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);
    
    // Should be in SelectingTheme state
    assert!(matches!(app.state, AppState::SelectingTheme { .. }));
    
    // Should be at index 0 (random)
    if let AppState::SelectingTheme { theme_list_state, .. } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(0));
    }

    // Select theme (should apply random theme)
    update::update(AppMsg::SelectTheme("random".to_string()), &mut app, &runtime);
    
    // Should be back to Running state
    assert!(matches!(app.state, AppState::Running));
    
    // Should have a real theme (not "random")
    assert_ne!(app.theme.name, "random");
    assert_ne!(app.theme.name, "🎲 Random (随机)");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_random_theme_navigation() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    // Open theme selector
    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);
    
    // Navigate down to second theme
    update::update(AppMsg::ThemeNavDown, &mut app, &runtime);
    
    if let AppState::SelectingTheme { theme_list_state, .. } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(1)); // Should be at "dark"
    }
    
    // Navigate back to random
    update::update(AppMsg::ThemeNavUp, &mut app, &runtime);
    
    if let AppState::SelectingTheme { theme_list_state, preview_theme, .. } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(0)); // Back to random
        // Preview theme should be a real theme
        assert_ne!(preview_theme.name, "🎲 Random (随机)");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_config_saves_random() {
    let (tx, _rx) = mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);
    
    // Setup config
    app.config = Some(repotui::config::Config::default());

    // Open theme selector
    update::update(AppMsg::OpenThemeSelector, &mut app, &runtime);
    
    // Select random theme
    update::update(AppMsg::SelectTheme("🎲 Random (随机)".to_string()), &mut app, &runtime);
    
    // Config should have "random" saved
    assert_eq!(app.config.as_ref().unwrap().ui.theme, "random");
}
```

---

### E2E 测试

**文件**: `tests/e2e/theme_random_e2e.rs` (可选，新建)

```rust
//! E2E test for random theme functionality

#[tokio::test(flavor = "multi_thread")]
async fn test_random_theme_full_flow() {
    // 1. Start application
    // 2. Open theme selector ('t')
    // 3. Verify random is selected by default
    // 4. Verify preview shows a real theme
    // 5. Press Enter to select
    // 6. Verify a theme was applied
    // 7. Verify config was saved with "random"
    // 8. Restart application
    // 9. Verify a (potentially different) theme was loaded
}
```

---

## 📁 文件修改清单

| 文件 | 修改内容 | 预估行数 | 优先级 |
|------|----------|----------|--------|
| `Cargo.toml` | 添加 `rand = "0.8"` 依赖 | +1 | P0 |
| `src/ui/themes/mod.rs` | 添加 random 到 `THEME_NAMES`，实现 `get_random_theme()` | ~+20 | P0 |
| `src/ui/theme.rs` | `Theme::new()` 处理 random | ~+6 | P0 |
| `src/app/state.rs` | `AppState::SelectingTheme` 添加 `preview_theme` 字段，添加访问器方法 | ~+10 | P0 |
| `src/app/update.rs` | `OpenThemeSelector`, `ThemeNav*`, `SelectTheme`, `ConfigLoaded` 处理 | ~+40 | P0 |
| `src/ui/render.rs` | `render_theme_selector()` 使用 AppState | ~+10 | P1 |
| `src/handler/keyboard.rs` | `get_selected_theme_name()` 处理 random | ~+6 | P1 |
| `tests/theme_random.rs` | 新建集成测试文件 | ~+80 | P2 |

**总计**: 8 个文件，约 +173 行代码

---

## ⚠️ 风险与缓解

| 风险 | 影响 | 缓解措施 | 状态 |
|------|------|----------|------|
| `rand` crate 版本冲突 | 中 | 使用 0.8 版本，兼容 Rust 1.56+ | ✅ 已验证 |
| Emoji 显示问题（某些终端） | 低 | 降级显示文本，不影响功能 | ✅ 可接受 |
| 配置兼容性问题 | 低 | 向后兼容，旧配置不受影响 | ✅ 已考虑 |
| 预览闪烁（每次渲染生成新随机） | 中 | 在 AppState 中存储 preview_theme | ✅ 已解决 |
| "所见非所得"问题 | 中 | 使用 preview_theme 保证一致性 | ✅ 已解决 |
| Random 被当作普通主题 | 中 | 多处检查 "Random" 字符串 | ✅ 已处理 |

---

## 🎯 验收标准

### 功能验收

- ✅ 主题选择器第一个选项是 "🎲 Random (随机)"
- ✅ 选中 random 时，预览区域显示一个具体主题的彩色预览
- ✅ 按 Enter 确认 random 后，应用预览的主题
- ✅ 配置文件中保存为 `theme = "random"`
- ✅ 启动时如果配置是 "random"，随机选择一个主题
- ✅ 按 j/k 导航时，random 选项的预览主题会更新
- ✅ 按 q 取消后，原主题不变

### 代码质量

- ✅ 所有现有测试通过
- ✅ 新增单元测试覆盖 random 功能
- ✅ 新增集成测试覆盖完整流程
- ✅ Clippy 无警告
- ✅ 代码符合项目风格

### 用户体验

- ✅ 默认选中 random 选项（降低选择困难）
- ✅ 预览区域实时反映当前选择
- ✅ 配置保存符合直觉（"random" 字符串）
- ✅ 启动行为一致（每次都随机）

---

## 📅 实施时间估算

| 阶段 | 任务 | 时间 |
|------|------|------|
| Phase 1 | 添加依赖 + 主题定义 | 10 分钟 |
| Phase 2 | 更新 Theme + AppState | 10 分钟 |
| Phase 3 | 更新 update 逻辑 | 15 分钟 |
| Phase 4 | 更新 render + keyboard | 10 分钟 |
| Phase 5 | 更新配置加载 | 5 分钟 |
| Phase 6 | 编写测试 | 15 分钟 |
| Phase 7 | 测试验证 + 修复 | 15 分钟 |
| **总计** | | **80 分钟** |

---

## 🔗 相关文档

- [PRD v2](./ghclone-prd-v2.md) - 产品需求文档
- [DEVELOPMENT_GUIDE.md](./DEVELOPMENT_GUIDE.md) - 开发指南
- [PHASE0_COMPLETE.md](./PHASE0_COMPLETE.md) - Phase 0 完成报告

---

## 📝 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-03-07 | 初始版本 - 完整实施计划 |

---

**文档状态**: ✅ 完成  
**下一步**: 等待用户确认后开始实施
