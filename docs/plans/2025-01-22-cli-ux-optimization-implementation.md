# CLI UX 优化实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 改进git-publish CLI的用户体验，处理边界情况，分离tag生成和push逻辑，支持tag自定义。

**Architecture:** 在ui.rs中新增边界提示函数和交互函数，在main.rs中调整主流程以支持分离的tag生成和push步骤，添加边界情况检测。主要改动是按照新的交互设计重构main函数的流程逻辑。

**Tech Stack:** Rust, clap, git2, anyhow, regex（用于tag格式验证）

---

## Task 1: 新增边界情况数据结构和UI函数（基础）

**Files:**
- Modify: `src/ui.rs` - 添加新的UI函数
- Create: `src/boundary.rs` - 创建边界情况类型定义

**Step 1: 在src目录创建boundary.rs文件定义边界情况枚举**

创建 `/Users/a10121/github/git-publish-rust/src/boundary.rs`：

```rust
/// 边界情况类型定义
#[derive(Debug, Clone)]
pub enum BoundaryWarning {
    /// 没有新commits（HEAD已被tag标记）
    NoNewCommits {
        latest_tag: String,
        current_commit_hash: String,
    },
    /// 无法解析tag版本
    UnparsableTag {
        tag: String,
        reason: String,
    },
    /// Tag格式不匹配pattern
    TagMismatchPattern {
        tag: String,
        pattern: String,
    },
    /// Fetch认证失败
    FetchAuthenticationFailed {
        remote: String,
    },
}

impl std::fmt::Display for BoundaryWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundaryWarning::NoNewCommits {
                latest_tag,
                current_commit_hash,
            } => {
                write!(
                    f,
                    "No new commits: HEAD {} is already tagged as {}",
                    &current_commit_hash[..7],
                    latest_tag
                )
            }
            BoundaryWarning::UnparsableTag { tag, reason } => {
                write!(f, "Cannot parse tag '{}': {}", tag, reason)
            }
            BoundaryWarning::TagMismatchPattern { tag, pattern } => {
                write!(
                    f,
                    "Tag '{}' does not match pattern '{}'",
                    tag, pattern
                )
            }
            BoundaryWarning::FetchAuthenticationFailed { remote } => {
                write!(f, "Authentication failed for remote '{}'", remote)
            }
        }
    }
}
```

**Step 2: 在ui.rs中添加新的UI函数**

在 `/Users/a10121/github/git-publish-rust/src/ui.rs` 末尾添加以下函数：

