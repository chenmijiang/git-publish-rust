# git-publish 快速参考手册 (更新于 2026-01-23)

## 1. 版本相关函数速查

### 1.1 版本解析 (domain/version.rs)

```rust
pub fn parse_from_tag(tag: &str) -> Option<Version>
```

从 Git 标签中提取语义版本号。支持 `v1.2.3`、`V1.2.3`、`1.2.3` 格式。

返回 `Some(Version)` 或 `None`（非标准格式）。

---

### 1.2 版本递增 (domain/version.rs)

```rust
pub fn bump(&mut self, bump_type: &VersionBump)
```

根据 VersionBump 类型递增版本号：
- `Major`: x+1, y=0, z=0 (例: 1.2.3 → 2.0.0)
- `Minor`: x, y+1, z=0 (例: 1.2.3 → 1.3.0)
- `Patch`: x, y, z+1 (例: 1.2.3 → 1.2.4)

---

## 2. Git 操作函数速查

### 2.1 获取分支最新标签 (git_ops.rs) ⭐

```rust
pub fn get_latest_tag_on_branch(&self, branch_name: &str) -> Result<Option<String>>
```

查找分支历史中最近的标签。支持本地和远程跟踪分支。

返回 `Ok(Some(tag))` 或 `Ok(None)`（无标签）。

---

### 2.2 获取新增 commits (git_ops.rs) ⭐

```rust
pub fn get_commits_since_tag(
    &self,
    branch_name: &str,
    tag_name: Option<&str>,
) -> Result<Vec<Commit<'_>>>
```

获取自某个标签以来的所有提交，按时间顺序排列。`tag_name = None` 返回所有 commits。

---

### 2.3 创建标签 (git_ops.rs)

```rust
pub fn create_tag(&self, tag_name: &str, branch_name: Option<&str>) -> Result<()>
```

---

### 2.4 推送标签 (git_ops.rs)

```rust
pub fn push_tag(&self, tag_name: &str, remote_name: &str) -> Result<()>
```

推送标签到指定远程仓库。

---

### 2.5 其他 Git 操作

```rust
// 获取所有远程仓库
pub fn list_remotes(&self) -> Result<Vec<String>>

// 检查远程仓库存在性
pub fn remote_exists(&self, remote_name: &str) -> Result<bool>

// 从远程拉取数据
pub fn fetch_from_remote(&self, remote_name: &str, branch_name: &str) -> Result<()>

// 获取当前 HEAD 哈希
pub fn get_current_head_hash(&self) -> Result<String>
```

---

## 3. Analyzer 模块（版本分析）

### 3.1 解析提交消息 (analyzer/version_analyzer.rs)

```rust
pub fn analyze_version_bump(
    messages: &[String],
    config: &ConventionalCommitsConfig,
) -> VersionBump
```

基于 commits 内容决定版本递增类型。优先级：破坏性变更 > 特性 > 修复 > 默认补丁。

---

## 4. 配置函数速查

### 4.1 加载配置 (config.rs)

```rust
pub fn load_config(config_path: Option<&str>) 
    -> Result<Config, Box<dyn std::error::Error>>
```

按优先级加载配置：CLI 参数 → 当前目录 → 用户目录 → 内置默认。

---

## 5. UI 交互函数速查

### 5.1 分支选择 (ui.rs)

```rust
pub fn select_branch(available_branches: &[String]) -> Result<String>
```

交互式提示用户选择要标记的分支。

---

### 5.2 标签格式验证 (ui.rs)

```rust
pub fn validate_tag_format(tag: &str, pattern: &str) -> Result<()>
```

验证标签是否符合配置的模式。

---

### 5.3 标签选择和编辑 (ui.rs)

```rust
pub fn select_or_customize_tag(recommended_tag: &str, _pattern: &str) -> Result<String>
```

让用户选择推荐标签或提供自定义值。

---

## 6. 主程序流程快速参考

### 6.1 完整调用链

```
main()
 │
 ├─→ load_config(config_path)
 │    └─ 搜索: --config → ./gitpublish.toml → ~/.config → defaults
 │
 ├─→ GitRepo::new()
 │    └─ Repository::discover(".")
 │
 ├─→ list_remotes()
 │    └─ 获取所有可用远程仓库
 │
 ├─→ select remote (CLI > config > prompt)
 │    └─ 三层次优先级选择远程
 │
 ├─→ fetch_from_remote("selected_remote", branch)
 │    └─ 拉取所有分支和标签
 │
 ├─→ get_latest_tag_on_branch(branch) ⭐
 │    └─ 从 HEAD 反向查找第一个标签
 │
 ├─→ get_commits_since_tag(branch, tag) ⭐
 │    └─ 收集 tag 到 HEAD 之间的所有 commits
 │
 ├─→ analyze_version_bump(messages, config) ⭐
 │    ├─ parse_commit() × N
 │    └─ 决策: Major/Minor/Patch
 │
 ├─→ Version::parse_from_tag(tag)
 │    └─ 提取版本: "v1.2.3" → Version(1,2,3)
 │
 ├─→ version.bump(bump_type)
 │    └─ 计算新版本: Version(1,2,3) → Version(1,3,0)
 │
 ├─→ pattern.replace("{version}", new_version)
 │    └─ 格式化标签: "v{version}" → "v1.3.0"
 │
 ├─→ [User interactions]
 │    ├─ select_or_customize_tag()
 │    └─ confirm_tag_use()
 │
 ├─→ create_tag(final_tag, Some(branch_name)) ✨ 支持分支参数
 │    └─ tag_lightweight() 在指定分支
 │
 ├─→ confirm_push_tag()
 │    └─ 用户确认是否推送
 │
 ├─→ push_tag(final_tag, "selected_remote") [if confirmed]
 │    └─ 推送到选定的远程
 │
 └─→ Success!
```

---

## 7. 常见任务速查

### 任务: 获取分支当前版本

```rust
let git_repo = GitRepo::new()?;
if let Some(tag) = git_repo.get_latest_tag_on_branch("main")? {
    if let Some(version) = Version::parse_from_tag(&tag) {
        println!("Current version: {}", version);
    }
}
```

### 任务: 分析新增 commits 并决定版本

```rust
let commits = git_repo.get_commits_since_tag("main", latest_tag.as_deref())?;
let messages: Vec<String> = commits
    .iter()
    .filter_map(|c| c.message().map(|m| m.to_string()))
    .collect();

let bump = analyze_version_bump(&messages, &config.conventional_commits);
let mut version = Version::parse_from_tag(&tag).unwrap_or_default();
version.bump(&bump);
```

### 任务: 创建和推送标签到指定分支

```rust
let tag_name = format!("v{}", new_version);
git_repo.create_tag(&tag_name, Some("main"))?;  // ✨ 在 main 分支上创建
git_repo.push_tag(&tag_name, "origin")?;
println!("Successfully published {}", tag_name);
```

---

## 8. 关键常量和默认值

默认分支标签格式：
```rust
main    = "v{version}"
develop = "d{version}"
gray    = "g{version}"
```

支持的 Commit 类型：
```rust
["feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf"]
```

破坏性变更指示符：
```rust
["BREAKING CHANGE:", "BREAKING-CHANGE:"]
```

---

**快速参考手册更新于**: 2026 年 1 月 23 日
