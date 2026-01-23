# git-publish 项目代码结构详解

## 项目概览

**git-publish** 是一个 Rust CLI 工具。它根据 Conventional Commits 规范自动创建和推送 Git 标签，支持语义化版本管理。

- **核心功能**: 自动 Git 标签发布
- **技术栈**: Rust 2021, clap (CLI), serde (TOML), git2, regex
- **最后更新**: 2026 年 1 月 23 日

---

## 1. 模块结构与功能

| 模块 | 主要功能 | 行数 |
|------|--------|------|
| **src/main.rs** | CLI 入口点，协调整个工作流程 | ~380 |
| **src/git_ops.rs** | Git 操作核心（标签、commit、fetch、push、remote selection） | ~469 |
| **src/ui.rs** | 用户交互（提示、确认、输出格式化） | ~308 |
| **src/analyzer/** | 版本分析器（commit 解析、版本决策） | ~200 |
| **src/config.rs** | 配置加载和管理 | ~350 |
| **src/domain/** | 核心数据结构（Version, Tag, PreRelease） | ~300 |
| **src/boundary.rs** | 边界情况警告处理 | ~54 |

---

## 2. 核心流程

### 数据流向

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
analyzer::determine_version_bump() ────→ 分析 commit 类型
    ↓
domain::Version::parse_from_tag() ────→ 解析当前版本号
    ↓
domain::Version::bump() ────→ 计算新版本号
    ↓
git_repo.create_tag() ────→ 本地创建标签
    ↓
ui::confirm_push_tag() ────→ 用户确认
    ↓
git_repo.push_tag() ────→ 推送到选定远程
```

### 核心函数

#### **获取当前版本** (src/domain/version.rs)

```rust
pub fn parse_from_tag(tag: &str) -> Option<Version> {
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

#### **版本递增** (src/domain/version.rs)

```rust
pub fn bump(&mut self, bump_type: &VersionBump) {
    match bump_type {
        VersionBump::Major => {
            self.major += 1;  // x → x+1
            self.minor = 0;   // reset → 0
            self.patch = 0;   // reset → 0
        }
        VersionBump::Minor => {
            self.minor += 1;  // 固定.y → 固定.y+1
            self.patch = 0;   // reset → 0
        }
        VersionBump::Patch => {
            self.patch += 1;  // 固定.固定.z → 固定.固定.z+1
        }
    }
}
```

#### **git_ops::get_latest_tag_on_branch()** ⭐ 关键函数

这是获取前一个标签的**关键函数**，支持两种模式：

1. **本地模式** - 仅查找本地标签
2. **远程模式** - 查找远程跟踪分支上的标签

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

#### **git_ops::create_tag()**

此函数在目标分支上创建标签：

```rust
pub fn create_tag(&self, tag_name: &str, branch_name: Option<&str>) -> Result<()> {
    if let Some(branch) = branch_name {
        // 在指定分支上创建标签
        let branch_oid = self.get_branch_head_oid(branch)?;
        let commit = self.repo.find_commit(branch_oid)?;
        self.repo.tag_lightweight(tag_name, commit.as_object(), false)?;
    } else {
        // 在当前 HEAD 上创建标签（默认行为）
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo.tag_lightweight(tag_name, head.as_object(), false)?;
    }
    Ok(())
}
```

---

## 3. Analyzer 模块（版本分析）

### 核心组件

`src/analyzer/version_analyzer.rs` 包含：

1. **ParsedCommit** - 解析后的 Conventional Commit
2. **VersionAnalyzer** - 版本分析逻辑
3. **analyze_version_bump()** - 核心决策函数

### 决策流程

```
commits 列表
    ↓
遍历每个 commit message
    ├─ 解析 Conventional Commit 格式
    ├─ 检测破坏性变更 (BREAKING CHANGE, !)
    ├─ 检测特性 (feat, feature)
    └─ 检测修复 (fix, perf, refactor)
    ↓
决策优先级：
1. 任何破坏性变更 → Major ⭐ 最高
2. 任何特性 → Minor
3. 任何修复 → Patch
4. 无标准格式 → Patch (默认)
    ↓
返回 VersionBump 类型
```

## 4. 主程序流程

```
┌─ CLI 参数解析 (Args::parse)
│  ├─ --config: 自定义配置文件
│  ├─ --branch: 指定分支
│  ├─ --remote: 指定远程仓库
│  ├─ --force: 跳过确认
│  ├─ --dry-run: 预览模式
│  └─ --list: 列出配置分支
│
├─ 第 1 阶段: 配置和分支选择
│  ├─ 加载配置 (config::load_config)
│  ├─ 交互式选择分支 (ui::select_branch) [如需]
│  └─ 验证选定分支存在
│
├─ 第 2 阶段: 远程选择和 Git 数据收集
│  ├─ 初始化 Git 仓库 (GitRepo::new)
│  ├─ 列出可用远程 (git_repo.list_remotes)
│  ├─ 三层次优先级选择远程: CLI > 配置 > 交互提示
│  ├─ 验证远程存在性 (git_repo.remote_exists)
│  ├─ 从选定远程拉取数据 (git_repo.fetch_from_remote)
│  ├─ 获取最新标签 (git_repo.get_latest_tag_on_branch)
│  └─ 获取新增 commits (git_repo.get_commits_since_tag)
│
├─ 第 3 阶段: 版本计算
│  ├─ 显示 commit 分析 (ui::display_commit_analysis)
│  ├─ 决策版本递增 (analyzer::analyze_version_bump)
│  ├─ 解析当前版本 (Version::parse_from_tag)
│  ├─ 计算新版本 (version.bump)
│  └─ 应用标签模式 (pattern.replace("{version}"))
│
├─ 第 4 阶段: 标签确认
│  ├─ 显示拟议标签 (ui::display_proposed_tag)
│  ├─ 用户选择/自定义标签 (ui::select_or_customize_tag)
│  └─ 验证标签格式并确认 (ui::confirm_tag_use)
│
└─ 第 5 阶段: 标签创建和推送
   ├─ [--dry-run] 预览操作后退出
   ├─ 创建本地标签 (git_repo.create_tag)
   ├─ 询问是否推送 (ui::confirm_push_tag)
   │  └─ [--force] 自动推送
   ├─ [确认推送] 推送到选定远程 (git_repo.push_tag)
   └─ 显示最终结果
```

---

## 5. 配置管理

### 配置加载顺序

```
1. --config 指定的路径
   ↓ (如果不存在)
2. ./gitpublish.toml (当前目录)
   ↓ (如果不存在)
3. ~/.config/.gitpublish.toml (用户配置目录)
   ↓ (如果不存在)
4. Config::default() (内置默认值)
```

### 配置结构

```rust
pub struct Config {
    pub branches: HashMap<String, String>,
    pub conventional_commits: ConventionalCommitsConfig,
    pub patterns: PatternsConfig,
    pub behavior: BehaviorConfig,
    pub prerelease: PreReleaseConfig,
}
```

### 默认配置

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
```

---

## 6. 核心数据结构

### 版本 (src/domain/version.rs)

```rust
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub enum VersionBump {
    Major,
    Minor,
    Patch,
}
```

### 解析后的 Commit (src/analyzer/version_analyzer.rs)

```rust
pub struct ParsedCommit {
    pub r#type: String,
    pub scope: Option<String>,
    pub description: String,
    pub is_breaking_change: bool,
}
```

### 标签 (src/domain/tag.rs)

```rust
pub struct Tag {
    pub name: String,
    pub pattern: TagPattern,
}

pub struct TagPattern {
    pub pattern: String,
}
```

### 错误类型 (src/error.rs)

```rust
pub enum GitPublishError {
    Git(#[from] git2::Error),
    Config(String),
    Version(String),
    Tag(String),
    Remote(String),
    Io(#[from] std::io::Error),
}
```

---

## 7. 总结

### 核心数据流

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

### 最重要的三个函数

1. **`get_latest_tag_on_branch()`** (src/git_ops.rs) - 获取当前版本的标签
2. **`get_commits_since_tag()`** (src/git_ops.rs) - 获取需要分析的提交
3. **`analyze_version_bump()`** (src/analyzer) - 基于提交计算版本递增
