---
type: fix
scope: fullstack
roles: rust-dev
worktree: fix-2026-03-14-list-scroll-widget-managed
---

# 列表滚动 Widget 自管理修复计划

## 概述

将列表滚动逻辑从 `render.rs` 移到各个 widget 内部，确保滚动时机准确。

**问题**: 目前滚动逻辑在 `render.rs` 中估算 `visible_count`，与 widget 内部实际计算不一致，导致：
- 仓库列表: 提前滚动（倒数第4项就开始滚动）
- 主题列表: 延迟滚动（选中项离开显示区后还要按一次才滚动）
- 目录列表: 延迟滚动（选中项离开显示区后按多次才滚动）

**目标**: Widget 自己管理滚动，使用相同的 `visible_count` 计算逻辑

## 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Widget 借用冲突 | 中 | 高 | 使用 `&mut` 传参，render 后同步状态 |
| 布局计算公式不准确 | 低 | 中 | 测试驱动，根据实际效果调整 |

## 任务清单

| 序号 | 任务 | 角色 | 依赖 | 状态 |
|------|------|------|------|------|
| 1 | RepoList Widget 自管理滚动 | rust-dev | - | pending |
| 2 | DirectoryChooser Widget 自管理滚动 | rust-dev | - | pending |
| 3 | ThemeSelector Widget 自管理滚动 | rust-dev | - | pending |
| 4 | MainDirManager Widget 自管理滚动 | rust-dev | - | pending |
| 5 | 清理 render.rs 中的滚动逻辑 | rust-dev | 1-4 | pending |
| 6 | 测试验证 | tester | 1-5 | pending |

---

## 详细实现

### 任务 1: RepoList Widget 自管理滚动

**文件**: `src/ui/widgets/repo_list.rs`

**当前问题**:
- `scroll_offset` 通过参数传入（只读）
- `render.rs` 在 widget 外部更新滚动

**修改方案**:
```rust
// 1. 添加 scroll_offset 可变引用参数
impl<'a> RepoList<'a> {
    pub fn new(
        repositories: &'a [Repository],
        filtered_indices: &'a [usize],
        theme: &'a Theme,
        scroll_offset: &'a mut usize,  // 新增
    ) -> Self { ... }
}

// 2. 在 Widget::render 开始时调用 self.update_scroll()
impl<'a> Widget for RepoList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 更新滚动偏移
        self.update_scroll();
        
        // ... 原有渲染逻辑
    }
}
```

**验收标准**:
- `render.rs` 中移除 `app.update_scroll_offset(chunks[2].height)`
- Widget 内部自行管理滚动
- 向下导航到最后一行才触发滚动

---

### 任务 2: DirectoryChooser Widget 自管理滚动

**文件**: `src/ui/widgets/dir_chooser.rs`

**当前结构**:
```rust
pub struct DirectoryChooser<'a> {
    state: &'a DirectoryChooserState,  // 不可变引用
    theme: &'a Theme,
    visible_height: u16,
}

pub struct DirectoryChooserState {
    pub scroll_offset: usize,
    // ...
}
```

**修改方案**:
```rust
// 1. 改为可变引用
pub struct DirectoryChooser<'a> {
    state: &'a mut DirectoryChooserState,  // ✅
    theme: &'a Theme,
    visible_height: u16,
}

// 2. render 时更新滚动
impl<'a> Widget for DirectoryChooser<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.state.update_scroll(self.visible_height);
        // ...
    }
}
```

**验收标准**:
- `render.rs` 中移除_directory_chooser 的滚动更新逻辑
- 向下导航到最后一行才触发滚动

---

### 任务 3: ThemeSelector Widget 自管理滚动

**文件**: `src/ui/widgets/theme_selector.rs`

**当前结构**:
```rust
pub struct ThemeSelector<'a> {
    scroll_offset: usize,  // 已有，但只读
    // ...
}

fn render_theme_selector(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    // 在这里更新 scroll_offset - ❌ 应该在 widget 内部
    let list_area_height = popup_area.height.saturating_sub(12);
    // ...
}
```

**修改方案**:
```rust
// 1. 添加 visible_height 参数
impl<'a> ThemeSelector<'a> {
    pub fn visible_height(mut self, h: u16) -> Self {
        self.visible_height = h;
        self
    }
}

// 2. render 时根据内部布局计算实际可见高度并更新滚动
impl<'a> Widget for ThemeSelector<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 根据布局约束计算实际列表区域高度
        // Title: 2, Preview: 7, Help: 1, Border: 2
        let list_area_height = area.height.saturating_sub(12);
        self.update_scroll(list_area_height);
        // ...
    }
}
```

