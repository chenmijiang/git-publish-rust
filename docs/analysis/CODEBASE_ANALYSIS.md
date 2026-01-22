# git-publish 项目代码结构详解

## 项目概览

**git-publish** 是一个 Rust CLI 工具，用于基于 Conventional Commits 分析自动创建和推送 Git 标签，支持语义化版本管理。

- **总代码行数**: 1555 行 Rust 源代码
- **核心模块**: 8 个（main + 7 个子模块）
- **技术栈**: Rust 2021, clap (CLI), serde (TOML), git2 (Git操作), regex (模式匹配)

---

## 1. 模块结构与功能

### 1.1 主文件清单

| 模块 | 行数 | 主要功能 |
|------|------|--------|
| **src/main.rs** | 303 | CLI 入口点，协调整个工作流程 |
| **src/git_ops.rs** | 422 | Git 操作核心（标签、commit、fetch、push） |
| **src/ui.rs** | 308 | 用户交互（提示、确认、输出格式化） |
| **src/conventional.rs** | 185 | Conventional Commit 解析和版本计算 |
| **src/config.rs** | 162 | 配置加载和管理 |
| **src/version.rs** | 115 | 语义版本处理（解析、递增） |
| **src/boundary.rs** | 54 | 边界情况警告处理 |
| **src/lib.rs** | 6 | 公共 API 导出 |

---

## 2. 版本与标签管理流程

### 2.1 完整数据流向

```
user input (args)
    ↓
config::load_config() ────→ gitpublish.toml / defaults
    ↓
git_ops::GitRepo::new() ────→ Repository discovery
    ↓
git_repo.fetch_from_remote() ────→ 同步远程数据
    ↓
git_repo.get_latest_tag_on_branch() ────→ 查找当前最新标签
    ↓
git_repo.get_commits_since_tag() ────→ 获取新增 commits
    ↓
conventional::determine_version_bump() ────→ 分析 commit 类型
    ↓
version::parse_version_from_tag() ────→ 解析当前版本号
    ↓
version::bump_version() ────→ 计算新版本号
    ↓
git_repo.create_tag() ────→ 本地创建标签
    ↓
ui::confirm_push_tag() ────→ 用户确认
    ↓
git_repo.push_tag() ────→ 推送到远程
```

### 2.2 版本获取的核心函数

#### **获取当前版本** (src/version.rs: lines 61-76)

```rust
pub fn parse_version_from_tag(tag: &str) -> Option<Version> {
    // 移除 'v' 或 'V' 前缀
    let clean_tag = tag.trim_start_matches('v').trim_start_matches('V');
    
    // 分割为 major.minor.patch
    let parts: Vec<&str> = clean_tag.split('.').collect();
    if parts.len() != 3 {
        return None;  // 必须是 3 个部分
    }
    
    // 解析每个部分为 u32
    Some(Version::new(major, minor, patch))
}
```

**调用位置**: src/main.rs:183 - 用于解析最新标签中的版本

#### **版本递增** (src/version.rs: lines 99-115)

```rust
pub fn bump_version(mut version: Version, bump_type: &VersionBump) -> Version {
    match bump_type {
        VersionBump::Major => {
            version.major += 1;  // x → x+1
            version.minor = 0;   // reset → 0
            version.patch = 0;   // reset → 0
        }
        VersionBump::Minor => {
            version.minor += 1;  // 固定.y → 固定.y+1
            version.patch = 0;   // reset → 0
        }
        VersionBump::Patch => {
            version.patch += 1;  // 固定.固定.z → 固定.固定.z+1
        }
    }
    version
}
```

**调用位置**: src/main.rs:184 - 基于 commit 分析结果递增版本

### 2.3 前一个标签的获取

#### **git_ops::get_latest_tag_on_branch()** (lines 222-256)

这是获取前一个标签的**关键函数**：

