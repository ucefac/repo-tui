# 循环导航功能实施计划

## 需求描述

实现仓库列表、目录选择器、主题选择器的循环上下移动功能：
- 最后一项向下移动 → 跳转到第一项
- 第一项向上移动 → 跳转到最后一项

## 影响范围

### 需要更新的列表

1. **仓库列表** (主界面)
   - 消息：`AppMsg::NextRepo` / `AppMsg::PreviousRepo`
   - 位置：`src/app/update.rs:55-69`

2. **目录选择器** (目录浏览界面)
   - 消息：`AppMsg::DirectoryNavDown` / `AppMsg::DirectoryNavUp`
   - 位置：`src/app/update.rs:266-299`

3. **主题选择器** (主题选择界面)
   - 消息：`AppMsg::ThemeNavDown` / `AppMsg::ThemeNavUp`
   - 位置：`src/app/update.rs:434-456`

## 实施步骤

### Phase 1: 更新 PRD 文档 ✅

**文件**: `docs/ghclone-prd-v2.md`

**更新位置**:

1. **章节 2.2 - F4: 列表导航**
   - 添加循环导航说明
   - 更新按键映射表
   - 添加实现逻辑代码示例

2. **章节 2.4 - 全局快捷键**
   - 更新按键映射表注释

3. **章节 4.1 - 主界面布局**
   - 更新底部快捷键提示

4. **章节 4.4 - 帮助面板**
   - 添加循环导航说明

### Phase 2: 代码实现

**文件**: `src/app/update.rs`

#### 2.1 仓库列表循环导航 (行 55-69)

```rust
// 当前实现 ❌
AppMsg::NextRepo => {
    if app.filtered_indices.is_empty() {
        return;
    }
    let current = app.selected_index().unwrap_or(0);
    let next = (current + 1).min(app.filtered_indices.len() - 1); // 不循环
    app.set_selected_index(Some(next));
}

AppMsg::PreviousRepo => {
    if app.filtered_indices.is_empty() {
        return;
    }
    let current = app.selected_index().unwrap_or(0);
    let prev = current.saturating_sub(1); // 不循环
    app.set_selected_index(Some(prev));
}
```

```rust
// 目标实现 ✅
AppMsg::NextRepo => {
    if app.filtered_indices.is_empty() {
        return;
    }
    let current = app.selected_index().unwrap_or(0);
    let len = app.filtered_indices.len();
    // 循环：最后一项 → 第一项
    let next = (current + 1) % len;
    app.set_selected_index(Some(next));
}

AppMsg::PreviousRepo => {
    if app.filtered_indices.is_empty() {
        return;
    }
    let current = app.selected_index().unwrap_or(0);
    let len = app.filtered_indices.len();
    // 循环：第一项 → 最后一项
    let prev = if current == 0 { len - 1 } else { current - 1 };
    app.set_selected_index(Some(prev));
}
```

#### 2.2 目录选择器循环导航 (行 266-299)

```rust
// 当前实现 ❌
AppMsg::DirectoryNavDown => {
    if let AppState::ChoosingDir { entries, selected_index, .. } = &mut app.state {
        if !entries.is_empty() {
            *selected_index = (*selected_index + 1).min(entries.len() - 1);
            // ... scroll logic
        }
    }
}

AppMsg::DirectoryNavUp => {
    if let AppState::ChoosingDir { entries, selected_index, .. } = &mut app.state {
        *selected_index = selected_index.saturating_sub(1);
        // ... scroll logic
    }
}
```

```rust
// 目标实现 ✅
AppMsg::DirectoryNavDown => {
    if let AppState::ChoosingDir { entries, selected_index, scroll_offset, .. } = &mut app.state {
        if !entries.is_empty() {
            let len = entries.len();
            // 循环：最后一项 → 第一项
            *selected_index = (*selected_index + 1) % len;
            // Auto-scroll logic (保持不变)
            let visible_count = 15usize;
            if *selected_index >= *scroll_offset + visible_count {
                *scroll_offset = selected_index.saturating_sub(visible_count - 1);
            }
        }
    }
}

AppMsg::DirectoryNavUp => {
    if let AppState::ChoosingDir { entries, selected_index, scroll_offset, .. } = &mut app.state {
        if !entries.is_empty() {
            let len = entries.len();
            // 循环：第一项 → 最后一项
            *selected_index = if *selected_index == 0 { len - 1 } else { *selected_index - 1 };
            // Auto-scroll logic (保持不变)
            if *selected_index < *scroll_offset {
                *scroll_offset = *selected_index;
            }
        }
    }
}
```

#### 2.3 主题选择器循环导航 (行 434-456)

```rust
// 当前实现 ❌
AppMsg::ThemeNavUp => {
    if let AppState::SelectingTheme { theme_list_state } = &mut app.state {
        let themes = crate::ui::themes::THEME_NAMES;
        if themes.is_empty() {
            return;
        }
        let current = theme_list_state.selected().unwrap_or(0);
        let next = current.saturating_sub(1); // 不循环
        theme_list_state.select(Some(next));
    }
}

AppMsg::ThemeNavDown => {
    if let AppState::SelectingTheme { theme_list_state } = &mut app.state {
        let themes = crate::ui::themes::THEME_NAMES;
        if themes.is_empty() {
            return;
        }
        let current = theme_list_state.selected().unwrap_or(0);
        let next = (current + 1).min(themes.len() - 1); // 不循环
        theme_list_state.select(Some(next));
    }
}
```

