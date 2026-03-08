# 主目录路径显示与切换设计

**日期**: 2026-03-07  
**作者**: repotui Team  
**状态**: 待评审

---

## 1. 需求概述

### 1.1 问题描述

当前 repotui 在项目列表界面不显示当前配置的主目录路径，用户无法直观看到正在扫描哪个目录。同时，用户想要切换主目录时需要手动编辑配置文件，缺乏交互式界面。

### 1.2 目标

1. 在主界面显示当前配置的主目录路径
2. 提供快捷键快速打开目录选择器，切换主目录

---

## 2. 设计方案

### 2.1 UI 布局变更

#### 当前布局

```
┌─────────────────────────────────────┐
│ 🔍 Search: [query]                  │  (3 行)
├─────────────────────────────────────┤
│  Repository List                    │
│  - repo1                            │  (Min 5 行)
│  - repo2                            │
├─────────────────────────────────────┤
│ [j/k] Navigate [g/G] Jump ...       │  (3 行)
└─────────────────────────────────────┘
```

#### 新布局

```
┌─────────────────────────────────────┐
│ 🔍 Search: [query]                  │  (3 行)
├─────────────────────────────────────┤
│  Repository List                    │
│  - repo1                            │  (Min 5 行)
│  - repo2                            │
├─────────────────────────────────────┤
│ [j/k] Navigate [g/G] Jump ...       │  (3 行)
│ 📂 /Users/username/main/dir         │  ← 新增（与工具栏同一区域）
└─────────────────────────────────────┘
```

**视觉设计**: 状态栏分为两行，第一行显示快捷键提示，第二行显示主目录路径，使用相同的边框和背景色，视觉上是一个整体区域。

### 2.2 新增组件：PathBar

**文件**: `src/ui/widgets/path_bar.rs`

**功能**:
- 显示当前主目录路径
- 路径过长时自动截断（居中显示省略号）
- **鼠标点击复制路径到剪贴板**
- 显示目录图标和统计信息（可选）

**样式**:
- 背景色：深色（Color::DarkGray）
- 前景色：次要文本色（theme.text_secondary）
- 边框：与状态栏一致（视觉上与状态栏是一个整体）

**交互**:
- **鼠标左键点击**: 复制路径到剪贴板
- 显示复制成功提示（可选）

**示例文本**:
```
 📂 /Users/yyyyyyh/Desktop/ghclone (12 repos)
```

### 2.3 快捷键设计

#### 新增快捷键：`m`

**作用**: 打开目录选择器，选择新的主目录

**助记**: "m" = "main directory"

**冲突检查**:
- ✅ `j/k` - 导航
- ✅ `g/G` - 跳转
- ✅ `Ctrl+d` - 向下滚动
- ✅ `d` - 未使用（小写）
- ✅ `m` - **未使用** ✓

**实现位置**: `src/handler/keyboard.rs:handle_running_keys()`

### 2.4 消息类型

#### 新增 AppMsg

无需新增消息类型，复用现有：

```rust
// 复用现有消息
AppMsg::ShowDirectoryChooser  // 打开目录选择器
AppMsg::DirectorySelected(String)  // 目录被选中
```

### 2.5 状态流转

```
Running --[按下 'm']--> ChoosingDir
ChoosingDir --[选择目录]--> Loading --> Running
ChoosingDir --[按下 'q'/Esc]--> Running
Running --[鼠标点击路径]--> CopyToClipboard
```

---

## 3. 实现细节

### 3.1 依赖添加

**文件**: `Cargo.toml`

```toml
[dependencies]
arboard = "3.3"  # 跨平台剪贴板支持
```

### 3.2 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/ui/widgets/path_bar.rs` | **新建** | PathBar 组件 |
| `src/ui/widgets/mod.rs` | 修改 | 导出 PathBar |
| `src/ui/render.rs` | 修改 | 在 render_main_ui 中渲染 PathBar |
| `src/handler/keyboard.rs` | 修改 | 添加 'm' 键处理 |
| `src/ui/theme.rs` | 可能修改 | 如有需要添加新颜色 |

