# Phase 1 MVP TUI 实现计划

**项目**: repotui  
**日期**: 2026-03-06  
**阶段**: Phase 1 MVP

## 目标
实现4个核心 TUI 功能，完成 MVP 可用的界面。

## 任务清单

### Task 1: 目录选择组件 (dir_chooser.rs)
- [x] 创建组件文件
- [x] 实现 DirChooser widget
- [x] 显示当前路径和子目录列表
- [x] 支持选中高亮
- [x] 实时统计 Git 仓库数量

### Task 2: 仓库列表组件 (repo_list.rs)
- [x] 创建组件文件
- [x] 实现 RepoList widget
- [x] 显示仓库名、分支、状态
- [x] 虚拟列表优化
- [x] 滚动位置同步

### Task 3: 搜索框组件 (search_box.rs)
- [x] 创建组件文件
- [x] 实现 SearchBox widget
- [x] 焦点状态指示
- [x] 搜索提示文字

### Task 4: 主题系统扩展 (theme.rs)
- [x] 添加更多颜色常量
- [x] 添加光标和焦点样式
- [x] 支持主题切换

### Task 5: 键盘导航完善 (keyboard.rs)
- [x] 添加目录选择器的键盘处理
- [x] 完善 Enter 键在目录选择中的逻辑
- [x] 添加目录进入/返回支持

### Task 6: Render 模块化重构 (render.rs)
- [x] 使用新组件重构
- [x] 移除内联渲染逻辑
- [x] 统一使用组件渲染

## 验收标准
- [x] 目录选择可正常浏览和选择
- [x] 仓库列表正确显示
- [x] 搜索实时过滤
- [x] 所有快捷键可用
- [x] cargo build 无错误
- [x] cargo clippy 无警告

## 依赖
已确认 Cargo.toml 包含:
- ratatui = "0.29"
- crossterm = "0.28"
- tokio = { version = "1", features = ["full"] }
