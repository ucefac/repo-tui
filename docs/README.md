# repotui 开发文档索引

**项目**: GitHub 仓库管理 TUI 工具  
**框架**: Rust + Ratatui + Tokio  
**架构**: Elm (Model-View-Update)

---

## 📚 文档分类

所有项目文档按类型分类存放：

| 分类 | 索引文件 | 说明 |
|------|----------|------|
| **需求文档 (PRD)** | [prd/index.md](./prd/index.md) | 产品需求文档索引 |
| **设计文档 (UI/UX)** | [design/index.md](./design/index.md) | 界面设计规范文档索引 |
| **开发任务** | [task/index.md](./task/index.md) | 开发计划、Phase 报告、修复记录索引 |
| **Bug 修复** | [bugs/index.md](./bugs/index.md) | Bug 分析与修复方案索引 |

---

## 📁 文档管理规范

### 存放规则（强制执行）

| 文档类型 | 存放文件夹 | 索引文件 |
|---------|-----------|---------|
| PRD 需求文档 | `docs/prd/` | `docs/prd/index.md` |
| UI/UX 设计文档 | `docs/design/` | `docs/design/index.md` |
| 开发任务文档 | `docs/task/` | `docs/task/index.md` |
| Bug 修复文档 | `docs/bugs/` | `docs/bugs/index.md` |

**禁止**将上述类型文档直接存放在 `docs/` 根目录。

### 创建流程

1. **确定文档类型**：根据内容判断属于哪一类
2. **创建在对应文件夹**：在指定文件夹内创建 `.md` 文件
3. **更新索引文件**：在对应的 `index.md` 中添加文档条目

### 命名规范

- **PRD**: `ghclone-prd-v{n}.md`
- **设计**: `component-name.md` (小写，`-` 连接)
- **任务**: `phase{n}-complete.md` 或 `feature-name.md`
- **Bug**: `brief-description.md` (小写，`-` 连接)

---

## 🔗 相关链接

- [../CLAUDE.md](../CLAUDE.md) - 开发规范与指南
- [../README.md](../README.md) - 项目说明
- [../Cargo.toml](../Cargo.toml) - 依赖配置

---

**最后更新**: 2026-03-08  
**维护者**: repotui Team
