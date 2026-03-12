# Subtitle Feature Analysis

## 修改目标

为以下界面添加副标题说明：

1. **主目录管理界面** (Main Directory Manager)
2. **目录选择器** (Directory Selector) - 根据打开来源显示不同副标题
   - 从主目录管理界面打开
   - 从仓库列表界面打开

## 影响范围

### 需要修改的文件

| 文件 | 修改内容 |
|------|---------|
| `src/ui/widgets/main_dir_manager.rs` | 为主界面添加副标题渲染 |
| `src/ui/widgets/dir_chooser.rs` | 为目录选择器添加副标题渲染，支持根据 mode 显示不同内容 |
| `src/app/state.rs` | 为 `DirectoryChooserMode::SelectMainDirectory` 添加副标题字段 |

### 不影响的功能

- 键盘事件处理
- 状态更新逻辑
- 数据模型

## 修改范围确认

✅ 确认为小调整：
- 仅添加 UI 副标题显示
- 不改变现有交互逻辑
- 不修改核心功能

## 副标题文案

| 位置 | 副标题文案 (英文) | 副标题文案 (中文) |
|------|------------------|------------------|
| 主目录管理界面 | "Manage root directories that store multiple repositories" | 管理用于存储多个仓库的主目录 |
| 目录选择器（从主目录打开） | "Select a folder to serve as root for multiple git repositories" | 选择一个文件夹作为主目录，用于存放多个 git 仓库 |
| 目录选择器（从仓库列表打开） | "Select a git-managed folder to add to the repository list" | 选择一个 git 仓库添加到列表 |
