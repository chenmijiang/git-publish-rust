# git-publish 项目代码结构详解

## 项目概览

**git-publish** 是一个 Rust CLI 工具，用于基于 Conventional Commits 分析自动创建和推送 Git 标签，支持语义化版本管理。

- **核心功能**: 自动化 Git 标签发布，基于 Conventional Commits 规范
- **技术栈**: Rust 2021, clap (CLI), serde (TOML), git2 (Git操作), regex (模式匹配)

---

## 1. 模块结构与功能

| 模块 | 主要功能 |
|------|--------|
| **src/main.rs** | CLI 入口点，协调整个工作流程 |
| **src/git_ops.rs** | Git 操作核心（标签、commit、fetch、push、remote selection） |
| **src/ui.rs** | 用户交互（提示、确认、输出格式化） |
| **src/conventional.rs** | Conventional Commit 解析和版本计算 |
| **src/config.rs** | 配置加载和管理 |
| **src/version.rs** | 语义版本处理（解析、递增） |
| **src/boundary.rs** | 边界情况警告处理 |

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
git_repo.push_tag() ────→ 推送到选定远程
```

### 核心函数

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

---

## 3. Conventional Commit 分析

#### **parse_conventional_commit()** (src/conventional.rs: lines 37-116)

支持三种标准格式：

1. **带 scope 的格式**: `type(scope)!: description`
2. **破坏性变更（无 scope）**: `type!: description`
3. **基础格式**: `type: description`

**决策优先级**:
```
Breaking Changes (BREAKING CHANGE:, !) → Major
Feature commits (feat, feature) → Minor
Fix commits (fix, perf, refactor) → Patch
No conventional commits → Patch (default)
```

---

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
├─ 第 2 阶段: Git 数据收集
│  ├─ 初始化 Git 仓库 (GitRepo::new)
│  ├─ 从选定远程拉取数据 (git_repo.fetch_from_remote)
│  ├─ 获取最新标签 (git_repo.get_latest_tag_on_branch)
│  └─ 获取新增 commits (git_repo.get_commits_since_tag)
│
├─ 第 3 阶段: 版本计算
│  ├─ 显示 commit 分析 (ui::display_commit_analysis)
│  ├─ 决策版本递增 (conventional::determine_version_bump)
│  ├─ 解析当前版本 (version::parse_version_from_tag)
│  ├─ 计算新版本 (version::bump_version)
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
}
```

---

## 7. 总结

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
