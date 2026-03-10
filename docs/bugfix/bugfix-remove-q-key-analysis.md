# Bug 修复分析：移除 q 键退出功能

## 问题描述

当前程序在子界面中仍支持使用 `q` 键关闭界面，这与主界面仅支持 `Ctrl+c` 退出的设计不一致，造成用户体验不统一。

## 根因分析

代码中存在以下位置将 `q` 键与 `Esc` 键并列处理：

1. **Action Menu** (`src/handler/keyboard.rs:58`)
   - `KeyCode::Char('q') | KeyCode::Esc` 用于关闭 Action Menu

2. **Theme Selector** (`src/handler/keyboard.rs:172`)
   - `KeyCode::Esc | KeyCode::Char('q')` 用于关闭 Theme Selector

3. **Main Directory Manager** (`src/handler/keyboard.rs:533`)
   - `KeyCode::Esc | KeyCode::Char('q')` 用于关闭 Main Directory Manager

4. **文档过时** (`docs/design/keyboard-shortcuts.md`)
   - 第 54 行仍记载 `q` 键可退出程序
   - 多处子界面帮助中仍提到 `q` 键

## 影响范围

- 用户需要适应使用 `Esc` 键关闭子界面
- 不再支持 `q` 键关闭子界面
- 文档需要同步更新

## 修复策略

移除所有子界面中的 `KeyCode::Char('q')` 处理，仅保留 `KeyCode::Esc` 用于关闭子界面。