```rust
pub fn get_latest_tag_on_branch(&self, branch_name: &str) -> Result<Option<String>> {
    // 1. 获取分支头部的 commit OID
    let branch_oid = self.get_branch_head_oid(branch_name)?;
    
    // 2. 从分支头部开始反向遍历 commit 历史
    let mut revwalk = self.repo.revwalk()?;
    revwalk.push(branch_oid)?;
    
    // 3. 构建所有标签的 OID 映射
    let mut tag_oids = std::collections::HashMap::new();
    let tags = self.repo.tag_names(None)?;
    
    for tag_name in tags.iter().flatten() {
        if let Ok(tag_ref) = self.repo.find_reference(&format!("refs/tags/{}", tag_name)) {
            // 处理轻量级标签和注解标签
            if let Ok(tag_obj) = tag_ref.peel(git2::ObjectType::Any) {
                let tag_oid = tag_obj.id();
                tag_oids.insert(tag_oid, tag_name.to_string());
            }
        }
    }
    
    // 4. 遍历历史找到最近的标签
    for oid in revwalk {
        match oid {
            Ok(oid) => {
                if let Some(tag_name) = tag_oids.get(&oid) {
                    return Ok(Some(tag_name.clone()));  // ← 第一个找到的就是最新的
                }
            }
            Err(_) => continue,
        }
    }
    
    Ok(None)  // 没有找到标签
}
```

**工作原理**:
- 从分支 HEAD 开始反向遍历提交历史（从新到旧）
- 第一个遇到有标签的提交就是**最新标签**
- 支持轻量级标签（lightweight）和注解标签（annotated）

**调用位置**: src/main.rs:129 - 在确定新版本前获取当前版本

---

## 3. Git 操作核心功能

### 3.1 Git 操作模块概览 (src/git_ops.rs)

所有 Git 操作都通过 `GitRepo` 结构体提供：

| 函数 | 行数 | 功能 |
|------|------|------|
| `GitRepo::new()` | 20-27 | 初始化 Git 仓库 |
| `fetch_from_remote()` | 44-106 | 从远程拉取数据 |
| `update_branch_from_remote()` | 120-194 | 快进合并更新本地分支 |
| `get_branch_head_oid()` | 204-208 | 获取分支 HEAD 的 OID |
| `get_latest_tag_on_branch()` | 222-256 | **获取分支最新标签** ⭐ |
| `get_commits_since_tag()` | 270-323 | **获取自标签以来的 commits** ⭐ |
| `get_current_head_hash()` | 327-333 | 获取当前 HEAD 的 SHA-1 哈希 |
| `create_tag()` | 343-348 | 创建轻量级标签 |
| `push_tag()` | 360-421 | 推送标签到远程 |

### 3.2 Commit 历史获取 (核心逻辑)

#### **get_commits_since_tag()** (lines 270-323)

```rust
pub fn get_commits_since_tag(
    &self,
    branch_name: &str,
    tag_name: Option<&str>,
) -> Result<Vec<Commit<'_>>> {
    // 1. 获取分支 HEAD OID
    let branch_oid = self.get_branch_head_oid(branch_name)?;
    
    // 2. 反向遍历从 branch HEAD 到 tag commit
    let mut revwalk = self.repo.revwalk()?;
    revwalk.push(branch_oid)?;
    
    if let Some(tag_name) = tag_name {
        // 3. 查找 tag OID（支持两种标签类型）
        let tag_oid = self.repo
            .find_reference(&format!("refs/tags/{}", tag_name))
            .ok()
            .and_then(|r| r.peel(git2::ObjectType::Any).ok())
            .map(|obj| obj.id());
        
        let mut commits = Vec::new();
        
        // 4. 收集所有 commit 直到遇到标签 commit
        for oid in revwalk {
            let oid = oid?;
            
            // 遇到标签 commit 时停止
            if let Some(target_oid) = tag_oid {
                if oid == target_oid {
                    break;  // ← 停止条件
                }
            }
            
            if let Ok(commit) = self.repo.find_commit(oid) {
                commits.push(commit);
            }
        }
        
        // 5. 反向排序为年代顺序（旧→新）
        commits.reverse();
        Ok(commits)
    } else {
        // 无标签时返回所有 commits
        // ...
    }
}
```

