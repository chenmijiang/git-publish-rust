# git-publish 快速参考手册

## 1. 版本相关函数速查

### 1.1 版本解析 (version.rs:61-76)

```rust
fn parse_version_from_tag(tag: &str) -> Option<Version>
```

**功能**: 从 Git 标签中提取语义版本号

**输入格式**:
- `"v1.2.3"` ✓
- `"V1.2.3"` ✓ (case-insensitive)
- `"1.2.3"` ✓ (可选前缀)
- `"release-v1.2.3"` ✗ (需要纯版本部分)

**返回**:
```rust
Some(Version { major: 1, minor: 2, patch: 3 })  // 成功
None                                              // 失败（非标准格式）
```

**调用位置**: `main.rs:183` - 基于最新标签解析当前版本

**示例**:
```rust
assert_eq!(
    parse_version_from_tag("v1.2.3"),
    Some(Version::new(1, 2, 3))
);
```

---

### 1.2 版本递增 (version.rs:99-115)

```rust
fn bump_version(mut version: Version, bump_type: &VersionBump) -> Version
```

**功能**: 根据类型递增版本号

**递增规则**:

| 类型 | 规则 | 示例 |
|------|------|------|
| `Major` | x+1, y=0, z=0 | 1.2.3 → 2.0.0 |
| `Minor` | x, y+1, z=0 | 1.2.3 → 1.3.0 |
| `Patch` | x, y, z+1 | 1.2.3 → 1.2.4 |

**示例**:
```rust
let v = Version::new(1, 2, 3);
assert_eq!(bump_version(v, &VersionBump::Major), Version::new(2, 0, 0));
assert_eq!(bump_version(v, &VersionBump::Minor), Version::new(1, 3, 0));
assert_eq!(bump_version(v, &VersionBump::Patch), Version::new(1, 2, 4));
```

---

## 2. Git 操作函数速查

### 2.1 获取分支最新标签 (git_ops.rs:222-256) ⭐

```rust
pub fn get_latest_tag_on_branch(&self, branch_name: &str) -> Result<Option<String>>
```

**功能**: 查找分支历史中最近的标签（当前版本）

**算法**:
1. 获取分支 HEAD commit OID
2. 从 HEAD 反向遍历 commit 历史（新→旧）
3. 构建所有标签的 OID 映射表
4. **返回第一个找到的标签**（最新的）

**返回值**:
```rust
Ok(Some("v1.2.3"))  // 找到标签
Ok(None)            // 没有标签
Err(...)            // 错误（分支不存在等）
```

**调用位置**: `main.rs:200`

**示例场景**:
```
History:
  HEAD ──→ commit2 ──→ commit1 ──→ commit0[tagged: v1.2.3]
  
Returns: Some("v1.2.3")
```

---

### 2.2 获取新增 commits (git_ops.rs:270-323) ⭐

```rust
pub fn get_commits_since_tag(
    &self,
    branch_name: &str,
    tag_name: Option<&str>,
) -> Result<Vec<Commit<'_>>>
```

**功能**: 获取自某个标签以来的所有提交（需要分析的内容）

**算法**:
1. 获取分支 HEAD commit OID
2. 从 HEAD 反向遍历，收集 commits
3. 当遇到指定标签的 commit 时停止
4. **反向排序为年代顺序**（旧→新）

**返回值**:
```rust
// commits 按时间顺序排列
vec![commit_oldest, ..., commit_newest]
```

**调用位置**: `main.rs:212`

**示例场景**:
```
History:
  HEAD ──→ commit2[msg: "feat: new"] ──→ commit1[msg: "fix: bug"] ──→ commit0[v1.2.3]
  
Call: get_commits_since_tag("main", Some("v1.2.3"))
Returns: [commit1("fix: bug"), commit2("feat: new")]
         ↑                      ↑
        oldest              newest
```

**特殊情况**: `tag_name = None` 返回所有 commits

---

### 2.3 从远程拉取 (git_ops.rs:90-152)

```rust
pub fn fetch_from_remote(&self, remote_name: &str, branch_name: &str) -> Result<()>
```

**功能**: 同步远程数据和更新本地分支

