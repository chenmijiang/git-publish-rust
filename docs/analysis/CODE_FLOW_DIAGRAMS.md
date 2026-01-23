# git-publish 代码结构可视化

## 1. 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                      git-publish CLI                             │
│                      (main.rs - 379行)                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 第1阶段: 初始化                                          │   │
│  │  • 解析命令行参数 (--config, --branch, --remote, etc)  │   │
│  │  • 加载配置 (config.rs)                                 │   │
│  │  • 选择分支 (ui.rs)                                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│           ↓                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 第2阶段: 远程选择 ⭐                                    │   │
│  │  • list_remotes() - 获取可用远程仓库                    │   │
│  │  • 三层次优先级选择远程: CLI > 配置 > 交互提示         │   │
│  │  • remote_exists() - 验证远程存在性                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│           ↓                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 第3阶段: Git 数据收集                                    │   │
│  │  • GitRepo::new() - 初始化仓库                          │   │
│  │  • fetch_from_remote() - 拉取数据                       │   │
│  │  • get_latest_tag_on_branch() ⭐ - 获取前一个标签      │   │
│  │  • get_commits_since_tag() ⭐ - 获取新增commits        │   │
│  │  (全部在 git_ops.rs - 469行)                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│           ↓                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 第4阶段: 版本计算                                        │   │
│  │  • determine_version_bump() (conventional.rs)           │   │
│  │  • parse_version_from_tag() (version.rs)               │   │
│  │  • bump_version() (version.rs)                         │   │
│  │  • 应用标签模式 pattern.replace("{version}")           │   │
│  └──────────────────────────────────────────────────────────┘   │
│           ↓                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 第5阶段: 用户交互确认 (ui.rs)                            │   │
│  │  • 显示 commit 分析                                     │   │
│  │  • 显示拟议标签变更                                     │   │
│  │  • 选择/自定义标签                                      │   │
│  │  • 验证标签格式                                         │   │
│  └──────────────────────────────────────────────────────────┘   │
│           ↓                                                      │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 第6阶段: 标签创建和推送 (git_ops.rs)                    │   │
│  │  • create_tag() - 创建本地标签                          │   │
│  │  • confirm_push_tag() - 确认推送                        │   │
│  │  • push_tag() - 推送到选定远程                          │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 模块依赖关系

```
main.rs (CLI 入口)
  ├─→ config.rs           (加载配置)
  ├─→ git_ops.rs          (Git 操作) ⭐
  │    ├─ Repository::discover()
  │    ├─ revwalk
  │    ├─ tag operations
  │    └─ push/fetch
  ├─→ conventional.rs     (Commit 分析)
  │    ├─ parse_conventional_commit()
  │    └─ determine_version_bump()
  ├─→ version.rs          (版本管理)
  │    ├─ Version struct
  │    ├─ parse_version_from_tag()
  │    └─ bump_version()
  ├─→ ui.rs               (用户交互)
  │    ├─ 输入/输出格式化
  │    ├─ 交互式选择
  │    └─ 验证和确认
  └─→ boundary.rs         (边界情况)
       └─ BoundaryWarning enum

lib.rs (公共导出)
  └─→ Re-exports all modules
```

---

## 3. 版本获取和计算流程