```rust
use crate::boundary::BoundaryWarning;

/// 显示边界情况警告
pub fn display_boundary_warning(warning: &BoundaryWarning) {
    match warning {
        BoundaryWarning::NoNewCommits {
            latest_tag,
            current_commit_hash,
        } => {
            println!("\n\x1b[1mⓘ 分支已完成此版本的发布\x1b[0m");
            println!("  当前HEAD: {} (已标记为 {})", &current_commit_hash[..7], latest_tag);
            println!("\n\x1b[33m建议操作:\x1b[0m");
            println!("  1. 如需发布新版本，请先创建新commits");
            println!("  2. 如需修改此tag，请先删除现有tag");
            println!("  3. 或在其他分支上继续工作");
        }
        BoundaryWarning::UnparsableTag { tag, reason } => {
            println!("\n\x1b[1m⚠️  无法从tag解析版本号\x1b[0m");
            println!("  Tag: {}", tag);
            println!("  原因: {}", reason);
            println!("  将使用初始版本 v0.1.0 作为基础版本");
            println!("\n\x1b[33m建议: 检查tag格式是否符合预期 (推荐格式: v1.2.3)\x1b[0m");
        }
        BoundaryWarning::TagMismatchPattern { tag, pattern } => {
            println!("\n\x1b[1m⚠️  Tag格式与配置不匹配\x1b[0m");
            println!("  你的输入:   {}", tag);
            println!("  配置pattern: {}", pattern);
            println!("\n\x1b[33m注: 这不会影响tag创建，但可能影响版本追踪\x1b[0m");
        }
        BoundaryWarning::FetchAuthenticationFailed { remote } => {
            println!("\n\x1b[1m⚠️  无法从远程获取最新数据\x1b[0m");
            println!("  远程: {} (SSH认证失败)", remote);
            println!("  将使用本地分支数据继续");
            println!("\n\x1b[33m注: 这可能导致使用过期的tag信息\x1b[0m");
        }
    }
}

/// 带原因的确认提示
pub fn confirm_action_with_reason(prompt: &str, reason: &str) -> Result<bool> {
    println!("\n{}", reason);
    confirm_action(prompt)
}

/// 验证tag格式是否匹配pattern
/// pattern: "v{version}" 或类似格式
/// tag: "v1.2.3" 或自定义值
pub fn validate_tag_format(tag: &str, pattern: &str) -> Result<()> {
    // 简单的格式检查：pattern中包含 {version}，tag应该是有效的
    // 如果pattern是 "v{version}"，则tag应该以"v"开头
    
    if pattern.contains("{version}") {
        let prefix = pattern.split("{version}").next().unwrap_or("");
        let suffix = pattern.rsplit("{version}").next().unwrap_or("");
        
        if !prefix.is_empty() && !tag.starts_with(prefix) {
            return Err(anyhow::anyhow!(
                "Tag does not start with expected prefix '{}'",
                prefix
            ));
        }
        if !suffix.is_empty() && !tag.ends_with(suffix) {
            return Err(anyhow::anyhow!(
                "Tag does not end with expected suffix '{}'",
                suffix
            ));
        }
    }
    
    Ok(())
}

/// 选择或自定义tag
/// 返回用户选择的tag值
pub fn select_or_customize_tag(recommended_tag: &str, pattern: &str) -> Result<String> {
    println!("\n\x1b[1m推荐的新tag: {}\x1b[0m", recommended_tag);
    println!("\n选项:");
    println!("  - 按Enter采用此tag");
    println!("  - 输入自定义tag值");
    println!("  - 输入 'e' 编辑推荐值");
    
    print!("\n你的选择 [{}]: ", recommended_tag);
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    
    // 按Enter采用默认
    if input.is_empty() {
        return Ok(recommended_tag.to_string());
    }
    
    // 输入'e'编辑
    if input == "e" || input == "edit" {
        print!("\n编辑模式 - 推荐值为 '{}'\n新的tag值 [{}]: ", 
               recommended_tag, recommended_tag);
        std::io::stdout().flush()?;
        
        let mut edited_input = String::new();
        std::io::stdin().read_line(&mut edited_input)?;
        let edited_input = edited_input.trim();
        
        if edited_input.is_empty() {
            return Ok(recommended_tag.to_string());
        }
        
        return Ok(edited_input.to_string());
    }
    
    // 自定义输入
    Ok(input.to_string())
}

/// 确认tag使用前检查格式
pub fn confirm_tag_use(tag: &str, pattern: &str) -> Result<bool> {
    // 先检查格式
    if let Err(e) = validate_tag_format(tag, pattern) {
        display_boundary_warning(&crate::boundary::BoundaryWarning::TagMismatchPattern {
            tag: tag.to_string(),
            pattern: pattern.to_string(),
        });
        
        println!("\n\x1b[31m格式错误: {}\x1b[0m", e);
        
        print!("\n仍然确认使用此tag? (y/N): ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        let response = input.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    } else {
        print!("\n确认tag: {}? (y/N): ", tag);
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        let response = input.trim().to_lowercase();
        Ok(response == "y" || response == "yes")
    }
}

/// 确认是否推送tag到远程
pub fn confirm_push_tag(tag: &str, remote: &str) -> Result<bool> {
    println!("\n\x1b[32m✓\x1b[0m 已创建本地tag: {}", tag);
    
    print!("\n将tag推送到远程 {}? (y/N): ", remote);
    std::io::stdout().flush()?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let response = input.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}

/// 显示手动推送提示
pub fn display_manual_push_instruction(tag: &str, remote: &str) {
    println!("\n\x1b[33mℹ️  tag已保留在本地\x1b[0m");
    println!("稍后可手动推送:");
    println!("  git push {} {}", remote, tag);
}
```

**Step 3: 运行测试确保没有编译错误**

```bash
cd /Users/a10121/github/git-publish-rust && cargo check 2>&1 | head -50
```

**Step 4: 更新lib.rs导出新的boundary模块**

在 `/Users/a10121/github/git-publish-rust/src/lib.rs` 添加：

```rust
pub mod boundary;
```

**Step 5: 编译并提交**

```bash
cd /Users/a10121/github/git-publish-rust && cargo build
git add -A
git commit -m "feat: add boundary warnings and UI functions for tag customization"
```

---

## Task 2: 集成git_ops获取当前HEAD哈希值

**Files:**
- Modify: `src/git_ops.rs` - 添加获取HEAD hash的函数

**Step 1: 在GitRepo实现中添加获取HEAD hash的方法**

在 `/Users/a10121/github/git-publish-rust/src/git_ops.rs` 中GitRepo impl块添加：

```rust
pub fn get_current_head_hash(&self) -> Result<String> {
    let head = self.repo.head()?;
    let oid = head.target()
        .ok_or_else(|| anyhow::anyhow!("HEAD is detached or invalid"))?;
    Ok(oid.to_string())
}
```