**步骤**:
1. 获取远程对象
2. 设置 SSH 凭证回调
3. 执行 fetch 操作
   ```
   +refs/heads/*:refs/remotes/{remote}/*  ← 远程分支
   +refs/tags/*:refs/tags/*               ← 所有标签
   ```
4. 快进合并本地分支

**错误处理**:
- SSH 密钥查找顺序: `id_ed25519` → `id_rsa` → `id_ecdsa` → SSH agent
- 认证错误会在 `main.rs:171-176` 检测并提示用户

**调用位置**: `main.rs:158-197`

---

### 2.4 获取所有远程仓库 (git_ops.rs:36-56)

```rust
pub fn list_remotes(&self) -> Result<Vec<String>>
```

**功能**: 获取所有配置的远程仓库名称，按规范排序

**算法**:
1. 获取仓库中的所有远程名称
2. 排序时 "origin" 优先，其他按字母顺序排列

**返回值**:
```rust
// 例如: ["origin", "upstream", "fork"] - origin 始终在首位
vec!["origin", "upstream"]
```

**调用位置**: `main.rs:116`

**示例**:
```rust
// 假设有 "upstream", "origin", "fork" 三个远程仓库
let remotes = git_repo.list_remotes()?;
// 返回: ["origin", "fork", "upstream"] - origin 优先，其余按字母排序
```

---

### 2.5 检查远程仓库存在性 (git_ops.rs:67-73)

```rust
pub fn remote_exists(&self, remote_name: &str) -> Result<bool>
```

**功能**: 检查指定名称的远程仓库是否存在

**算法**:
1. 尝试查找指定名称的远程
2. 根据错误码判断远程是否存在

**返回值**:
```rust
Ok(true)  // 远程存在
Ok(false) // 远程不存在
Err(...)  // 其他错误
```

**调用位置**: `main.rs:101-112`

**示例**:
```rust
let exists = git_repo.remote_exists("origin")?;  // 检查 origin 是否存在
assert_eq!(exists, true);
```

---

### 2.6 创建标签 (git_ops.rs:389-394)

```rust
pub fn create_tag(&self, tag_name: &str) -> Result<()>
```

**功能**: 创建轻量级标签在当前 HEAD

**实现**:
```rust
let head = self.repo.head()?.peel_to_commit()?;
self.repo.tag_lightweight(tag_name, head.as_object(), false)?;
```

**参数**:
- `tag_name`: 标签名称（如 `"v1.3.0"`）
- `force = false`: 不覆盖现有标签

**调用位置**: `main.rs:318`

---

### 2.5 推送标签 (git_ops.rs:407-468)

```rust
pub fn push_tag(&self, tag_name: &str, remote_name: &str) -> Result<()>
```

**功能**: 推送标签到指定远程仓库

**步骤**:
1. 查找指定远程仓库
2. 设置 SSH 凭证和 push 选项
3. 推送 `refs/tags/{tag_name}`
4. 处理错误（网络、认证、引用）

**参数**:
- `tag_name`: 要推送的标签名
- `remote_name`: 目标远程仓库名（如 "origin"）

**调用位置**: `main.rs:337`

---

### 2.6 获取当前 HEAD 哈希 (git_ops.rs:372-379)

```rust
pub fn get_current_head_hash(&self) -> Result<String>
```

**功能**: 获取当前 HEAD 的完整 40 字符 SHA-1 哈希

**返回值**:
- `Ok(String)`: 完整的 40 字符 SHA-1 哈希
- `Err(...)`: HEAD 无效或分离状态

**调用位置**: `main.rs:230`

---

## 3. Commit 分析函数速查

### 3.1 解析 Conventional Commit (conventional.rs:37-116)

```rust
pub fn parse_conventional_commit(message: &str) -> Option<ParsedCommit>
```

**功能**: 提取标准化 commit 信息中的结构数据

**支持的格式**:

| 格式 | 示例 | 解析结果 |
|------|------|--------|
| `type(scope)!: desc` | `feat(auth)!: add login` | type=feat, scope=auth, breaking=true |
| `type(scope): desc` | `fix(api): handle error` | type=fix, scope=api, breaking=false |
| `type!: desc` | `feat!: redesign` | type=feat, scope=None, breaking=true |
| `type: desc` | `docs: update readme` | type=docs, scope=None, breaking=false |
| 非标准 | `anything` | type=chore, scope=None, breaking=false |

