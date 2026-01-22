# CLI UX 优化设计文档

**日期**: 2025-01-22  
**作者**: OpenCode  
**目标**: 改进git-publish CLI的用户体验，处理边界情况，分离tag生成和push逻辑，支持tag自定义

---

## 1. 需求概述

### 1.1 核心需求
1. **边界情况提示** - 对所有边界情况给出清晰的提示
2. **分离tag生成和push** - 用户可以选择是否推送
3. **Tag自定义** - 用户可以采用默认值或自定义tag（需要格式校验）

### 1.2 边界情况清单
| 场景 | 当前行为 | 改进方向 |
|------|--------|--------|
| 无新commits | 询问用户是否继续 | 明确提示：HEAD已被tag标记 |
| Tag已在HEAD | 报告"No new commits" | 提示用户该分支已完成，建议操作 |
| 无法解析tag版本 | 降级到0.1.0 | 警告用户，提示原因 |
| 非标准tag格式 | 可能导致版本解析失败 | 提示用户该tag格式不符合预期 |
| SSH认证失败 | 显示警告但继续 | 改进：允许用户选择是否继续 |

---

## 2. 交互流程设计

### 2.1 主流程（新建tag → push分离）

```
┌─ 选择分支
│
├─ Fetch远程
│  └─ (失败时提示警告，用户可选择继续或退出)
│
├─ 分析commits
│  ├─ 无新commits → [边界1处理]
│  └─ 有新commits → 继续
│
├─ 版本碰撞决定
│  ├─ tag不可解析 → [边界3处理]  
│  └─ 正常 → 建议版本
│
├─ Tag选项确认
│  ├─ [默认] 按Enter采用推荐的tag → 2.1
│  ├─ [自定义] 输入自定义tag → 2.1 + 格式校验
│  └─ [编辑] "e"或其他修改推荐值 → 继续提示直到确认
│
├─ 创建本地tag ✓
│  │
│  └─ 询问是否push到远程?
│     ├─ [是] y → 推送tag
│     └─ [否] n → 退出（tag已创建本地）
│
└─ 完成
```

### 2.2 边界情况处理

#### 边界1：无新commits（HEAD已被tag标记）
```
状态: HEAD已标记为 v1.28.1，之后无新commits

提示信息:
  ⓘ 该分支已完成此版本的发布 (v1.28.1)
  ℹ️ 建议操作:
     1. 如需发布新版本，请先创建新commits
     2. 如需修改此tag，请先删除现有tag
     3. 或在其他分支上继续工作

继续? (y/N): 
```

#### 边界2：无法解析tag版本
```
状态: 最新tag为 'release-2025' 无法解析版本号

提示信息:
  ⚠️  无法从tag 'release-2025' 解析版本号
  将使用初始版本 v0.1.0 作为基础版本
  
  建议: 检查tag格式是否符合预期 (推荐格式: v1.2.3)

确认继续? (y/N):
```

#### 边界3：非标准tag格式（自定义tag时）
```
用户输入: "my-tag-123"
配置的pattern: "v{version}"

提示信息:
  ⚠️  你输入的tag格式不符合配置的pattern
  
  配置pattern: v{version}
  你的输入:   my-tag-123
  
  注: 这不会影响tag创建，但可能影响版本追踪
  
确认使用此tag? (y/N):
```

#### 边界4：SSH认证失败（fetch时）
```
状态: 无法从远程fetch

提示信息:
  ⚠️  无法从远程 'origin' 获取最新数据 (SSH认证失败)
  将使用本地分支数据继续
  
  注: 这可能导致使用过期的tag信息
  
继续? (y/N):
  - 是(y): 使用本地数据
  - 否(n): 退出
```

---

## 3. Tag选项交互设计

### 3.1 Tag确认界面

```
┌─ Tag选项确认 ─────────────────────────────────┐
│                                                 │
│  推荐的新tag: v1.28.2                          │
│                                                 │
│  选项:                                          │
│  - 按Enter采用此tag                            │
│  - 输入自定义tag值                             │
│  - 输入 'e' 或 'edit' 编辑推荐值               │
│                                                 │
│  你的选择 [v1.28.2]: _                         │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 3.2 Tag自定义流程

**用户按Enter** → 采用默认tag
```
确认tag: v1.28.2? (y/N): 
```

**用户输入自定义值** → 格式校验
```
你的选择 [v1.28.2]: release-v1.28.2