**Step 2: 编译测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo check
```

**Step 3: 提交**

```bash
git add src/git_ops.rs
git commit -m "feat: add get_current_head_hash method to GitRepo"
```

---

## Task 3: 改进main.rs主流程 - 边界检测

**Files:**
- Modify: `src/main.rs` - 改进边界检测逻辑

**Step 1: 添加use语句**

在 `/Users/a10121/github/git-publish-rust/src/main.rs` 顶部添加：

```rust
use git_publish::boundary::BoundaryWarning;
```

**Step 2: 改进"无新commits"的处理**

替换第134-140行的代码：

```rust
if commits.is_empty() {
    let head_hash = git_repo.get_current_head_hash()?;
    let warning = BoundaryWarning::NoNewCommits {
        latest_tag: latest_tag.clone().unwrap_or_else(|| "unknown".to_string()),
        current_commit_hash: head_hash,
    };
    
    ui::display_boundary_warning(&warning);
    
    if !args.force && !args.dry_run {
        if !ui::confirm_action_with_reason(
            "继续? (y/N): ",
            ""
        )? {
            println!("Operation cancelled by user.");
            return Ok(());
        }
    }
}
```

**Step 3: 改进无法解析tag的处理**

在第152-157行（计算新版本时）改进错误处理：

```rust
let new_version = match latest_tag.as_ref() {
    Some(tag) => {
        if let Some(current_version) = version::parse_version_from_tag(tag) {
            version::bump_version(current_version, &version_bump)
        } else {
            // 无法解析tag - 显示警告
            let warning = BoundaryWarning::UnparsableTag {
                tag: tag.clone(),
                reason: "Version number format not recognized".to_string(),
            };
            ui::display_boundary_warning(&warning);
            
            if !args.force && !args.dry_run {
                if !ui::confirm_action_with_reason(
                    "使用初始版本v0.1.0继续? (y/N): ",
                    ""
                )? {
                    println!("Operation cancelled by user.");
                    return Ok(());
                }
            }
            
            version::Version::new(0, 1, 0)
        }
    }
    None => version::Version::new(0, 1, 0),
};
```

**Step 4: 编译测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo check
```

**Step 5: 提交**

```bash
git add src/main.rs
git commit -m "feat: add boundary warnings for edge cases (no commits, unparsable tags)"
```

---

## Task 4: 实现Tag选项交互

**Files:**
- Modify: `src/main.rs` - 改进tag确认流程

**Step 1: 替换tag展示和确认逻辑**

在 `/Users/a10121/github/git-publish-rust/src/main.rs` 中，替换第173-180行：

```rust
// Display the proposed tag
ui::display_proposed_tag(latest_tag.as_deref(), &new_tag);

// Get user's tag selection (use default, customize, or edit)
let final_tag = if !args.force && !args.dry_run {
    ui::select_or_customize_tag(&new_tag, &new_tag_pattern)?
} else {
    new_tag.clone()
};

// Confirm tag use (checks format)
if !args.force && !args.dry_run {
    if !ui::confirm_tag_use(&final_tag, &new_tag_pattern)? {
        println!("Tag creation cancelled by user.");
        return Ok(());
    }
} else {
    // In dry-run or force mode, use the final tag
}
```

**Step 2: 编译测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo check
```

**Step 3: 提交**

```bash
git add src/main.rs
git commit -m "feat: add tag customization and format validation"
```

---

## Task 5: 分离Tag创建和Push逻辑

**Files:**
- Modify: `src/main.rs` - 改进tag创建后推送逻辑

**Step 1: 改进dry_run处理**

替换第182-190行：

```rust
if args.dry_run {
    ui::display_status("Dry run: Would create tag");
    ui::display_success(&format!("Would create tag: {}", final_tag));
    
    ui::display_status("Dry run: Would push tag to remote");
    ui::display_success(&format!("Would push tag: {} to remote", final_tag));
    return Ok(());
}
```

**Step 2: 改进tag创建和push分离逻辑**

替换第192-199行和后续的push逻辑，改为：

```rust
// Create the tag
ui::display_status(&format!("Creating tag: {}", final_tag));
if let Err(e) = git_repo.create_tag(&final_tag) {
    ui::display_error(&format!("Failed to create tag '{}': {}", final_tag, e));
    std::process::exit(1);
}
ui::display_success(&format!("Created tag: {}", final_tag));

// Ask user whether to push the tag
let should_push = if !args.force {
    ui::confirm_push_tag(&final_tag, "origin")?
} else {
    true // In force mode, push automatically
};

if should_push {
    // Push the tag to remote
    ui::display_status(&format!("Pushing tag: {} to remote", final_tag));
    if let Err(e) = git_repo.push_tag(&final_tag) {
        ui::display_error(&format!("Failed to push tag '{}': {}", final_tag, e));
        std::process::exit(1);
    }
    ui::display_success(&format!("Pushed tag: {} to remote", final_tag));
} else {
    // Tag created locally, but not pushed
    ui::display_manual_push_instruction(&final_tag, "origin");
}
```

**Step 3: 更新最终消息**

替换最后的println：

```rust
println!(
    "\n\x1b[32m✓\x1b[0m Successfully published tag {} for branch {}\n",
    final_tag, branch_to_tag
);
```

**Step 4: 编译测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo check
```