**验收标准**:
- `render.rs` 中移除 `SelectingTheme` 的滚动更新逻辑
- 滚动时机准确

---

### 任务 4: MainDirManager Widget 自管理滚动

**文件**: `src/ui/widgets/main_dir_manager.rs`

**当前结构**:
```rust
pub struct MainDirManager<'a> {
    scroll_offset: usize,  // 已有，但只读
    // ...
}

fn render_main_dir_manager(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    // 在这里更新 scroll_offset - ❌ 应该在 widget 内部
    let list_area_height = area.height.saturating_sub(8);
    // ...
}
```

**修改方案**:
```rust
impl<'a> MainDirManager<'a> {
    // 添加方法计算并更新滚动
    fn update_scroll(&self, area: Rect) {
        // 根据布局计算实际列表区域高度
        let list_area_height = area.height.saturating_sub(8); // Title 3 + Help 3 + Border 2
        // ... 更新逻辑
    }
}

impl<'a> Widget for MainDirManager<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        self.update_scroll(area);
        // ...
    }
}
```

**验收标准**:
- `render.rs` 中移除 `ManagingDirs` 的滚动更新逻辑
- 向下导航到最后一行才触发滚动

---

### 任务 5: 清理 render.rs

**文件**: `src/ui/render.rs`

**清理内容**:
1. 删除 `AppState::ChoosingDir` 分支中的滚动更新代码
2. 删除 `AppState::SelectingTheme` 分支中的滚动更新代码
3. 删除 `AppState::ManagingDirs` 分支中的滚动更新代码
4. 删除 `app.update_scroll_offset(chunks[2].height)` 调用

**验收标准**:
- `render.rs` 中所有列表状态的滚动逻辑已被移除
- 相关代码从 `app/model.rs` 的 `update_scroll_offset` 方法删除
- 编译通过

---

### 任务 6: 测试验证

**测试项**:
1. 仓库列表（Running 状态）
   - 10 项列表，可见 5 项
   - 向下导航到第 5 项，再按一次才滚动
   - 向上导航到第 1 项，再按一次才滚动
   
2. 主题列表（SelectingTheme 状态）
   - 多个主题，按 ↓ 导航到最后一行才滚动
   - 循环导航正确

3. 目录列表（ChoosingDir 状态）
   - 多个目录，按 ↓ 导航到最后一行才滚动
   - 进入子目录测试滚动

4. 主目录列表（ManagingDirs 状态）
   - 多个目录，按 ↓ 导航到最后一行才滚动
   - 浏览模式切换测试

---

## 关键设计决策

### Widget 如何更新 app.scroll_offset？

**采用方案**: Widget 接收 `&mut usize` 引用，在 render 时直接更新

```rust
// 示例：RepoList
impl<'a> RepoList<'a> {
    pub fn new(
        repos: &'a [Repository],
        filtered: &'a [usize],
        theme: &'a Theme,
        scroll_offset: &'a mut usize,  // ✅ 可变引用
    ) -> Self {
        Self { scroll_offset, ... }
    }
}

impl<'a> Widget for RepoList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.update_scroll();
    }
}

impl<'a> RepoList<'a> {
    fn update_scroll(&self) {
        // 直接修改 *self.scroll_offset
        if selected >= *self.scroll_offset + visible_count {
            *self.scroll_offset = selected - visible_count + 1;
        }
    }
}
```

**优点**: 简单直接，无需同步步骤

---

## 实现顺序建议

**Phase 1: RepoList** (最简单)
- 验证方案可行性
- 建立基本模式

**Phase 2: DirectoryChooser**
- 已有 scroll_offset 字段
- 只需改为可变引用

**Phase 3: ThemeSelector**
- 需要调整布局计算（加上 visible_height builder）

**Phase 4: MainDirManager**
- 需要调整布局计算

**Phase 5: 清理 render.rs**
- 移除所有滚动更新逻辑

**Phase 6: 测试验证**
- 确保所有列表行为一致

---

## 关键文件清单

| 文件 | 修改类型 | 说明 |
|------|---------|------|
| `src/ui/widgets/repo_list.rs` | 修改 | Widget 自管理滚动 |
| `src/ui/widgets/dir_chooser.rs` | 修改 | Widget 自管理滚动 |
| `src/ui/widgets/theme_selector.rs` | 修改 | Widget 自管理滚动 |
| `src/ui/widgets/main_dir_manager.rs` | 修改 | Widget 自管理滚动 |
| `src/ui/render.rs` | 删除 | 移除滚动更新逻辑 |
| `src/app/model.rs` | 删除 | 移除 update_scroll_offset 方法 |

---

**最后更新**: 2026-03-14  
**版本**: v1.0  
**状态**: 待执行