检查tag格式... 
⚠️  tag 'release-v1.28.2' 与pattern 'v{version}' 不匹配

确认使用此tag? (y/N):
```

**用户输入'e'编辑** → 提示前缀
```
你的选择 [v1.28.2]: e

编辑模式 - 推荐值为 'v1.28.2'
新的tag值 [v1.28.2]: v1.28.2-rc1

确认tag: v1.28.2-rc1? (y/N):
```

### 3.3 Tag创建→Push分离

**创建后确认** → 用户可修改主意

```
✓ 已创建本地tag: v1.28.2

将tag推送到远程 origin? (y/N):
  - 是(y): 推送到远程
  - 否(n): tag保留在本地，稍后可手动推送
         使用: git push origin v1.28.2
```

---

## 4. 实现架构

### 4.1 新增UI函数

```rust
// 边界情况提示
pub fn display_boundary_warning(boundary_type: BoundaryWarning, context: &str)
pub fn confirm_action_with_reason(prompt: &str, reason: &str) -> Result<bool>

// Tag选项交互
pub fn select_or_customize_tag(recommended_tag: &str, pattern: &str) -> Result<String>
pub fn validate_tag_format(tag: &str, pattern: &str) -> Result<()>
pub fn confirm_tag_use(tag: &str, pattern: &str) -> Result<bool>

// 推送确认
pub fn confirm_push_tag(tag: &str, remote: &str) -> Result<bool>
```

### 4.2 边界情况枚举

```rust
pub enum BoundaryWarning {
    NoNewCommits {
        latest_tag: String,
        latest_commit: String,
    },
    UnparsableTag {
        tag: String,
        reason: String,
    },
    TagMismatchPattern {
        tag: String,
        pattern: String,
    },
    FetchAuthenticationFailed {
        remote: String,
    },
}
```

### 4.3 主流程改动

**main.rs**: 
- 检测边界情况并调用对应的提示函数
- 拆分"创建tag"和"推送tag"为两个独立步骤
- 在每个关键步骤添加边界检查

**git_ops.rs**:
- 无需修改（已有足够的API）

**ui.rs**:
- 新增上述UI函数
- 改进提示文案

---

## 5. 数据流

```
用户选择分支
    ↓
Fetch远程 → [失败?] → 边界提示:FetchAuthenticationFailed
    ↓
分析commits → [无新?] → 边界提示:NoNewCommits → 用户确认?
    ↓
计算版本 → [解析失败?] → 边界提示:UnparsableTag → 用户确认?
    ↓
生成推荐tag
    ↓
Tag选项交互:
    ├─ Enter采用 → 下一步
    ├─ 自定义输入 → 格式校验 → [失败?] → 边界提示:TagMismatchPattern → 用户确认?
    └─ 编辑推荐 → 同上
    ↓
创建本地tag ✓
    ↓
询问是否push → 用户确认?
    ├─ 是 → 推送tag ✓
    └─ 否 → 提示手动命令 ✓
```

---

## 6. 测试计划

### 6.1 边界情况测试
- [ ] 无新commits时的提示和流程
- [ ] 无法解析tag时的提示和流程
- [ ] 非标准tag格式的提示和流程
- [ ] SSH认证失败的提示和流程

### 6.2 交互测试
- [ ] Tag采用默认值
- [ ] Tag自定义输入
- [ ] Tag格式校验
- [ ] Tag创建后确认push
- [ ] Tag创建后取消push

### 6.3 集成测试
- [ ] 完整流程：从选择分支到推送tag
- [ ] 中途取消的各个阶段
- [ ] Dry-run模式下的行为

---

## 7. 成功指标

1. ✅ 用户在所有边界情况下收到清晰的指导
2. ✅ Tag生成和push逻辑分离，用户可选择
3. ✅ 用户可自定义tag，并得到格式反馈
4. ✅ 测试覆盖所有边界情况
5. ✅ 代码通过clippy检查和所有测试
