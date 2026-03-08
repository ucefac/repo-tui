# Phase 3 实施计划

**阶段**: Phase 3 - 增强体验  
**开始日期**: 2026-03-07  
**预计完成**: 2026-03-08  
**状态**: 🚀 进行中

---

## 📋 开发任务

### Task 1: Git 状态检测增强

**优先级**: 🔴 P0  
**预计时间**: 2-3 小时  
**负责人**: Backend Dev

#### 目标
- 实现异步后台 Git 状态检测
- 添加 TTL 缓存机制（5 分钟）
- 支持脏状态实时更新

#### 技术方案
```rust
// 1. 后台任务调度器
struct GitStatusScheduler {
    cache: Arc<DashMap<PathBuf, GitStatus>>,
    tx: mpsc::Sender<CheckRepo>,
}

// 2. TTL 缓存
struct GitStatus {
    is_dirty: bool,
    branch: String,
    last_checked: Instant,
    ttl: Duration, // 5 分钟
}

// 3. 批量检测
async fn check_batch(repos: Vec<Repository>) -> Vec<(usize, GitStatus)>
```

#### 交付物
- [ ] `src/git/status.rs` - Git 状态检测逻辑
- [ ] `src/git/cache.rs` - TTL 缓存实现
- [ ] `src/git/scheduler.rs` - 后台调度器
- [ ] 单元测试：缓存命中/失效、批量检测
- [ ] 集成测试：1000 仓库性能测试

---

### Task 2: 主题支持

**优先级**: 🟡 P1  
**预计时间**: 2 小时  
**负责人**: Frontend Dev

#### 目标
- dark/light 主题切换
- 自定义颜色配置
- 配置文件支持主题设置

#### 技术方案
```rust
// 1. 主题定义
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
}

pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub background: Color,
    pub foreground: Color,
}

// 2. 主题切换
impl App {
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme.name.as_str() {
            "dark" => Theme::light(),
            _ => Theme::dark(),
        };
    }
}

// 3. 配置支持
#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,
}
```

#### 交付物
- [ ] `src/ui/theme.rs` - 主题系统重构
- [ ] `src/config/types.rs` - 主题配置字段
- [ ] `src/ui/widgets/*.rs` - 所有 widget 适配主题
- [ ] 快捷键 `t` - 切换主题
- [ ] 单元测试：主题切换、配置保存

---

### Task 3: 性能优化验证

**优先级**: 🟡 P1  
**预计时间**: 1.5 小时  
**负责人**: Tester + Backend Dev

#### 目标
- 验证虚拟列表渲染性能
- 测试搜索防抖效果
- 基准测试：1000 仓库场景

#### 技术方案
```rust
// 1. 基准测试
#[bench]
fn bench_render_1000_repos(b: &mut Bencher) {
    let repos = create_mock_repos(1000);
    b.iter(|| render_list(&repos));
}

#[bench]
fn bench_search_filter(b: &mut Bencher) {
    let repos = create_mock_repos(1000);
    b.iter(|| filter_repos(&repos, "query"));
}

// 2. 性能指标
// - 渲染时间 < 16ms (60 FPS)
// - 搜索响应 < 50ms (p95)
// - 内存占用 < 50MB
```

#### 交付物
- [ ] `benches/performance.rs` - 性能基准测试
- [ ] 测试报告：性能指标对比
- [ ] 优化建议文档
- [ ] 如有问题：性能优化 PR

---

### Task 4: 响应式布局

**优先级**: 🟢 P2  
**预计时间**: 1.5 小时  
**负责人**: Frontend Dev

#### 目标
- 终端宽度自适应
- 最小尺寸适配（80x24）
- 响应式组件设计

#### 技术方案
```rust
// 1. 宽度断点
const WIDTH_SM: u16 = 60;   // 小屏：隐藏元数据
const WIDTH_MD: u16 = 100;  // 中屏：显示分支
const WIDTH_LG: u16 = 120;  // 大屏：完整信息

// 2. 响应式渲染
fn render_repo_item(repo: &Repository, width: u16) -> Line {
    if width < WIDTH_SM {
        Line::from(repo.name.clone())
    } else if width < WIDTH_MD {
        Line::from(format!("{} ({})", repo.name, repo.branch))
    } else {
        Line::from(format!("{} {} {}", repo.name, repo.branch, status_icon))
    }
}

// 3. 动态布局
fn calculate_layout(area: Rect) -> Layout {
    let constraints = if area.width < WIDTH_SM {
        vec![Length(3), Min(0), Length(3)]
    } else {
        vec![Length(3), Min(0), Length(4)]
    };
    Layout::default().constraints(constraints)
}
```

#### 交付物
- [ ] `src/ui/layout.rs` - 响应式布局计算
- [ ] `src/ui/render.rs` - 条件渲染逻辑
- [ ] 所有 widget 适配响应式
- [ ] 单元测试：不同尺寸下的渲染
- [ ] 手动测试：多终端兼容性

---

## 🏗️ 团队分工

| 角色 | 任务 | 交付物 |
|------|------|--------|
| **Backend Dev** | Task 1, Task 3 | Git 状态模块、性能测试 |
| **Frontend Dev** | Task 2, Task 4 | 主题系统、响应式布局 |
| **Tester** | Task 3 验收 | 性能测试报告、质量报告 |
| **Code Reviewer** | 代码审查 | 审查报告、优化建议 |
| **Product Manager** | 全程跟进 | 验收确认、进度同步 |

---

## 📅 时间线

```
Day 1 (2026-03-07)
├─ 09:00-12:00: Task 1 - Git 状态检测
├─ 13:00-15:00: Task 2 - 主题支持
├─ 15:30-17:00: Task 3 - 性能测试
└─ 17:00-18:30: Task 4 - 响应式布局

Day 2 (2026-03-08)
├─ 09:00-10:30: Code Review
├─ 10:30-12:00: 修复审查问题
├─ 13:00-15:00: 集成测试
├─ 15:00-16:00: 性能验证
└─ 16:00-17:00: Phase 3 完成报告
```

---

## ✅ 验收标准

### 功能验收
- [ ] Git 状态检测异步执行，不阻塞 UI
- [ ] 缓存命中时响应时间 < 1ms
- [ ] 主题切换立即生效，配置持久化
- [ ] 搜索防抖生效，无频繁过滤
- [ ] 响应式布局在 60/80/100/120 宽度下正常显示

### 性能验收
- [ ] 1000 仓库渲染时间 < 16ms
- [ ] 搜索响应时间 < 50ms (p95)
- [ ] 内存占用 < 50MB
- [ ] Git 状态检测 < 5s (1000 仓库，后台)

### 质量验收
- [ ] 所有测试通过（预计 150+ 测试）
- [ ] Clippy 无警告
- [ ] 代码审查通过
- [ ] 文档已更新

---

## 📊 进度跟踪

| 任务 | 状态 | 负责人 | 进度 |
|------|------|--------|------|
| Task 1: Git 状态检测 | 🚀 待开始 | Backend | 0% |
| Task 2: 主题支持 | 🚀 待开始 | Frontend | 0% |
| Task 3: 性能优化 | 🚀 待开始 | Backend+Tester | 0% |
| Task 4: 响应式布局 | 🚀 待开始 | Frontend | 0% |
| Code Review | 🚀 待开始 | Reviewer | 0% |
| 集成测试 | 🚀 待开始 | Tester | 0% |

---

## 📝 变更记录

| 日期 | 变更内容 | 负责人 |
|------|----------|--------|
| 2026-03-07 | Phase 3 计划创建 | PM |

---

**最后更新**: 2026-03-07  
**维护者**: repotui Team
