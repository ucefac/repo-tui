# Phase 4 实施计划：可选增强功能

**日期**: 2026-03-07  
**阶段**: Phase 4 - 可选增强  
**状态**: 📋 计划中

---

## 📊 功能概览

根据 PRD v2 Section 10，Phase 4 包含 4 个可选增强功能：

| 功能 | 用户价值 | 实现难度 | 优先级 | 预计用时 |
|------|----------|----------|--------|----------|
| Fuzzy Search | 高 | 中 | 🟡 P1 | 4 小时 |
| 收藏夹功能 | 中 | 低 | 🟢 P2 | 3 小时 |
| 最近打开记录 | 中 | 低 | 🟢 P2 | 2 小时 |
| 批量操作 | 低 | 高 | 🔴 P3 | 6 小时 |

**总预计用时**: 15 小时

---

## 🎯 功能规格

### 1. Fuzzy Search (P1)

**目标**: 使用 nucleo-matcher 实现模糊搜索，提升搜索体验

**功能需求**:
- 支持子序列匹配（如 "fbreact" 匹配 "facebook/react"）
- 支持大小写不敏感
- 支持得分排序（匹配度越高越靠前）
- 保持与现有搜索系统的兼容性

**技术实现**:
```toml
# Cargo.toml
[dependencies]
nucleo-matcher = "0.3"  # 已存在，启用 fuzzy 特性
```

**依赖**:
- `nucleo-matcher` (已在 Cargo.toml 中，需启用 fuzzy 特性)
- `src/repo/filter.rs` (新增)

**验收标准**:
- [ ] 搜索 "fbreact" 能匹配 "facebook/react"
- [ ] 搜索结果按匹配度排序
- [ ] 性能：1000 仓库搜索 < 100ms (p95)
- [ ] 单元测试覆盖率 ≥ 90%

---

### 2. 收藏夹功能 (P2)

**目标**: 允许用户收藏重要仓库，快速访问

**功能需求**:
- 收藏/取消收藏当前选中的仓库
- 收藏夹视图（快捷键 `f`）
- 收藏夹持久化到配置文件
- 支持收藏夹分类（可选）

**数据格式**:
```toml
# config.toml
[favorites]
repositories = [
    "github/facebook/react",
    "vercel/next.js",
]
```

**快捷键**:
- `Ctrl+f`: 收藏/取消收藏当前仓库
- `f`: 切换到收藏夹视图
- `Esc`: 返回全部仓库视图

**依赖**:
- `src/favorites/mod.rs` (新增)
- `src/favorites/store.rs` (新增)
- `src/config/types.rs` (扩展)
- `src/app/model.rs` (扩展)

**验收标准**:
- [ ] 可以收藏/取消收藏仓库
- [ ] 收藏夹持久化到配置文件
- [ ] 快捷键 `f` 切换到收藏夹视图
- [ ] 收藏夹视图显示正常

---

### 3. 最近打开记录 (P2)

**目标**: 记录最近访问的仓库，快速跳转

**功能需求**:
- 记录最近打开的 10-20 个仓库
- 最近打开列表（快捷键 `r`）
- 自动记录执行操作的仓库
- 持久化存储

**数据格式**:
```toml
# config.toml
[recent]
repositories = [
    { path = "github/facebook/react", opened_at = "2026-03-07T10:00:00Z" },
    { path = "vercel/next.js", opened_at = "2026-03-07T09:30:00Z" },
]
```

**快捷键**:
- `Ctrl+r`: 切换到最近打开视图
- `Esc`: 返回全部仓库视图

**依赖**:
- `src/recent/mod.rs` (新增)
- `src/recent/store.rs` (新增)
- `src/config/types.rs` (扩展)
- `src/app/update.rs` (记录操作)

**验收标准**:
- [ ] 执行操作时自动记录仓库
- [ ] 最多保留最近 20 个仓库
- [ ] 快捷键 `Ctrl+r` 切换到最近打开视图
- [ ] 持久化到配置文件

---

### 4. 批量操作 (P3)

**目标**: 支持多选仓库，批量执行命令

**功能需求**:
- 多选模式（快捷键 `v` 进入）
- 选择/取消选择当前仓库（空格键）
- 全选/取消全选（`Ctrl+a`）
- 批量执行操作（打开编辑器、执行命令）
- 显示选中数量