**Step 5: 提交**

```bash
git add src/main.rs
git commit -m "feat: separate tag creation and push logic with user confirmation"
```

---

## Task 6: 改进Fetch认证失败的处理

**Files:**
- Modify: `src/main.rs` - 改进fetch错误处理

**Step 1: 改进fetch时的错误处理**

替换第91-102行（fetch逻辑）：

```rust
// Fetch latest from remote to ensure we have the latest tags and commits
ui::display_status("Fetching latest data from remote...");
match git_repo.fetch_from_remote("origin") {
    Ok(_) => {
        ui::display_success("Successfully fetched latest data from remote");
    }
    Err(e) => {
        // Check if it's an authentication error
        let error_msg = e.to_string();
        if error_msg.contains("auth") || error_msg.contains("Auth") {
            let warning = BoundaryWarning::FetchAuthenticationFailed {
                remote: "origin".to_string(),
            };
            ui::display_boundary_warning(&warning);
            
            if !args.force && !args.dry_run {
                if !ui::confirm_action_with_reason(
                    "继续使用本地数据? (y/N): ",
                    ""
                )? {
                    println!("Operation cancelled by user.");
                    return Ok(());
                }
            }
        } else {
            // Non-auth errors are still warnings
            ui::display_status(&format!(
                "Warning: Could not fetch from remote: {}. Using local branch data.",
                e
            ));
        }
    }
}
```

**Step 2: 编译测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo check
```

**Step 3: 提交**

```bash
git add src/main.rs
git commit -m "feat: improve fetch authentication error handling with boundary warning"
```

---

## Task 7: 添加集成测试

**Files:**
- Modify: `tests/integration_test.rs` - 添加新的测试

**Step 1: 添加边界情况测试**

在 `/Users/a10121/github/git-publish-rust/tests/integration_test.rs` 末尾添加：

```rust
#[test]
fn test_boundary_warning_no_new_commits() {
    use git_publish::boundary::BoundaryWarning;
    
    let warning = BoundaryWarning::NoNewCommits {
        latest_tag: "v1.0.0".to_string(),
        current_commit_hash: "abc1234".to_string(),
    };
    
    assert!(warning.to_string().contains("No new commits"));
}

#[test]
fn test_boundary_warning_unparsable_tag() {
    use git_publish::boundary::BoundaryWarning;
    
    let warning = BoundaryWarning::UnparsableTag {
        tag: "release-123".to_string(),
        reason: "Invalid format".to_string(),
    };
    
    assert!(warning.to_string().contains("Cannot parse tag"));
}

#[test]
fn test_tag_format_validation() {
    use git_publish::ui;
    
    // Valid format
    assert!(ui::validate_tag_format("v1.2.3", "v{version}").is_ok());
    
    // Invalid format - missing prefix
    assert!(ui::validate_tag_format("1.2.3", "v{version}").is_err());
}
```

**Step 2: 运行测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo test --test integration_test 2>&1 | tail -30
```

**Step 3: 提交**

```bash
git add tests/integration_test.rs
git commit -m "test: add integration tests for boundary warnings and tag validation"
```

---

## Task 8: 最终验证和代码质量检查

**Files:**
- All modified files

**Step 1: 运行所有测试**

```bash
cd /Users/a10121/github/git-publish-rust && cargo test
```

**Step 2: 运行clippy检查**

```bash
cd /Users/a10121/github/git-publish-rust && cargo clippy -- -D warnings
```

**Step 3: 代码格式化**

```bash
cd /Users/a10121/github/git-publish-rust && cargo fmt
```

**Step 4: 最后编译检查**

```bash
cd /Users/a10121/github/git-publish-rust && cargo build --release
```

**Step 5: 手动测试整个流程（可选）**

```bash
cd /Users/a10121/github/git-publish-rust && ./target/debug/git-publish --help
```

**Step 6: 提交最后的cleanup**

```bash
git add -A
git commit -m "refactor: code cleanup and formatting"
```

---

## 验收标准

- [ ] 所有测试通过 (`cargo test`)
- [ ] clippy无警告 (`cargo clippy -- -D warnings`)
- [ ] 代码格式正确 (`cargo fmt`)
- [ ] 所有边界情况有清晰的提示
- [ ] Tag生成和push逻辑分离
- [ ] 用户可以自定义tag（默认采用）
- [ ] Tag格式校验有效
- [ ] Dry-run模式工作正常
- [ ] 手动测试流程完整可用