```rust
// 目标实现 ✅
AppMsg::ThemeNavUp => {
    if let AppState::SelectingTheme { theme_list_state } = &mut app.state {
        let themes = crate::ui::themes::THEME_NAMES;
        if themes.is_empty() {
            return;
        }
        let current = theme_list_state.selected().unwrap_or(0);
        let len = themes.len();
        // 循环：第一项 → 最后一项
        let prev = if current == 0 { len - 1 } else { current - 1 };
        theme_list_state.select(Some(prev));
    }
}

AppMsg::ThemeNavDown => {
    if let AppState::SelectingTheme { theme_list_state } = &mut app.state {
        let themes = crate::ui::themes::THEME_NAMES;
        if themes.is_empty() {
            return;
        }
        let current = theme_list_state.selected().unwrap_or(0);
        let len = themes.len();
        // 循环：最后一项 → 第一项
        let next = (current + 1) % len;
        theme_list_state.select(Some(next));
    }
}
```

### Phase 3: 更新测试

**文件**: `src/app/update.rs` (测试模块)

#### 3.1 添加仓库列表循环测试

```rust
#[test]
fn test_update_next_repo_cyclic() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.filtered_indices = vec![0, 1, 2];
    app.set_selected_index(Some(2)); // 最后一项

    // 向下应该循环到第一项
    update(AppMsg::NextRepo, &mut app, &runtime);
    assert_eq!(app.selected_index(), Some(0)); // 循环到第一项
}

#[test]
fn test_update_previous_repo_cyclic() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.filtered_indices = vec![0, 1, 2];
    app.set_selected_index(Some(0)); // 第一项

    // 向上应该循环到最后一项
    update(AppMsg::PreviousRepo, &mut app, &runtime);
    assert_eq!(app.selected_index(), Some(2)); // 循环到最后一项
}
```

#### 3.2 添加目录选择器循环测试

```rust
#[test]
fn test_directory_nav_down_cyclic() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::ChoosingDir {
        path: std::path::PathBuf::from("/tmp"),
        entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
        selected_index: 2, // 最后一项
        scroll_offset: 0,
    };

    update(AppMsg::DirectoryNavDown, &mut app, &runtime);

    if let AppState::ChoosingDir { selected_index, .. } = app.state {
        assert_eq!(selected_index, 0); // 循环到第一项
    } else {
        panic!("State should be ChoosingDir");
    }
}

#[test]
fn test_directory_nav_up_cyclic() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::ChoosingDir {
        path: std::path::PathBuf::from("/tmp"),
        entries: vec!["dir1".to_string(), "dir2".to_string(), "dir3".to_string()],
        selected_index: 0, // 第一项
        scroll_offset: 0,
    };

    update(AppMsg::DirectoryNavUp, &mut app, &runtime);

    if let AppState::ChoosingDir { selected_index, .. } = app.state {
        assert_eq!(selected_index, 2); // 循环到最后一项
    } else {
        panic!("State should be ChoosingDir");
    }
}
```

#### 3.3 添加主题选择器循环测试

```rust
#[test]
fn test_update_theme_nav_cyclic() {
    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut app = App::new(tx.clone());
    let runtime = Runtime::new(tx);

    app.state = AppState::SelectingTheme {
        theme_list_state: ratatui::widgets::ListState::default(),
    };

    // 假设当前有 5 个主题，从最后一个向上
    // 先设置到最后
    if let AppState::SelectingTheme { theme_list_state } = &mut app.state {
        theme_list_state.select(Some(4)); // 最后一个
    }

    update(AppMsg::ThemeNavDown, &mut app, &runtime);
    
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(0)); // 循环到第一个
    }

    // 从第一个向上
    update(AppMsg::ThemeNavUp, &mut app, &runtime);
    
    if let AppState::SelectingTheme { theme_list_state } = &app.state {
        assert_eq!(theme_list_state.selected(), Some(4)); // 循环到最后一个
    }
}
```

### Phase 4: 验证与测试

#### 4.1 编译检查

```bash
cargo check
```

#### 4.2 运行测试

```bash
cargo test
```

确保所有测试通过，包括新增的循环导航测试。

#### 4.3 手动验证

运行程序并测试：
```bash
cargo run
```

验证场景：
1. 仓库列表：最后一项按 `j` → 跳转到第一项
2. 仓库列表：第一项按 `k` → 跳转到最后一项
3. 目录选择器：同样验证循环
4. 主题选择器：同样验证循环

## 验收标准

- ✅ PRD 文档更新完成
- ✅ 3 个列表的循环导航实现完成
- ✅ 新增 6 个单元测试（每个列表 2 个）
- ✅ 所有测试通过（`cargo test`）
- ✅ 代码编译无警告（`cargo clippy`）
- ✅ 手动验证循环导航功能正常

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 空列表处理 | 中 | 保持现有的空列表检查，避免除零错误 |
| 滚动逻辑冲突 | 低 | 保留现有滚动逻辑，只修改选中索引计算 |
| 用户不习惯 | 低 | 在帮助面板和 PRD 中明确说明循环行为 |

## 时间估算

- Phase 1 (PRD 更新): 10 分钟
- Phase 2 (代码实现): 20 分钟
- Phase 3 (测试添加): 20 分钟
- Phase 4 (验证测试): 10 分钟

**总计**: 约 60 分钟