**返回值**: 年代顺序的 commits（从最旧到最新）

**调用位置**: src/main.rs:141 - 获取需要分析的提交

---

## 4. Conventional Commit 分析

### 4.1 Commit 解析流程

#### **parse_conventional_commit()** (src/conventional.rs: lines 37-116)

支持三种标准格式：

1. **带 scope 的格式**: `type(scope)!: description`
   ```regex
   ^([a-z]+)\(([^)]+)\)(!?):\s*(.*)
   ```

2. **破坏性变更（无 scope）**: `type!: description`
   ```regex
   ^([a-z]+)!:\s*(.*)
   ```

3. **基础格式**: `type: description`
   ```regex
   ^([a-z]+):\s*(.*)
   ```

**解析结果** (`ParsedCommit` 结构):
```rust
pub struct ParsedCommit {
    pub r#type: String,           // 如: "feat", "fix"
    pub scope: Option<String>,     // 如: "auth", "api"
    pub description: String,       // 提交描述
    pub is_breaking_change: bool,  // 是否破坏性变更
}
```

**特殊处理**:
- 非标准 commit 默认为 `type = "chore"`
- 检查 `BREAKING CHANGE:` 关键字判断破坏性变更
- 支持 `!` 标记表示破坏性变更

### 4.2 版本递增决策

#### **determine_version_bump()** (src/conventional.rs: lines 132-185)

基于 commit 类型和配置决策版本递增：

```rust
pub fn determine_version_bump(
    commit_messages: &[String],
    config: &config::ConventionalCommitsConfig,
) -> VersionBump {
    let mut has_breaking_changes = false;
    let mut has_features = false;
    let mut has_fixes = false;
    
    for message in commit_messages {
        let parsed_commit = parse_conventional_commit(message);
        
        if let Some(parsed) = parsed_commit {
            // 1. 检查破坏性变更 → Major
            if parsed.is_breaking_change {
                has_breaking_changes = true;
            }
            
            // 2. 检查配置中的关键词
            for keyword in &config.major_keywords {
                if message.to_lowercase().contains(keyword) {
                    has_features = true;
                }
            }
            
            // 3. 检查 commit 类型
            match parsed.r#type.as_str() {
                "feat" | "feature" => has_features = true,  // → Minor
                "fix" | "perf" | "refactor" => has_fixes = true,  // → Patch
                _ => {}
            }
        }
        
        // 提前返回以优化性能
        if has_breaking_changes {
            return VersionBump::Major;
        }
    }
    
    // 优先级: Major > Minor > Patch > Default
    if has_features {
        VersionBump::Minor
    } else if has_fixes {
        VersionBump::Patch
    } else {
        VersionBump::Patch  // 默认
    }
}
```

**决策优先级**:
```
Breaking Changes (BREAKING CHANGE:, !) → Major
Feature commits (feat, feature) → Minor  
Fix commits (fix, perf, refactor) → Patch
No conventional commits → Patch (default)
```

**调用位置**: src/main.rs:177-178 - 基于 commits 计算版本递增类型

---

## 5. 主程序流程详解

### 5.1 main() 函数完整流程 (src/main.rs: lines 38-283)

