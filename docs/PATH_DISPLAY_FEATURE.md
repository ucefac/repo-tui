# 主目录路径显示与切换功能

**实现日期**: 2026-03-07  
**状态**: ✅ 已完成

---

## 功能概述

本次更新为 repotui 添加了主目录路径显示和快速切换功能：

1. **路径显示**: 在界面底部状态栏下方显示当前配置的主目录路径
2. **快捷键切换**: 按 `m` 键快速打开目录选择器，切换主目录
3. **点击复制**: 鼠标点击路径区域可复制路径到剪贴板

---

## 界面布局

```
┌──────────────────────────────────────────────────────┐
│ 🔍 Search: [query]                                   │
├──────────────────────────────────────────────────────┤
│  Repository List                                     │
│  - repo1 (main)                                      │
│  - repo2                                             │
│  - repo3 *                                           │
├──────────────────────────────────────────────────────┤
│ [j/k] Navigate  [g/G] Jump  [/] Search  [q] Quit    │
│ 📂 /Users/yyyyyyh/Desktop/ghclone (12 repos)         │  ← 新增
└──────────────────────────────────────────────────────┘
```

---

## 使用说明

### 1. 查看主目录路径

启动 repotui 后，主目录路径会自动显示在界面底部的状态栏下方。

**显示内容**:
- 📂 目录图标
- 完整的主目录路径
- 发现的 Git 仓库数量

**示例**:
```
📂 /Users/yyyyyyh/Desktop/ghclone (12 repos)
```

### 2. 切换主目录

**步骤**:
1. 在主界面按 `m` 键
2. 使用 `↑/↓` 导航到目标目录
3. 按 `→` 进入目录，`Space` 确认选择
4. 系统会自动扫描新目录下的 Git 仓库并更新配置

**快捷键**:
- `m`: 打开目录选择器
- `j/↓`: 向下导航
- `k/↑`: 向上导航
- `Enter`: 进入子目录
- `←`: 返回上级目录
- `Space`: 确认选择当前目录
- `q/Esc`: 取消

### 3. 复制路径到剪贴板

**鼠标操作**:
- 在支持鼠标的终端中，直接点击路径显示区域
- 路径会自动复制到系统剪贴板
- 显示成功提示："✅ Path copied to clipboard"

**使用场景**:
- 快速分享项目路径
- 在终端中粘贴路径执行命令
- 在其他工具中引用项目路径

---

## 技术细节

### 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/ui/widgets/path_bar.rs` | 新建 | PathBar 组件实现 |
| `src/ui/widgets/mod.rs` | 修改 | 导出 PathBar |
| `src/ui/render.rs` | 修改 | 渲染 PathBar，记录点击区域 |
| `src/handler/keyboard.rs` | 修改 | 添加 'm' 键处理 |
| `src/handler/mouse.rs` | 新建 | 鼠标事件处理 |
| `src/handler/mod.rs` | 修改 | 导出鼠标处理器 |
| `src/app/model.rs` | 修改 | 添加 path_bar_area 字段 |
| `src/app/msg.rs` | 修改 | 添加 CopyPathToClipboard 消息 |
| `src/app/update.rs` | 修改 | 实现复制逻辑 |
| `src/ui/widgets/help_panel.rs` | 修改 | 添加 'm' 键说明 |
| `src/lib.rs` | 修改 | 处理鼠标事件 |
| `Cargo.toml` | 修改 | 添加 arboard 依赖 |

### 依赖添加

```toml
[dependencies]
arboard = "3.3"  # 跨平台剪贴板支持
```

### 关键代码

**PathBar 组件**:
```rust
pub struct PathBar<'a> {
    pub path: &'a Path,
    pub repo_count: Option<usize>,
    pub theme: &'a Theme,
    pub truncate: bool,
    pub max_length: usize,
}
```

**路径截断**:
```rust
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }
    
    let available = max_len - 4;
    let start_len = available / 2;
    let end_len = available - start_len;
    
    format!("{}...{}", &path[..start_len], &path[path.len() - end_len..])
}
```

---

## 测试覆盖

### 单元测试 (7 个)

1. `test_path_bar_widget_creation` - PathBar 组件创建
2. `test_path_bar_truncation` - 路径截断功能
3. `test_path_bar_display_with_main_dir` - 主目录显示
4. `test_m_key_opens_directory_chooser` - 'm' 键打开目录选择器
5. `test_directory_chooser_can_be_opened_with_m_key` - 目录选择器集成
6. `test_copy_path_to_clipboard_message` - 复制路径消息处理
7. `test_help_panel_includes_m_key` - 帮助面板说明

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行 PathBar 相关测试
cargo test --test path_display

# 运行 PathBar 组件测试
cargo test path_bar
```

**测试结果**:
```
test result: ok. 119 passed; 0 failed
```

---

## 兼容性

### 终端要求

- **最小尺寸**: 80x24 (无变化)
- **推荐尺寸**: 100x30+
- **鼠标支持**: 可选（用于点击复制）

### 平台支持

- ✅ macOS (配置文件：`~/.config/repotui/config.toml`)
- ✅ Linux (配置文件：`~/.config/repotui/config.toml`)
- ✅ Windows (WSL 和原生)

### 向后兼容

- ✅ 不影响现有快捷键
- ✅ 不改变配置文件格式
- ✅ 不影响现有组件 API
- ✅ 可优雅降级（无鼠标时可用键盘切换）

---

## 已知限制

1. **剪贴板依赖**: 某些 Wayland 环境可能需要额外配置
2. **鼠标支持**: 部分终端模拟器不支持鼠标事件
3. **路径截断**: 超长路径会显示为 `/start/.../end` 格式

---

## 未来优化

### Phase 1 (已实现) ✅

- [x] 显示主目录路径
- [x] 'm' 键切换目录
- [x] 点击复制路径

### Phase 2 (计划)

- [ ] 面包屑导航（点击层级快速跳转）
- [ ] 最近目录历史（快速切换）
- [ ] 相对路径显示（在主目录内）
- [ ] 自定义路径格式

### Phase 3 (可选)

- [ ] 多目录支持（工作区模式）
- [ ] 目录收藏（快速访问）
- [ ] 路径搜索（模糊匹配）

---

## 故障排除

### 问题：按 'm' 键无反应

**原因**: 消息处理延迟或终端不支持

**解决**:
1. 检查终端是否支持 Crossterm
2. 确认未在其他功能中绑定 'm' 键
3. 尝试重启应用

### 问题：点击路径无法复制

**原因**: 终端不支持鼠标或剪贴板权限

**解决**:
1. 检查终端是否启用鼠标支持
2. macOS: 检查系统权限设置
3. Linux: 检查 Wayland/X11 配置
4. 使用替代方案：手动复制显示的路径文本

### 问题：路径显示不全

**原因**: 终端宽度不足

**解决**:
1. 放大终端窗口
2. 路径会自动截断为首尾格式
3. 点击复制可获得完整路径

---

## 相关文档

- [设计文档](../plans/2026-03-07-main-directory-path-display-design.md)
- [开发指南](../DEVELOPMENT_GUIDE.md)
- [PRD v2](../ghclone-prd-v2.md)

---

**最后更新**: 2026-03-07  
**维护者**: repotui Team
