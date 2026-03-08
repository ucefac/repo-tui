# 空路径验证 Bug 修复方案

**Bug ID**: CONFIG-001  
**严重程度**: 🔴 高  
**状态**: 已诊断，待修复  
**创建时间**: 2026-03-06

---

## 📋 问题摘要

当配置文件中 `main_directory` 字段为空字符串时，程序运行时崩溃，错误信息：

```
Failed to scan directory: Failed to read directory : No such file or directory (os error 2)
```

---

## 🔍 根因分析

### 漏洞链条

| 阶段 | 发生什么 | 结果 |
|------|----------|------|
| **1. 配置加载** | TOML 解析器读取 `main_directory = ""` | 空字符串被存入配置结构体 |
| **2. 配置验证** | `validate_directory()` 将空字符串规范化为绝对路径 | 空字符串 → 当前工作目录 |
| **3. 验证通过** | 当前目录存在、可读、在主目录内 | ✅ 验证通过 |
| **4. 实际使用** | 原始空字符串被传给目录扫描函数 | ❌ `read_dir("")` 失败 |

### 核心问题

**验证逻辑与实际使用逻辑不一致**：
- 验证时：空字符串被转换为绝对路径，检查通过
- 使用时：原始空字符串直接传给文件系统 API，导致失败

这属于**输入验证不充分**的安全问题。

---

## 🎯 PRD v2 规范对比

根据 PRD v2 第 2.1 节 F2 要求：

> **主目录不存在 → 返回 F1 目录选择**

当前实现违反此要求：
- 配置验证通过但包含无效路径
- 未触发目录选择界面
- 直接尝试使用无效路径导致崩溃

---

## 🛠️ 修复方案

### 阶段 1: 配置验证修复（核心）

**文件**: `src/config/validators.rs`

**修改内容**:

1. 在 `validate_directory()` 函数开头添加**空路径检查**（必须在 `absolutize()` 之前）

```rust
pub fn validate_directory(path: &Path) -> AppResult<PathBuf> {
    // 新增：检查空路径（必须在 absolutize 之前）
    if path.as_os_str().is_empty() {
        return Err(AppError::Config(ConfigError::PathError(
            "main_directory is empty in config file".to_string()
        )));
    }
    
    // 现有验证逻辑...
    let abs_path = path
        .absolutize()
        .map_err(|e| ConfigError::PathError(e.to_string()))?
        .to_path_buf();
    
    // ... 后续验证
}
```

**验证链更新**（5+1 层）：
```
1. 空路径检查 ← 新增
2. absolutize() - 规范化为绝对路径
3. exists() - 检查存在性
4. is_dir() - 检查是目录
5. starts_with(home) - 检查在主目录内
6. read_dir() - 检查读取权限
```

### 阶段 2: 错误处理优化

**文件**: `src/app/update.rs`

**修改内容**:

1. 修改 `AppMsg::ConfigLoaded` 错误处理分支（约第 110-128 行）

```rust
AppMsg::ConfigLoaded(result) => {
    match result {
        Ok(config) => {
            // 现有逻辑...
        }
        Err(e) => {
            app.error_message = Some(e.user_message());
            
            // 修改：当配置无效时触发目录选择
            match e {
                // 配置不存在 → 目录选择
                ConfigError::NotFound(_) => {
                    app.state = AppState::ChoosingDir {
                        path: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
                        entries: Vec::new(),
                        selected_index: 0,
                    };
                    runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                    ));
                }
                
                // 新增：配置无效（如空路径）→ 也触发目录选择
                ConfigError::PathError(_) |
                ConfigError::DirectoryNotFound(_) |
                ConfigError::NotADirectory(_) |
                ConfigError::NoReadPermission(_) => {
                    // 触发 F1 目录选择器
                    app.state = AppState::ChoosingDir {
                        path: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
                        entries: Vec::new(),
                        selected_index: 0,
                    };
                    runtime.dispatch(crate::app::msg::Cmd::ScanDirectory(
                        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                    ));
                }
                
                // 其他错误 → 错误状态
                _ => {
                    app.state = AppState::Error {
                        message: e.user_message(),
                    };
                }
            }
        }
    }
    app.loading = false;
    app.loading_message = None;
}
```

**关键改进**:
- 使用 `unwrap_or_else(|| PathBuf::from("/"))` 替代 `unwrap_or_default()`，确保不会得到空路径
- 将路径验证错误也引导至目录选择器，符合 PRD v2 F2 要求

### 阶段 3（可选）: 配置反序列化验证