**返回结构**:
```rust
ParsedCommit {
    r#type: "feat",
    scope: Some("auth"),
    description: "add login",
    is_breaking_change: true,
}
```

**调用位置**: `conventional.rs:141` (在 `determine_version_bump()` 内)

**示例**:
```rust
let parsed = parse_conventional_commit("feat(auth): add login").unwrap();
assert_eq!(parsed.r#type, "feat");
assert_eq!(parsed.scope, Some("auth".to_string()));
assert_eq!(parsed.is_breaking_change, false);
```

---

### 3.2 决策版本递增 (conventional.rs:132-185)

```rust
pub fn determine_version_bump(
    commit_messages: &[String],
    config: &config::ConventionalCommitsConfig,
) -> VersionBump
```

**功能**: 基于 commits 内容决定版本递增类型

**决策优先级**:

```
1. 破坏性变更 (BREAKING CHANGE: 或 !)
   └─→ VersionBump::Major ⭐ 最高优先

2. 特性 commits (feat, feature)
   └─→ VersionBump::Minor

3. 修复 commits (fix, perf, refactor)
   └─→ VersionBump::Patch

4. 无标准 commits
   └─→ VersionBump::Patch (默认)
```

**配置参数**:
```rust
config.major_keywords     // 触发 Major 的关键词
config.minor_keywords     // 触发 Minor 的关键词
config.breaking_change_indicators  // 破坏性变更标记
```

**调用位置**: `main.rs:177-178`

**示例**:
```rust
let messages = vec![
    "fix: bug fix".to_string(),
    "feat: new feature".to_string(),
];
let config = ConventionalCommitsConfig::default();

let bump = determine_version_bump(&messages, &config);
assert_eq!(bump, VersionBump::Minor);  // 因为有 feat
```

---

## 4. 配置函数速查

### 4.1 加载配置 (config.rs:144-162)

```rust
pub fn load_config(config_path: Option<&str>) -> Result<Config, Box<dyn std::error::Error>>
```

**功能**: 按优先级加载配置文件

**搜索顺序**:

1. **命令行指定** (--config 参数)
2. **当前目录** (./gitpublish.toml)
3. **用户目录** (~/.config/.gitpublish.toml)
4. **内置默认** (Config::default())

**返回值**:
```rust
Ok(Config {
    branches: HashMap<String, String>,
    conventional_commits: ConventionalCommitsConfig,
    patterns: PatternsConfig,
})
```

**调用位置**: `main.rs:52`

**配置文件示例**:
```toml
[branches]
main = "v{version}"
develop = "d{version}"

[conventional_commits]
types = ["feat", "fix", "docs", ...]
breaking_change_indicators = ["BREAKING CHANGE:", "BREAKING-CHANGE:"]
major_keywords = ["breaking", "deprecate"]
minor_keywords = ["feature", "feat"]

[patterns]
version_format = {major = "{major}.{minor}.{patch}"}
```

---

## 5. UI 交互函数速查

### 5.1 分支选择 (ui.rs:53-75)

```rust
pub fn select_branch(available_branches: &[String]) -> Result<String>
```

**功能**: 交互式选择要标记的分支

**行为**:
- 单分支: 自动选择，无提示
- 多分支: 显示列表，用户输入编号

**返回值**: 选定的分支名称

---

### 5.2 标签格式验证 (ui.rs:120-171)

```rust
pub fn validate_tag_format(tag: &str, pattern: &str) -> Result<()>
```

**功能**: 验证标签是否符合配置的模式

**示例验证**:
```rust
validate_tag_format("v1.2.3", "v{version}")          // ✓ OK
validate_tag_format("v1.2.3", "v{version}-release")  // ✗ Err (缺少后缀)
validate_tag_format("1.2.3", "v{version}")           // ✗ Err (缺少前缀)
validate_tag_format("v1.2.x", "v{version}")          // ✗ Err (非数字版本)
```

---

### 5.3 标签选择和编辑 (ui.rs:202-225)

```rust
pub fn select_or_customize_tag(recommended_tag: &str, _pattern: &str) -> Result<String>
```

**功能**: 让用户选择推荐标签或提供自定义值

**用户选项**:
- `Enter` → 使用推荐值
- `e` → 编辑模式
- 其他 → 自定义标签