### 3.3 PathBar 组件 API

```rust
pub struct PathBar<'a> {
    /// 主目录路径
    pub path: &'a Path,
    /// 仓库数量（可选）
    pub repo_count: Option<usize>,
    /// 主题
    pub theme: &'a Theme,
}

impl<'a> PathBar<'a> {
    pub fn new(path: &'a Path, repo_count: Option<usize>, theme: &'a Theme) -> Self;
    pub fn render(self, area: Rect, buf: &mut Buffer);
}
```

### 3.4 鼠标事件处理

**文件**: `src/handler/mouse.rs` (新建)

```rust
pub fn handle_mouse_event(
    event: MouseEvent,
    app: &App,
    layout: &LayoutAreas,
) -> Option<AppMsg> {
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // 检查是否点击在 PathBar 区域
            if layout.path_bar.contains(event.column, event.row) {
                if let Some(ref path) = app.main_dir {
                    return Some(AppMsg::CopyPathToClipboard(path.clone()));
                }
            }
        }
        _ => {}
    }
    None
}
```

**新增 AppMsg**:

```rust
// src/app/msg.rs
pub enum AppMsg {
    // ... 现有消息 ...
    
    /// 复制路径到剪贴板
    CopyPathToClipboard(PathBuf),
}
```

**Update 处理**:

```rust
// src/app/update.rs
AppMsg::CopyPathToClipboard(path) => {
    match arboard::Clipboard::new().and_then(|mut c| c.set_text(path.to_string_lossy())) {
        Ok(()) => {
            // 可选：显示成功提示
            app.loading_message = Some("Path copied to clipboard".to_string());
        }
        Err(e) => {
            app.error_message = Some(format!("Failed to copy: {}", e));
        }
    }
}
```

### 3.5 路径截断逻辑

当路径超过显示宽度时：

```rust
// 示例：保留开头和结尾
// /Users/yyyyyyh/Desktop/ghclone/repo1/repo2/.../repoN
fn truncate_path(path: &str, max_width: usize) -> String {
    if path.len() <= max_width {
        return path.to_string();
    }
    
    let start_len = max_width / 2 - 2;
    let end_len = max_width / 2 - 2;
    
    format!("{}...{}", &path[..start_len], &path[path.len() - end_len..])
}
```

### 3.6 渲染逻辑修改

**文件**: `src/ui/render.rs:render_main_ui()`

```rust
fn render_main_ui(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    // 修改布局：状态栏区域增加高度
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(5),    // Repository list
            Constraint::Length(4), // Status bar (增加 1 行给 PathBar)
        ])
        .split(area);

    // 渲染 SearchBox
    frame.render_widget(search_box, chunks[0]);

    // 渲染 RepoList
    frame.render_widget(repo_list, chunks[1]);

    // 渲染状态栏区域（包含快捷键提示和路径）
    render_status_bar_with_path(frame, app, chunks[2], theme);
}

fn render_status_bar_with_path(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    // 分割为上下两部分
    let status_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 快捷键提示
            Constraint::Length(1), // 路径显示
        ])
        .split(area);

    // 渲染快捷键提示（带边框）
    let status_text = if app.loading {
        format!(" ⏳ {}", app.loading_message.as_deref().unwrap_or("Loading..."))
    } else if let Some(ref error) = app.error_message {
        format!(" ⚠️ {}", error)
    } else {
        " [↑/↓] Navigate  [Home/End] Jump  [/] Search  [Enter] Open  [r] Refresh  [?] Help  [Ctrl+C] Quit ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_normal));

    let paragraph = Paragraph::new(status_text)
        .block(block)
        .style(Style::default().fg(theme.text_secondary).bg(Color::DarkGray));

    frame.render_widget(paragraph, status_chunks[0]);

    // 渲染路径（只有边框的左右两侧，底部边框与上面共享）
    if let Some(ref main_dir) = app.main_dir {
        let path_text = format!(" 📂 {}", main_dir.display());
        let path_paragraph = Paragraph::new(path_text)
            .style(Style::default().fg(theme.text_secondary).bg(Color::DarkGray));
        
        // 记录 PathBar 的点击区域供鼠标事件使用
        // TODO: 需要保存这个区域用于鼠标点击检测
        
        frame.render_widget(path_paragraph, status_chunks[1]);
    }
}
```