```
┌─ CLI 参数解析 (Args::parse)
│  ├─ --config: 自定义配置文件
│  ├─ --branch: 指定分支
│  ├─ --force: 跳过确认
│  ├─ --dry-run: 预览模式
│  ├─ --list: 列出配置分支
│  └─ --version: 显示版本
│
├─ 处理特殊标志
│  ├─ --version: 打印版本退出
│  └─ --list: 列出分支退出
│
├─ 第 1 阶段: 配置和分支选择
│  ├─ 加载配置 (config::load_config)
│  ├─ 交互式选择分支 (ui::select_branch) [如需]
│  └─ 验证选定分支存在
│
├─ 第 2 阶段: Git 数据收集
│  ├─ 初始化 Git 仓库 (GitRepo::new)
│  ├─ 从远程拉取数据 (git_repo.fetch_from_remote)
│  │  └─ [可能失败] 检查认证错误，询问用户是否继续
│  ├─ 获取最新标签 (git_repo.get_latest_tag_on_branch)
│  └─ 获取新增 commits (git_repo.get_commits_since_tag)
│     └─ [无新 commits] 显示警告，询问用户是否继续
│
├─ 第 3 阶段: 版本计算
│  ├─ 显示 commit 分析 (ui::display_commit_analysis)
│  ├─ 决策版本递增 (conventional::determine_version_bump)
│  ├─ 解析当前版本 (version::parse_version_from_tag)
│  │  └─ [解析失败] 显示警告，使用 v0.1.0
│  ├─ 计算新版本 (version::bump_version)
│  └─ 应用标签模式 (pattern.replace("{version}"))
│
├─ 第 4 阶段: 标签确认
│  ├─ 显示拟议标签 (ui::display_proposed_tag)
│  ├─ 用户选择/自定义标签 (ui::select_or_customize_tag)
│  └─ 验证标签格式并确认 (ui::confirm_tag_use)
│
├─ 第 5 阶段: 标签创建和推送
│  ├─ [--dry-run] 预览操作后退出
│  ├─ 创建本地标签 (git_repo.create_tag)
│  ├─ 询问是否推送 (ui::confirm_push_tag)
│  │  └─ [--force] 自动推送
│  ├─ [确认推送] 推送到远程 (git_repo.push_tag)
│  └─ 显示最终结果
│
└─ 返回成功或错误
```

### 5.2 关键决策点

| 行号 | 操作 | 处理方式 |
|------|------|--------|
| 94-126 | Fetch 失败 | 检查认证错误，可继续 |
| 129-138 | 获取标签失败 | 显示错误，退出 |
| 141-150 | 获取 commits 失败 | 显示错误，退出 |
| 158-171 | 无新 commits | 显示警告，询问继续 |
| 181-208 | 解析版本失败 | 显示警告，使用 v0.1.0 |
| 222-226 | --force/--dry-run | 跳过交互 |
| 234-242 | --dry-run 模式 | 预览后退出 |

---

## 6. 配置管理

### 6.1 配置加载顺序 (src/config.rs: lines 144-162)

```
1. --config 指定的路径
   ↓ (如果不存在)
2. ./gitpublish.toml (当前目录)
   ↓ (如果不存在)
3. ~/.config/.gitpublish.toml (用户配置目录)
   ↓ (如果不存在)
4. Config::default() (内置默认值)
```

### 6.2 默认配置

```toml
[branches]
main = "v{version}"
develop = "d{version}"
gray = "g{version}"

[conventional_commits]
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf"]
breaking_change_indicators = ["BREAKING CHANGE:", "BREAKING-CHANGE:"]
major_keywords = ["breaking", "deprecate"]
minor_keywords = ["feature", "feat", "enhancement"]

[patterns]
version_format = {major = "{major}.{minor}.{patch}", ...}
```

---

## 7. 错误和边界情况处理

### 7.1 边界警告类型 (src/boundary.rs)

```rust
pub enum BoundaryWarning {
    NoNewCommits { latest_tag, current_commit_hash },
    UnparsableTag { tag, reason },
    TagMismatchPattern { tag, pattern },
    FetchAuthenticationFailed { remote },
}
```

### 7.2 错误处理策略

| 情况 | 处理 | 用户选项 |
|------|------|--------|
| Fetch 认证失败 | 显示警告 | y/N 继续/取消 |
| 无新 commits | 显示警告 | y/N 继续/取消 |
| 标签无法解析 | 显示警告，使用 v0.1.0 | y/N 接受/取消 |
| 仓库不是 Git | 显示错误 | 退出 |
| 分支不存在 | 显示错误 | 退出 |