**UI 设计**:
```
╭─ Repositories [3 selected] ───────────────────────────────────╮
│ [✓] github_facebook_react           main    ● dirty          │
│ [✓] vercel_next.js                  main    ✓ clean          │
│ [ ] personal_my_project             feat    ✓ clean          │
│ [✓] company_internal_tool           dev     ✓ clean          │
╰───────────────────────────────────────────────────────────────╯
```

**快捷键**:
- `v`: 进入/退出多选模式
- `Space`: 选择/取消选择当前仓库
- `Ctrl+a`: 全选/取消全选
- `Enter`: 对选中仓库执行操作

**依赖**:
- `src/app/model.rs` (扩展 selection 字段)
- `src/action/batch.rs` (新增)
- `src/ui/widgets/batch_menu.rs` (新增)
- `src/handler/selection.rs` (新增)

**验收标准**:
- [ ] 可以选择多个仓库
- [ ] 显示选中数量
- [ ] 批量执行命令
- [ ] 批量操作有进度提示

---

## 🏗️ 技术架构

### 模块依赖图

```
src/
├── app/
│   ├── model.rs          ← 扩展：favorites, recent, selection
│   ├── msg.rs            ← 扩展：Favorite, Recent, Batch 消息
│   └── update.rs         ← 扩展：处理新消息
├── favorites/
│   ├── mod.rs
│   └── store.rs          ← 收藏夹存储
├── recent/
│   ├── mod.rs
│   └── store.rs          ← 最近打开存储
├── repo/
│   ├── filter.rs         ← Fuzzy Search 实现
│   └── discover.rs
├── action/
│   ├── execute.rs
│   └── batch.rs          ← 批量操作执行
├── ui/
│   ├── widgets/
│   │   ├── batch_menu.rs ← 批量操作菜单
│   │   └── ...
│   └── render.rs         ← 渲染选中状态
├── handler/
│   ├── keyboard.rs       ← 扩展快捷键
│   └── selection.rs      ← 选择逻辑
└── config/
    └── types.rs          ← 扩展配置结构
```

### 数据流

```
用户操作
   ↓
[Keyboard Handler] → AppMsg::ToggleFavorite
   ↓
[Update] → 更新 FavoritesStore
   ↓
[Save Config] → config.toml
```

---

## 📋 实施计划

### Task 1: Fuzzy Search (P1)

**负责人**: Backend Dev  
**用时**: 4 小时

**交付物**:
- [ ] `src/repo/filter.rs` - 模糊搜索实现
- [ ] `Cargo.toml` - 启用 fuzzy 特性
- [ ] `src/app/update.rs` - 集成模糊搜索
- [ ] 单元测试 (≥10 个)
- [ ] 基准测试 (性能对比)

**步骤**:
1. 启用 `nucleo-matcher` 依赖
2. 实现 `filter_repos_fuzzy` 函数
3. 集成到搜索系统（可选切换）
4. 编写测试
5. 性能基准测试

---

### Task 2: 收藏夹功能 (P2)

**负责人**: Full Stack Dev  
**用时**: 3 小时

**交付物**:
- [ ] `src/favorites/mod.rs`
- [ ] `src/favorites/store.rs`
- [ ] `src/config/types.rs` - 扩展配置
- [ ] `src/app/model.rs` - 扩展状态
- [ ] `src/handler/keyboard.rs` - `Ctrl+f` 快捷键
- [ ] UI 渲染支持
- [ ] 单元测试 (≥8 个)

**步骤**:
1. 定义 FavoritesStore 数据结构
2. 实现收藏/取消收藏 API
3. 集成到配置文件（加载/保存）
4. 添加快捷键处理
5. UI 渲染收藏夹视图
6. 编写测试

---

### Task 3: 最近打开记录 (P2)

**负责人**: Full Stack Dev  
**用时**: 2 小时

**交付物**:
- [ ] `src/recent/mod.rs`
- [ ] `src/recent/store.rs`
- [ ] `src/config/types.rs` - 扩展配置
- [ ] `src/app/update.rs` - 记录操作
- [ ] UI 渲染支持
- [ ] 单元测试 (≥6 个)

**步骤**:
1. 定义 RecentStore 数据结构
2. 实现添加/查询 API
3. 集成到配置文件
4. 在执行操作时自动记录
5. UI 渲染最近打开视图
6. 编写测试

---

### Task 4: 批量操作 (P3)

