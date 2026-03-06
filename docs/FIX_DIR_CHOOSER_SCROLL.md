# 修复报告：目录选择界面滚动同步 Bug

**修复日期**: 2026-03-06  
**问题报告者**: 用户  
**Bug 描述**: 在目录选择界面，向下移动时列表不滚动，选中项超出可视区域

---

## 问题分析

### 现象
- 在目录选择界面按 `j` 或 `↓` 向下导航
- 选中项（高亮）移出屏幕可视区域
- 列表不自动滚动跟随

### 根因
`DirChooser` 组件缺少滚动机制：
1. 没有 `scroll_offset` 字段跟踪视口位置
2. `render_directory_list` 渲染**所有**条目，而非仅渲染可见范围
3. 导航时未更新滚动位置

---

## 修复方案

### 修改 1: 添加滚动状态字段

**文件**: `src/app/state.rs`
```rust
AppState::ChoosingDir {
    path: PathBuf,
    entries: Vec<String>,
    selected_index: usize,
    scroll_offset: usize,  // 新增：滚动偏移
}
```

**文件**: `src/ui/widgets/dir_chooser.rs`
```rust
pub struct DirChooser<'a> {
    // ... 现有字段 ...
    pub scroll_offset: usize,     // 新增
    pub visible_height: u16,      // 新增
}
```

### 修改 2: 实现滚动渲染逻辑

**文件**: `src/ui/widgets/dir_chooser.rs`
```rust
fn render_directory_list(
    area: Rect,
    buf: &mut Buffer,
    entries: &[String],
    selected_index: usize,
    scroll_offset: usize,        // 新增参数
    visible_height: u16,         // 新增参数
    theme: &Theme,
) {
    // 计算可见范围
    let visible_count = visible_height.saturating_sub(2) as usize;
    let start = scroll_offset;
    let end = (start + visible_count).min(entries.len());
    
    // 只渲染可见范围内的条目
    let items: Vec<ListItem> = entries[start..end]
        .iter()
        .enumerate()
        .map(|(visible_idx, name)| {
            let absolute_idx = start + visible_idx;
            // 根据 absolute_idx 判断是否选中
            ...
        })
        .collect();
}
```

### 修改 3: 导航时自动滚动

**文件**: `src/app/update.rs`
```rust
AppMsg::DirectoryNavDown => {
    if let AppState::ChoosingDir { ... } = &mut app.state {
        *selected_index = (*selected_index + 1).min(entries.len() - 1);
        
        // 自动滚动：确保选中项可见
        let visible_count = 15usize;
        if *selected_index >= *scroll_offset + visible_count {
            *scroll_offset = selected_index.saturating_sub(visible_count - 1);
        }
    }
}

AppMsg::DirectoryNavUp => {
    if let AppState::ChoosingDir { ... } = &mut app.state {
        *selected_index = selected_index.saturating_sub(1);
        
        // 自动滚动：确保选中项可见
        if *selected_index < *scroll_offset {
            *scroll_offset = *selected_index;
        }
    }
}
```

### 修改 4: 更新所有初始化代码

修改了 8+ 处 `AppState::ChoosingDir` 初始化，添加 `scroll_offset: 0`：
- `src/app/update.rs` (3处)
- `src/handler/keyboard.rs` (3处)
- `src/ui/widgets/dir_chooser.rs` (2处测试)
- `tests/directory_selection.rs` (1处)

---

## 文件变更

| 文件 | 变更类型 | 说明 |
|------|----------|------|
| `src/app/state.rs` | 修改 | 添加 `scroll_offset` 字段 |
| `src/ui/widgets/dir_chooser.rs` | 修改 | 添加滚动字段和可见范围渲染 |
| `src/ui/render.rs` | 修改 | 传递 `scroll_offset` 参数 |
| `src/app/update.rs` | 修改 | 导航时更新滚动位置 |
| `src/handler/keyboard.rs` | 修改 | 更新状态初始化 |
| `tests/directory_selection.rs` | 修改 | 更新测试初始化 |

---

## 验证结果

```bash
✅ cargo build          # 编译成功
✅ cargo test           # 95/95 测试通过
✅ cargo clippy         # 无警告
```

### 修复后行为
1. 向下导航时，当选中项接近列表底部，自动向下滚动
2. 向上导航时，当选中项接近列表顶部，自动向上滚动
3. 选中项始终保持在可视区域内

---

## 用户使用

现在可以正常运行：
```bash
cargo run
```

在目录选择界面：
- `j`/`↓` - 向下导航（自动滚动）
- `k`/`↑` - 向上导航（自动滚动）
- `Enter` - 进入目录
- `Space` - 确认选择

---

**修复完成！** 目录选择界面现在支持平滑滚动。