---

## 8. 用户交互 (src/ui.rs)

### 8.1 交互函数清单

| 函数 | 功能 | 返回值 |
|------|------|--------|
| `display_error()` | 显示红色错误信息 | - |
| `display_success()` | 显示绿色成功信息 | - |
| `display_status()` | 显示黄色状态信息 | - |
| `select_branch()` | 交互式选择分支 | String |
| `confirm_action()` | 是/否确认 | bool |
| `select_or_customize_tag()` | 选择或编辑标签 | String |
| `confirm_tag_use()` | 验证格式并确认 | bool |
| `confirm_push_tag()` | 确认推送标签 | bool |
| `validate_tag_format()` | 验证标签是否匹配模式 | Result |

### 8.2 标签验证逻辑

```rust
pub fn validate_tag_format(tag: &str, pattern: &str) -> Result<()> {
    // 模式: "v{version}-release"
    // tag:    "v1.2.3-release"
    
    let parts: Vec<&str> = pattern.split("{version}").collect();
    let prefix = parts[0];   // "v"
    let suffix = parts[1];   // "-release"
    
    // 检查前缀
    if !tag.starts_with(prefix) { return Err(...); }
    
    // 检查后缀
    if !tag.ends_with(suffix) { return Err(...); }
    
    // 检查版本部分只包含数字和点
    let version_part = &tag[prefix.len()..tag.len() - suffix.len()];
    if !version_part.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(...);
    }
    
    Ok(())
}
```

---

## 9. 数据结构和类型

### 9.1 核心结构体

```rust
// 版本
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

// 版本递增类型
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

// 解析后的 commit
pub struct ParsedCommit {
    pub r#type: String,
    pub scope: Option<String>,
    pub description: String,
    pub is_breaking_change: bool,
}

// 配置
pub struct Config {
    pub branches: HashMap<String, String>,
    pub conventional_commits: ConventionalCommitsConfig,
    pub patterns: PatternsConfig,
}

// Git 仓库包装
pub struct GitRepo {
    repo: Repository,  // git2::Repository
}
```

---

## 10. 关键函数快速查询

### 10.1 版本相关

| 函数 | 路径 | 主要逻辑 |
|------|------|--------|
| `parse_version_from_tag` | version.rs:61-76 | 从标签解析 semver |
| `bump_version` | version.rs:99-115 | 递增版本号 |
| `Version::new()` | version.rs:28-34 | 创建版本对象 |
| `Version::fmt()` | version.rs:37-41 | 格式化为字符串 |

### 10.2 Git 操作相关

| 函数 | 路径 | 主要逻辑 |
|------|------|--------|
| `get_latest_tag_on_branch` | git_ops.rs:222-256 | **查找前一个标签** ⭐ |
| `get_commits_since_tag` | git_ops.rs:270-323 | **获取新增 commits** ⭐ |
| `fetch_from_remote` | git_ops.rs:44-106 | 拉取远程数据 |
| `create_tag` | git_ops.rs:343-348 | 创建本地标签 |
| `push_tag` | git_ops.rs:360-421 | 推送标签 |

### 10.3 Commit 分析相关

| 函数 | 路径 | 主要逻辑 |
|------|------|--------|
| `parse_conventional_commit` | conventional.rs:37-116 | 解析 commit 格式 |
| `determine_version_bump` | conventional.rs:132-185 | 决策版本递增类型 |

### 10.4 配置相关

| 函数 | 路径 | 主要逻辑 |
|------|------|--------|
| `load_config` | config.rs:144-162 | 按优先级加载配置 |
| `Config::default()` | config.rs:115-128 | 内置默认配置 |

---

## 11. 执行示例

### 11.1 完整流程示例

```bash
$ git-publish --branch main
```

**步骤分解**:

1. **解析参数** → branch = "main"

2. **加载配置** → branches.get("main") = "v{version}"