**文件**: `src/config/types.rs`

**修改内容**:

1. 添加自定义反序列化器，在 TOML 解析阶段即拒绝空路径

```rust
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;

fn deserialize_non_empty_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = PathBuf::deserialize(deserializer)?;
    if path.as_os_str().is_empty() {
        return Err(serde::de::Error::custom("main_directory cannot be empty"));
    }
    Ok(path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    
    #[serde(deserialize_with = "deserialize_non_empty_path")]
    pub main_directory: PathBuf,
    
    #[serde(default)]
    pub editors: EditorConfig,
    
    // ...
}
```

### 阶段 4（可选）: 错误 UI 优化

**文件**: `src/ui/render.rs`

**修改内容**:

1. 当显示配置错误时，提供更友好的提示

```rust
// 在错误 UI 渲染中添加
if let Some(ref error) = app.error_message {
    if error.contains("empty") {
        // 显示引导信息
        let help_text = "Press Enter to select a directory";
        // ... 渲染帮助文本
    }
}
```

---

## 📝 用户引导文案

### 错误提示（验证失败时）

```
主目录路径为空

请在配置文件中设置 main_directory，或按 Enter 选择目录。

示例配置:
main_directory = "/home/username/projects"
```

### 目录选择界面

```
╭─ Select Main Directory ──────────────────────────────────────╮
│                                                               │
│ ⚠️  Previous configuration had empty main_directory          │
│                                                               │
│   ../                                                        │
│   Desktop/                                                   │
│   Documents/                                                 │
│ ▌ Developer/                                                 │
│   Downloads/                                                 │
│                                                               │
│ Current: /home/username/projects                             │
│ Found: 42 Git repositories                                    │
│                                                               │
│ [↑/↓] Navigate  [→] Select  [Esc] Cancel                   │
╰───────────────────────────────────────────────────────────────╯
```

---

## ✅ 验收测试用例

### 配置验证测试

```rust
#[test]
fn test_validate_empty_path() {
    let empty_path = Path::new("");
    let result = validate_directory(empty_path);
    
    assert!(matches!(
        result,
        Err(AppError::Config(ConfigError::PathError(_)))
    ));
}

#[test]
fn test_validate_valid_path() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();
    
    let result = validate_directory(path);
    assert!(result.is_ok());
}

#[test]
fn test_validate_nonexistent_path() {
    let path = Path::new("/nonexistent/path");
    let result = validate_directory(path);
    
    assert!(matches!(
        result,
        Err(AppError::Config(ConfigError::DirectoryNotFound(_)))
    ));
}
```

### 集成测试

```rust
#[tokio::test]
async fn test_empty_config_triggers_dir_chooser() {
    // 创建临时配置文件，main_directory 为空
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(&config_path, r#"
version = "1.0"
main_directory = ""
"#).unwrap();
    
    // 加载配置应失败
    let result = load_config(&config_path);
    assert!(result.is_err());
    
    // 错误应触发目录选择状态
    // ... 验证应用状态转换
}
```

---

## 📊 修复影响评估

| 修改 | 涉及文件 | 预计行数 | 风险 | 优先级 |
|------|----------|----------|------|--------|
| 空路径验证 | `validators.rs` | +10 行 | 低 | 🔴 P0 |
| 错误处理优化 | `update.rs` | +20 行 | 中 | 🔴 P0 |
| 反序列化验证（可选）| `types.rs` | +15 行 | 低 | 🟡 P1 |
| 错误 UI 优化（可选）| `render.rs` | +10 行 | 低 | 🟢 P2 |
| **总计** | **4 文件** | **~55 行** | **低** | - |

---

## ⏱️ 修复时间线

| 阶段 | 文件 | 预计时间 | 依赖 |
|------|------|----------|------|
| 阶段 1 | `validators.rs` | 10 分钟 | 无 |
| 阶段 2 | `update.rs` | 15 分钟 | 阶段 1 |
| 单元测试 | 测试文件 | 10 分钟 | 阶段 1-2 |
| 集成测试 | 手动验证 | 10 分钟 | 全部 |
| **总计** | - | **~45 分钟** | - |

---

## 🔗 相关文档

- [ghclone-prd-v2.md](ghclone-prd-v2.md) - 需求文档 F1、F2 章节
- [CLAUDE.md](../CLAUDE.md) - 开发指南
- [PHASE0_STATUS.md](./PHASE0_STATUS.md) - Phase 0 状态

---

**修复负责人**: 待分配  
**审查人**: 待分配  
**最后更新**: 2026-03-06
