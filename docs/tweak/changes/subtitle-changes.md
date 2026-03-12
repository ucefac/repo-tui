# Subtitle Feature Changes

## 修改内容

### 1. 主目录管理界面 (src/ui/widgets/main_dir_manager.rs)

**修改位置**: `render_title()` 方法（第 72-89 行）

**修改内容**:
- 将标题区域分为两行（标题 + 副标题）
- 添加副标题："Manage root directories that store multiple repositories"
- 副标题使用 `text_muted` 颜色显示

### 2. 目录选择器 (src/ui/widgets/dir_chooser.rs)

**修改位置**:
- `DirectoryChooserState` 方法（第 41-71 行）
- `render_title()` 方法（第 168-194 行）
- 导入语句（第 11 行）

**修改内容**:
1. 添加 `subtitle()` 方法，根据 `return_to` 返回不同副标题：
   - `ReturnTarget::ManagingDirs`: "Select a folder to serve as root for multiple git repositories"
   - `ReturnTarget::Running`: "Select a git-managed folder to add to the repository list"
   - `AddSingleRepository` 模式：同 `Running` 状态

2. 修改 `render_title()` 方法：
   - 将标题区域分为两行（标题 + 副标题）
   - 副标题使用 `text_muted` 颜色显示

3. 添加 `ReturnTarget` 导入

### 3. 状态定义 (src/app/state.rs)

**修改位置**: `DirectoryChooserMode` 枚举（第 204-216 行）

**修改内容**:
- 为 `SelectMainDirectory` 变体添加 `return_to: ReturnTarget` 字段
- 更新 `Default` 实现

### 4. 更新逻辑 (src/app/update.rs)

**修改位置**: 所有创建 `DirectoryChooserMode::SelectMainDirectory` 的地方

**修改内容**:
- 添加 `return_to` 字段初始化

### 5. 键盘处理 (src/handler/keyboard.rs)

**修改位置**: 创建 `DirectoryChooserMode` 的测试代码

**修改内容**:
- 添加 `return_to` 字段初始化

## 副标题文案总结

| 位置 | 副标题文案 |
|------|-----------|
| 主目录管理界面 | Manage root directories that store multiple repositories |
| 目录选择器（从主目录打开） | Select a folder to serve as root for multiple git repositories |
| 目录选择器（从仓库列表打开） | Select a git-managed folder to add to the repository list |

## 测试

添加了 `test_state_subtitle()` 测试用例验证副标题逻辑。