3. **初始化 Git** → Repository::discover(".")

4. **Fetch 远程**:
   ```
   git fetch origin +refs/heads/*:refs/remotes/origin/* +refs/tags/*:refs/tags/*
   ```

5. **获取最新标签**:
   ```
   revwalk from HEAD → find first commit with tag → return "v1.2.3"
   ```

6. **获取新增 commits**:
   ```
   revwalk from HEAD → stop at "v1.2.3" commit → return [commit1, commit2, ...]
   ```

7. **分析 commits**:
   ```
   message: "feat(auth): add login"
   parsed: type="feat", is_breaking=false
   decision: has_features=true → VersionBump::Minor
   ```

8. **计算新版本**:
   ```
   current: "1.2.3"
   bump: Minor
   result: "1.3.0"
   ```

9. **应用模式**:
   ```
   template: "v{version}"
   result: "v1.3.0"
   ```

10. **用户交互**:
    ```
    Display: v1.2.3 → v1.3.0
    Confirm: (y/N)? y
    ```

11. **创建标签**:
    ```
    repo.tag_lightweight("v1.3.0", HEAD_commit, false)
    ```

12. **推送标签**:
    ```
    remote.push(["refs/tags/v1.3.0"], ...)
    ```

---

## 12. 代码设计要点

### 12.1 架构特点

✅ **分层设计**:
- **CLI 层** (main.rs): 参数处理、流程编排
- **业务逻辑层** (git_ops, conventional, version): 核心功能
- **配置层** (config): 配置加载和默认值
- **UI 层** (ui): 用户交互
- **工具层** (boundary): 边界情况处理

✅ **错误处理**:
- 使用 `Result<T>` 和 `anyhow::Error`
- 生产代码中无 `unwrap()`（测试除外）
- 提供详细的错误上下文

✅ **模式匹配**:
- Git 功能都通过 `GitRepo` 包装器统一接口
- Conventional Commit 支持多种标准格式
- 版本递增逻辑集中在 `determine_version_bump()`

### 12.2 关键依赖

| 库 | 用途 |
|----|------|
| `git2` | Git 仓库操作 |
| `clap` | CLI 参数解析 |
| `serde` + `toml` | TOML 配置解析 |
| `regex` | Commit 格式匹配 |
| `anyhow` | 错误传播 |
| `dirs` | 配置目录查找 |

---

## 13. 总结

### 核心数据流:

```
Git Repository
    ↓
[Fetch] → Get latest tag → Parse version (e.g., "v1.2.3")
    ↓
[Get commits] → Extract messages → Parse conventional commits
    ↓
[Analyze] → Determine version bump type (Major/Minor/Patch)
    ↓
[Calculate] → Bump version (e.g., 1.2.3 → 1.3.0)
    ↓
[Format] → Apply pattern (e.g., "v{version}" → "v1.3.0")
    ↓
[Confirm] → Get user approval → Create tag → Push tag
```

### 最重要的三个函数:

1. **`get_latest_tag_on_branch()`** - 获取当前版本的标签
2. **`get_commits_since_tag()`** - 获取需要分析的提交
3. **`determine_version_bump()`** - 基于提交计算版本递增

---

## 附录: 文件映射

```
src/
├── main.rs                 # 303行 - CLI 入口，流程协调
├── git_ops.rs             # 422行 - Git 操作核心 (⭐ 版本和标签逻辑)
├── conventional.rs        # 185行 - Commit 分析
├── version.rs             # 115行 - 语义版本处理
├── config.rs              # 162行 - 配置管理
├── ui.rs                  # 308行 - 用户交互
├── boundary.rs            # 54行  - 边界警告
└── lib.rs                 # 6行   - 公共导出

tests/
├── integration_test.rs    # 完整工作流测试
├── boundary_test.rs       # 边界情况测试
└── config_test.rs         # 配置加载测试

Cargo.toml                 # 依赖声明
```

---

**最后更新**: 2026 年 1 月 22 日