```
┌─────────────────────────────────────────────────────────────┐
│ Git Repository                                              │
│ Commit History:                                             │
│   HEAD ─→ commit2 ─→ commit1 ─→ commit0 (tagged v1.2.3)   │
│                                                             │
│   new commits: commit2, commit1                             │
│   old tag: v1.2.3                                           │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ get_latest_tag_on_branch() (git_ops.rs: 222-256)           │
│                                                             │
│ 1. get_branch_head_oid(branch_name)                         │
│    ↓ returns: commit2's OID                                 │
│ 2. revwalk(branch_oid) 反向遍历                            │
│    ↓ iterate: commit2 → commit1 → commit0                 │
│ 3. tag_oids 映射所有标签到 OID                             │
│    ↓ {commit0_oid: "v1.2.3", ...}                          │
│ 4. 第一个匹配的就是最新标签                                 │
│    ↓ return "v1.2.3" ✓                                     │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ parse_version_from_tag() (version.rs: 61-76)               │
│                                                             │
│ Input: "v1.2.3"                                             │
│ 1. trim_start_matches('v') → "1.2.3"                       │
│ 2. split('.') → ["1", "2", "3"]                            │
│ 3. parse each to u32 → 1, 2, 3                             │
│ 4. Version::new(1, 2, 3) ✓                                 │
│                                                             │
│ Output: Version { major: 1, minor: 2, patch: 3 }          │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ get_commits_since_tag() (git_ops.rs: 270-323)              │
│                                                             │
│ 1. revwalk from HEAD (commit2)                              │
│ 2. stop at tag (commit0)                                    │
│ 3. collect: [commit2, commit1]                              │
│ 4. reverse for chronological: [commit1, commit2] ✓         │
│                                                             │
│ Output: Vec<Commit> (chronological order)                  │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ determine_version_bump() (conventional.rs: 132-185)        │
│                                                             │
│ For each commit message:                                    │
│   commit1: "fix: bug fix"                                   │
│     → parse_conventional_commit()                           │
│     → type="fix" → has_fixes=true                           │
│                                                             │
│   commit2: "feat: new feature"                              │
│     → parse_conventional_commit()                           │
│     → type="feat" → has_features=true                       │
│                                                             │
│ Decision: has_features=true → return VersionBump::Minor ✓ │
│                                                             │
│ Output: VersionBump::Minor                                  │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ bump_version() (version.rs: 99-115)                         │
│                                                             │
│ Input: Version(1, 2, 3), VersionBump::Minor                │
│ Match Minor:                                                │
│   minor += 1 → 3                                            │
│   patch = 0  → 0                                            │
│ Output: Version { major: 1, minor: 3, patch: 0 } ✓        │
│                                                             │
│ String output: "1.3.0"                                      │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ Apply tag pattern (main.rs: 211-216)                        │
│                                                             │
│ pattern = "v{version}"                                      │
│ version = "1.3.0"                                           │
│ pattern.replace("{version}", version)                       │
│ → "v1.3.0" ✓                                               │
│                                                             │
│ Final tag: "v1.3.0"                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Commit 解析流程详解

```
┌─────────────────────────────────────────────────────────────┐
│ Raw Commit Message                                          │
│                                                             │
│ Examples:                                                   │
│ • "feat(auth): add login"                                   │
│ • "fix(api)!: breaking change"                              │
│ • "chore: update deps"                                      │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ parse_conventional_commit() (conventional.rs: 37-116)      │
│                                                             │
│ Regex 1: ^([a-z]+)\(([^)]+)\)(!?):\s*(.*)                  │
│ ├─ Matches: "feat(auth)!: desc"                            │
│ ├─ Captures: type, scope, !, description                   │
│ └─ is_breaking = ("!" == "!" or "BREAKING CHANGE:" in msg) │
│                                                             │
│ Regex 2: ^([a-z]+)!:\s*(.*)                                │
│ ├─ Matches: "feat!: desc"                                   │
│ ├─ Captures: type, description                             │
│ └─ is_breaking = true                                      │
│                                                             │
│ Regex 3: ^([a-z]+):\s*(.*)                                 │
│ ├─ Matches: "feat: desc"                                    │
│ ├─ Captures: type, description                             │
│ └─ is_breaking = "BREAKING CHANGE:" in message             │
│                                                             │
│ Default: Non-conventional commits → type="chore"           │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ ParsedCommit Result                                         │
│                                                             │
│ struct ParsedCommit {                                       │
│     type: "feat",                  // from regex match      │
│     scope: Some("auth"),           // from scope group      │
│     description: "add login",      // from description      │
│     is_breaking_change: false,     // ! or BREAKING CHANGE │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────────┐
│ Analyze for version bump                                    │
│                                                             │
│ Check each message:                                         │
│   1. is_breaking_change? → Major ← HIGHEST PRIORITY        │
│   2. matches major_keywords?                                │
│   3. type == "feat"/"feature"? → Minor                      │
│   4. type == "fix"/"perf"/"refactor"? → Patch              │
│   5. default → Patch                                        │
│                                                             │
│ Decision: if any has breaking → Major                       │
│           else if any has features → Minor                  │
│           else if any has fixes → Patch                     │
│           else → Patch (default)                            │
└─────────────────────────────────────────────────────────────┘
```

---

## 5. 配置加载优先级

```
┌────────────────────────────────────────────────┐
│ Configuration Loading Priority                  │
├────────────────────────────────────────────────┤
│                                                │
│ 1. Command line parameter: --config <path>    │
│    ↓ (if exists)                              │
│    Load from <path>                           │
│    ↓ (if not exists)                          │
│                                                │
│ 2. Current directory: ./gitpublish.toml       │
│    ↓ (if exists)                              │
│    Load from ./gitpublish.toml                │
│    ↓ (if not exists)                          │
│                                                │
│ 3. User config dir: ~/.config/.gitpublish.toml│
│    ↓ (if exists)                              │
│    Load from ~/.config/.gitpublish.toml       │
│    ↓ (if not exists)                          │
│                                                │
│ 4. Built-in defaults: Config::default()       │
│    → branches: main, develop, gray             │
│    → commit_types: feat, fix, docs, ...       │
│    → breaking indicators: BREAKING CHANGE:    │
│    → behavior: skip_remote_selection = false   │
│                                                │
└────────────────────────────────────────────────┘
```

---

## 6. Git 操作调用栈

```
main.rs
  │
  ├─ GitRepo::new()
  │  └─ Repository::discover(".")
  │     └─ .git directory
  │
  ├─ list_remotes()
  │  ├─ repo.remotes()
  │  ├─ sort with "origin" first
  │  └─ return Vec<String>
  │
  ├─ remote_exists("origin")
  │  ├─ repo.find_remote(name)
  │  ├─ check ErrorCode::NotFound
  │  └─ return bool
  │
  ├─ fetch_from_remote("selected_remote", "main")
  │  ├─ remote.fetch(refspecs)
  │  │  ├─ +refs/heads/*:refs/remotes/origin/*
  │  │  └─ +refs/tags/*:refs/tags/*
  │  └─ update_branch_from_remote("main", "origin")
  │     ├─ find_branch("main", Local)
  │     ├─ find_reference("refs/remotes/origin/main")
  │     └─ set_target(remote_oid) ← fast-forward
  │
  ├─ get_latest_tag_on_branch("main")
  │  ├─ get_branch_head_oid("main") → Oid
  │  ├─ revwalk(branch_oid) ← 反向遍历
  │  ├─ tag_names(None) ← 所有标签
  │  ├─ peel(ObjectType::Any) ← 解包任何类型
  │  └─ match first tag in history → "v1.2.3"
  │
  ├─ get_commits_since_tag("main", "v1.2.3")
  │  ├─ get_branch_head_oid("main") → Oid
  │  ├─ revwalk(branch_oid) ← 反向遍历
  │  ├─ find_commit(oid) ← 收集所有
  │  ├─ stop at tag commit
  │  └─ reverse to chronological order → Vec<Commit>
  │
  ├─ create_tag("v1.3.0")
  │  ├─ head().peel_to_commit()
  │  └─ tag_lightweight(tag_name, commit, false)
  │
  └─ push_tag("v1.3.0", "selected_remote")
     ├─ find_remote("selected_remote")
     ├─ set up SSH credentials
     └─ remote.push(["refs/tags/v1.3.0"])
```

---

## 7. 错误处理决策树

```
              Start
                │
                ▼
         ┌─────────────┐
         │ Fetch Data? │
         └──────┬──────┘
                │
        ┌───────┴───────┐
        │ Auth Error?   │
        ├───────┬───────┤
        ▼       ▼       ▼
       YES     NO    SUCCESS
        │       │       │
        │   WARNING   Continue
        │   Continue   │
        │   ?(y/N)     ▼
        │   │      ┌─────────────────┐
        │   │      │ Get latest tag? │
        │   │      └────────┬────────┘
        │   │               │
        │   │        ┌──────┴──────┐
        │   │        │   Success?  │
        │   │        ├──────┬──────┤
        │   │        ▼      ▼      ▼
        │   │      YES    NO    ERROR
        │   │       │      │      │
        │   │   Continue   │   ABORT
        │   │   tag=None   │
        │   │       │      │
        │   │       ▼      ▼
        │   │   Continue
        │   │       │
        │   │       ▼
        │   └──→ ┌─────────────────────┐
        │        │ Get commits since?  │
        │        └────────┬────────────┘
        │                 │
        │          ┌──────┴──────┐
        │          │ Empty list? │
        │          ├──────┬──────┤
        │          ▼      ▼      ▼
        │         YES    NO   SUCCESS
        │          │      │      │
        │      WARNING    │  Continue
        │      Continue   │      │
        │      ?(y/N)     ▼      ▼
        │       │        │   ┌──────────────────┐
        │       │        │   │ Analyze commits? │
        │       │        │   └────────┬─────────┘
        │       │        │            │
        │       └────────┴────────┬───┘
        │                        │
        │                        ▼
        │        ┌──────────────────────────┐
        │        │ Determine version bump?  │
        │        └────────┬─────────────────┘
        │                 │
        │                 ▼
        │        ┌──────────────────────────┐
        │        │ Parse current version?   │
        │        └────────┬─────────────────┘
        │                 │
        │          ┌──────┴──────┐
        │          │ Parseable?  │
        │          ├──────┬──────┤
        │          ▼      ▼      ▼
        │         YES    NO   version
        │          │      │    defaults to
        │      bump_version   v0.1.0
        │          │      │      │
        │          │   WARNING   │
        │          │   Continue? │
        │          │   (y/N)     │
        │          │      │      │
        │          └──────┴──────┘
        │                 │
        │                 ▼
        │        ┌──────────────────┐
        │        │ User confirm?    │
        │        └────────┬─────────┘
        │                 │
        │          ┌──────┴──────┐
        │          ▼      ▼      ▼
        │         YES    NO   ABORT
        │          │      │      │
        │          │      │      └──→ EXIT
        │          │      │
        │          │      └──→ Rollback?
        │          │             │
        │          ▼             ▼
        │        Create tag
        │          │
        │          ▼
        │        Push tag?
        │        (y/N)
        │       /   |   \
        │      /    |    \
        │    YES   NO   --dry-run
        │     │     │      │
        │     ▼     ▼      ▼
        │   Push  Local  Preview
        │    │    Only    │
        │    ▼    │       ▼
        │  SUCCESS│     EXIT
        │    │    │
        │    └────┴──→ SUCCESS
        │
        └────────────→ EXIT

Legend:
  ↓      : Process flow
  ABORT  : Error and exit
  EXIT   : Normal exit
  ?      : User prompt
```

---

## 8. 代码文件大小分布

```
git_ops.rs          ████████████████████████ 422 行 (27%)
main.rs             ███████████████████░    303 行 (19%)
ui.rs               ███████████████████░    308 行 (20%)
conventional.rs     ███████████░            185 行 (12%)
config.rs           ██████████░             162 行 (10%)
version.rs          ███████░                115 行 (7%)
boundary.rs         ███░                     54 行 (3%)
lib.rs              ░                          6 行 (0%)
                    ├────────────────────────────
                    1555 行 总计
```

---

## 9. 关键数据结构的生命周期

```
GitRepo (Owned)
├─ Created: GitRepo::new()
├─ Lived: 整个 main() 函数作用域
└─ Dropped: main() 结束

Config (Owned)
├─ Created: config::load_config()
├─ Lived: 整个 main() 函数作用域
├─ Used by: 分支选择, 版本计算
└─ Dropped: main() 结束

Repository (Owned by GitRepo)
├─ Created: Repository::discover()
├─ Used by: 所有 git_ops 函数
├─ Lived: GitRepo 生命周期
└─ Dropped: GitRepo 销毁时

Commit (Borrowed from Repository)
├─ Created: revwalk 迭代中
├─ Lived: get_commits_since_tag() 作用域
├─ Converted: 提取 message() 后变为 String
└─ Dropped: Vec<Commit> 销毁时

ParsedCommit (Owned)
├─ Created: parse_conventional_commit() 中 for each message
├─ Used: 版本决策分析中
├─ Lived: 分析函数作用域
└─ Dropped: 分析完成后
```

---

## 10. 执行时间复杂度

| 操作 | 复杂度 | 说明 |
|------|--------|------|
| `get_latest_tag_on_branch()` | O(n) | n = commit 数量（从 HEAD 向后） |
| `get_commits_since_tag()` | O(n) | n = tag 和 HEAD 之间的 commits |
| `parse_conventional_commit()` | O(1) | 正则表达式固定复杂度 |
| `determine_version_bump()` | O(m) | m = commit 数量 × 关键词数量 |
| `fetch_from_remote()` | O(network) | 网络往返延迟 |
| `push_tag()` | O(network) | 网络往返延迟 |

**总体工作流**: O(n + m + network) ≈ **线性复杂度**

---

## 11. 典型错误场景

```
错误场景 1: 不是 Git 仓库
  GitRepo::new()
    └─ Repository::discover(".") fails
       └─ Err: "Not in a git repository"
          └─ main.rs:87 → display_error() → exit(1)

错误场景 2: 分支不存在
  get_branch_head_oid()
    └─ find_branch(name, Local) fails
       └─ Err: anyhow!("Not found")
          └─ main.rs:134 → display_error() → exit(1)

错误场景 3: 标签无法解析版本
  parse_version_from_tag("v1.2") // 只有 2 个部分
    └─ split('.').len() != 3
       └─ return None
          └─ main.rs:187 → BoundaryWarning::UnparsableTag
             └─ display_boundary_warning()
                └─ confirm_action("Use v0.1.0?") → y/N

错误场景 4: 网络认证失败
  fetch_from_remote() → Err(auth error)
    └─ main.rs:101 → detect auth error
       └─ BoundaryWarning::FetchAuthenticationFailed
          └─ display_boundary_warning()
             └─ confirm_action("Continue?") → y/N

错误场景 5: 标签格式不匹配
  validate_tag_format("v1.2.3", "v{version}-release")
    └─ !tag.ends_with(suffix) // 缺少 "-release"
       └─ Err: "Tag does not match pattern"
          └─ main.rs:229 → confirm_tag_use() fails
             └─ user can retry or abort
```

---

**图表生成时间**: 2026 年 1 月 22 日