---

### 5.4 推送确认 (ui.rs:278-290)

```rust
pub fn confirm_push_tag(tag: &str, remote: &str) -> Result<bool>
```

**功能**: 询问用户是否推送标签到远程

**返回值**:
```rust
true   // 用户确认推送
false  // 用户拒绝（标签仅在本地）
```

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
 │    └─ 三层次优先级选择远程: CLI标志 > 配置(skip_remote_selection) > 交互式提示
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
 ├─→ determine_version_bump(messages, config) ⭐
 │    ├─ parse_conventional_commit() × N
 │    └─ 决策: Major/Minor/Patch
 │
 ├─→ parse_version_from_tag(tag)
 │    └─ 提取版本: "v1.2.3" → Version(1,2,3)
 │
 ├─→ bump_version(current_version, bump_type)
 │    └─ 计算新版本: Version(1,2,3) → Version(1,3,0)
 │
 ├─→ pattern.replace("{version}", new_version)
 │    └─ 格式化标签: "v{version}" → "v1.3.0"
 │
 ├─→ [User interactions]
 │    ├─ select_or_customize_tag()
 │    └─ confirm_tag_use()
 │
 ├─→ create_tag(final_tag)
 │    └─ tag_lightweight() 在 HEAD
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

### 6.2 错误处理检查点

| 行号 | 检查 | 失败行为 | 恢复方式 |
|------|------|--------|--------|
| 52-58 | 加载配置 | 显示错误，退出 | - |
| 92-98 | 初始化仓库 | 显示错误，退出 | - |
| 101-112 | 验证远程存在性 | 显示错误，退出 | - |
| 158-197 | Fetch 数据 | 检测认证错误 | 询问继续 |
| 200-209 | 获取标签 | 显示错误，退出 | - |
| 212-221 | 获取 commits | 显示错误，退出 | - |
| 229-242 | 无新 commits | 显示警告 | 询问继续 |
| 252-279 | 解析版本 | 显示警告，使用 v0.1.0 | 询问接受 |
| 300 | 验证格式 | 显示错误 | 允许重新编辑 |
| 318-321 | 创建标签 | 显示错误，退出 | - |
| 337-340 | 推送标签 | 显示错误，退出 | - |

---

## 7. 常见任务速查

### 任务: 获取分支当前版本

```rust
let git_repo = GitRepo::new()?;
if let Some(tag) = git_repo.get_latest_tag_on_branch("main")? {
    if let Some(version) = parse_version_from_tag(&tag) {
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

let bump = determine_version_bump(&messages, &config.conventional_commits);
let new_version = bump_version(current_version, &bump);
```

### 任务: 创建和推送标签

```rust
let tag_name = format!("v{}", new_version);
git_repo.create_tag(&tag_name)?;
git_repo.push_tag(&tag_name)?;
println!("Successfully published {}", tag_name);
```

### 任务: 验证标签格式

```rust
if let Err(e) = validate_tag_format("v1.2.3", "v{version}") {
    eprintln!("Invalid tag: {}", e);
}
```

---

## 8. 关键常量和默认值

### 默认分支配置

```rust
main    = "v{version}"
develop = "d{version}"
gray    = "g{version}"
```

### 默认 Commit 类型

```rust
["feat", "fix", "docs", "style", "refactor", "test", "chore", "build", "ci", "perf"]
```

### 破坏性变更指示符

```rust
["BREAKING CHANGE:", "BREAKING-CHANGE:"]
```

### 主版本关键词

```rust
["breaking", "deprecate"]
```

### 次版本关键词

```rust
["feature", "feat", "enhancement"]
```

### SSH 密钥搜索路径

```rust
~/.ssh/id_ed25519
~/.ssh/id_rsa
~/.ssh/id_ecdsa
SSH Agent (fallback)
```

---

## 9. 性能提示

| 操作 | 时间复杂度 | 优化建议 |
|------|-----------|--------|
| 获取最新标签 | O(n) | 首次 fetch 后快速 |
| 获取 commits | O(n) | 两个标签间距越小越快 |
| 分析 commits | O(n×m) | m=配置关键词数，通常很小 |
| 网络操作 | O(network) | 使用 SSH 而非 HTTPS |

---

**快速参考手册生成于**: 2026 年 1 月 22 日