**注意**: 需要保存 PathBar 的点击区域，用于鼠标事件处理。

### 3.5 键盘处理修改

**文件**: `src/handler/keyboard.rs:handle_running_keys()`

```rust
fn handle_running_keys(key: KeyEvent, app: &mut App, _runtime: &Runtime) {
    match key.code {
        // ... 现有代码 ...

        // 新增：打开目录选择器
        KeyCode::Char('m') => {
            let _ = app.msg_tx.try_send(AppMsg::ShowDirectoryChooser);
        }

        // ... 其他代码 ...
    }
}
```

---

## 4. 测试策略

### 4.1 单元测试

**测试文件**: `src/ui/widgets/path_bar.rs`

| 测试用例 | 说明 |
|----------|------|
| `test_path_bar_empty` | 空路径渲染 |
| `test_path_bar_short_path` | 短路径完整显示 |
| `test_path_bar_long_path` | 长路径截断 |
| `test_path_bar_with_repo_count` | 显示仓库数量 |
| `test_path_bar_without_repo_count` | 不显示仓库数量 |

**测试文件**: `tests/path_display.rs`（集成测试）

| 测试用例 | 说明 |
|----------|------|
| `test_main_path_displayed` | 主路径在界面显示 |
| `test_m_key_opens_chooser` | 按 'm' 打开目录选择器 |
| `test_change_main_directory` | 切换主目录流程 |

### 4.2 手动测试

1. 启动应用，检查主路径显示
2. 按 'm' 键，验证目录选择器打开
3. 选择新目录，验证路径更新
4. 按 'q' 取消，验证返回主界面

---

## 5. 兼容性考虑

### 5.1 最小终端尺寸

当前最小尺寸：80x24

新增 PathBar 后：
- 原来：3 + 5 + 3 = 11 行
- 现在：3 + 3 + 5 + 3 = 14 行

**建议**: 保持最小高度 24 行不变（14 行内容有足够余量）

### 5.2 向后兼容

- ✅ 不影响现有快捷键
- ✅ 不改变配置格式
- ✅ 不影响现有组件 API
- ✅ 可优雅降级（无主路径时不显示 PathBar）

---

## 6. 验收标准

### 6.1 功能验收

- [ ] 主目录路径显示在状态栏下方
- [ ] 路径与状态栏在视觉上是一个整体区域
- [ ] 按 'm' 键打开目录选择器
- [ ] **点击路径复制路径到剪贴板**
- [ ] 选择新目录后，路径显示更新
- [ ] 取消选择后，返回主界面

### 6.2 质量验收

- [ ] 所有现有测试通过
- [ ] 新增测试覆盖核心功能
- [ ] 代码通过 `cargo clippy`
- [ ] 代码通过 `cargo fmt`
- [ ] 无编译警告

---

## 7. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 路径过长显示不全 | 用户体验 | 实现智能截断，保留首尾 |
| 终端太小 | 显示问题 | 保持现有最小尺寸限制 |
| 快捷键冲突 | 功能冲突 | 已检查无冲突 |
| 性能影响 | 渲染速度 | PathBar 简单渲染，影响可忽略 |

---

## 8. 后续优化（可选）

1. **点击复制**: 在支持鼠标的终端，点击路径可复制
2. **面包屑导航**: 显示完整路径层级，支持快速跳转
3. **最近目录**: 记录最近使用的 5 个目录，快速切换
4. **相对路径**: 在主目录内显示相对路径，更简洁

---

## 9. 总结

本设计通过在搜索框下方新增 PathBar 组件显示主目录路径，并通过 'm' 快捷键快速打开目录选择器，解决了用户无法直观查看和切换主目录的问题。设计简洁、实现成本低、向后兼容。

**预计工作量**: 2-3 小时  
**风险等级**: 低  
**推荐优先级**: 高