**负责人**: Backend Dev + Frontend Dev  
**用时**: 6 小时

**交付物**:
- [ ] `src/app/model.rs` - selection 字段
- [ ] `src/handler/selection.rs` - 选择逻辑
- [ ] `src/action/batch.rs` - 批量执行
- [ ] `src/ui/widgets/batch_menu.rs` - 批量菜单
- [ ] `src/ui/render.rs` - 渲染选中状态
- [ ] 集成测试 (≥5 个)

**步骤**:
1. 扩展 Model 支持多选状态
2. 实现选择/取消选择逻辑
3. 批量操作 UI（复选框、计数）
4. 批量命令执行（并发控制）
5. 进度提示和错误处理
6. 编写测试

---

## 🧪 测试策略

### 单元测试

| 模块 | 测试范围 | 目标覆盖率 |
|------|----------|------------|
| repo/filter | fuzzy 匹配算法 | 95%+ |
| favorites/store | 收藏 CRUD | 90%+ |
| recent/store | 最近打开 CRUD | 90%+ |
| action/batch | 批量执行 | 90%+ |
| handler/selection | 选择逻辑 | 85%+ |

### 集成测试

| 测试场景 | 验证点 |
|----------|--------|
| Fuzzy Search | 搜索 "fbreact" → 匹配 "facebook/react" |
| 收藏夹 | 收藏 → 保存配置 → 重启 → 加载收藏夹 |
| 最近打开 | 执行操作 → 记录 → 查询最近列表 |
| 批量操作 | 选择 3 个 → 批量打开 → 3 个窗口 |

### 性能测试

| 测试 | 目标 |
|------|------|
| fuzzy_search_1000 | < 100ms (p95) |
| favorites_load_100 | < 10ms |
| batch_execute_10 | < 1s |

---

## 📈 验收标准

### 功能验收
- [ ] Fuzzy Search 正常工作，支持子序列匹配
- [ ] 收藏夹可以添加/删除/持久化
- [ ] 最近打开自动记录，最多 20 个
- [ ] 批量操作可以选择多个仓库并执行命令

### 性能验收
- [ ] Fuzzy Search: 1000 仓库 < 100ms
- [ ] 收藏夹加载: 100 个收藏 < 10ms
- [ ] 批量操作: 10 个仓库 < 1s

### 体验验收
- [ ] 所有操作可通过键盘完成
- [ ] 快捷键不冲突
- [ ] UI 显示清晰（选中状态、收藏标记）
- [ ] 错误提示友好

### 代码质量
- [ ] 所有测试通过 (目标：160+ tests)
- [ ] Clippy 无警告
- [ ] cargo fmt 格式化
- [ ] 代码审查通过

---

## ⚠️ 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| nucleo-matcher 性能 | 中 | 基准测试验证，必要时降级为普通搜索 |
| 配置文件膨胀 | 低 | 限制收藏夹/最近打开数量 |
| 批量操作资源占用 | 中 | 并发控制（最多 5 个同时） |
| 快捷键冲突 | 低 | 使用 Ctrl 组合键 |

---

## 📅 时间线

```
Day 1 (2026-03-07)
├─ 09:00-13:00: Task 1 - Fuzzy Search ✅
├─ 14:00-17:00: Task 2 - 收藏夹功能 ✅
└─ 17:00-19:00: Task 3 - 最近打开记录 ✅

Day 2 (2026-03-08)
├─ 09:00-12:00: Task 4a - 多选逻辑
├─ 13:00-16:00: Task 4b - 批量操作 UI
├─ 16:00-18:00: Task 4c - 批量执行
└─ 18:00-19:00: 最终测试验证

Day 3 (2026-03-09)
├─ 09:00-11:00: 代码审查
├─ 11:00-12:00: 修复问题
└─ 13:00-14:00: Phase 4 完成报告
```

**总用时**: 15 小时

---

## 🔗 相关文档

- [PRD v2](./ghclone-prd-v2.md) - Section 10: 开发计划
- [Phase 3 完成报告](./PHASE3_COMPLETE.md)
- [性能测试报告](./PERFORMANCE_REPORT.md)

---

## 📝 变更记录

| 日期 | 变更内容 | 版本 |
|------|----------|------|
| 2026-03-07 | 初始版本 | v1.0 |

---

**创建时间**: 2026-03-07  
**维护者**: repotui Team  
**状态**: 📋 计划中
