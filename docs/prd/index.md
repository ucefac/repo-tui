# PRD 需求文档索引

本文件夹存放所有产品需求文档 (Product Requirements Document)。

---

## 文档列表

| 文档 | 版本 | 说明 | 状态 | 关联 |
|------|------|------|------|------|
| [ghclone-prd-v1.md](./ghclone-prd-v1.md) | v1 | 初始版本 - 已审查 | ✅ 已归档 | - |
| [ghclone-prd-v2.md](./ghclone-prd-v2.md) | v2 | 当前版本 - 基于审查反馈的完整 PRD | ✅ 当前有效 | - |
| [multi-directory-prd-v1.md](./multi-directory-prd-v1.md) | v1 | 多主目录管理功能 PRD | 📝 待审查 | 主目录管理 |
| [clone-feature-prd-v1.md](./clone-feature-prd-v1.md) | v1 | Git Clone 功能 PRD | 📝 已审查 | Clone 功能 |
| [clone-feature-prd-v2.md](./clone-feature-prd-v2.md) | v2 | Git Clone 功能 PRD（基于审查修复） | 📝 已审查 | Clone 功能 |
| [clone-feature-prd-v3.md](./clone-feature-prd-v3.md) | v3-final | Git Clone 功能 PRD（审查通过） | ✅ **定稿** | Clone 功能 |
| [auto-update-prd-v1.md](./auto-update-prd-v1.md) | v1 | 自动检测更新功能 PRD | 📝 待审查 | 版本管理 |
| [repo-delete-feature.md](./repo-delete-feature.md) | v1 | 仓库删除功能 PRD | ✅ 已完成 | 删除功能 |

---

## 文档规范

### 创建规则

1. **文件名格式**: `ghclone-prd-v{n}.md`
2. **必须包含**: 版本号、更新日期、审查状态
3. **新版本创建时**: 同步更新此索引

### 文档结构

```markdown
# PRD: [产品名称] - v{n}

## 1. 产品概述
## 2. 功能需求
## 3. 技术架构
## 4. UI 设计
## 5. 安全实现
## 6. 错误处理
## 7. 测试策略
## 8. 性能标准
## 9. 开发计划
## 10. 验收标准
```

---

## 变更日志

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-03-10 | - | 新增自动检测更新功能 PRD |
| 2026-03-09 | v3 | Git Clone 功能 PRD v3（修复所有 High Priority 审查问题） |
| 2026-03-09 | v2 | Git Clone 功能 PRD 定稿（基于审查意见修复） |
| 2026-03-09 | v1 | 新增 Git Clone 功能 PRD |
| 2026-03-08 | - | 新增多主目录管理功能 PRD |
| 2026-03-05 | v2 | 基于 codeteam 审查修复安全性、架构和测试问题 |
| 2026-03-XX | v1 | 初始版本创建 |

---

**最后更新**: 2026-03-10
**维护者**: repotui Team

