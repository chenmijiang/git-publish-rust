# Tag Selection Choice UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the single free-text tag prompt with a choice-based selector for parseable tags while preserving the existing custom-input flow for unparsable tags.

**Architecture:** Add a small domain helper to generate ordered bump candidates from the current version, then teach the UI layer to present numbered tag choices with a custom fallback. Keep the unparsable-tag path on the existing free-text prompt so the fallback behavior remains unchanged.

**Tech Stack:** Rust 2021, `clap`, `anyhow`, `git2`, existing `ui` prompts.

---

### Task 1: Generate ordered bump candidates

**Files:**
- Modify: `src/domain/version.rs:98-120`
- Test: `src/domain/version.rs:225-255`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn test_version_bump_options_major() {
    let version = Version::new(0, 1, 0);
    let options = version.bump_options(VersionBump::Major);

    assert_eq!(options, vec![Version::new(1, 0, 0), Version::new(0, 2, 0), Version::new(0, 1, 1)]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib test_version_bump_options_major -- --exact`
Expected: FAIL because `bump_options` does not exist yet.

- [ ] **Step 3: Write minimal implementation**

```rust
pub fn bump_options(&self, bump_type: &VersionBump) -> Vec<Self> {
    match bump_type {
        VersionBump::Major => vec![self.bump(&VersionBump::Major), self.bump(&VersionBump::Minor), self.bump(&VersionBump::Patch)],
        VersionBump::Minor => vec![self.bump(&VersionBump::Minor), self.bump(&VersionBump::Patch)],
        VersionBump::Patch => vec![self.bump(&VersionBump::Patch)],
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib test_version_bump_options_major -- --exact`
Expected: PASS

### Task 2: Add choice-based tag selection UI

**Files:**
- Modify: `src/ui/mod.rs:208-324`
- Test: `src/ui/mod.rs:326-362`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn test_validate_tag_format_simple() {
    assert!(validate_tag_format("v1.2.3", "v{version}").is_ok());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib test_validate_tag_format_simple -- --exact`
Expected: PASS for the existing test harness, then add a new unit test for a pure helper if one is introduced.

- [ ] **Step 3: Write minimal implementation**

```rust
pub fn select_tag_from_candidates(recommended_tag: &str, candidate_tags: &[String]) -> Result<String> {
    // print numbered options, accept default 1, or "c" for custom input
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib`
Expected: PASS

### Task 3: Route parseable and unparsable paths differently

**Files:**
- Modify: `src/main.rs:257-314`

- [ ] **Step 1: Write the failing integration scenario**

```rust
// Parseable tags should use the choice menu; unparsable tags should keep free-text customization.
```

- [ ] **Step 2: Run the relevant tests**

Run: `cargo test --lib && cargo test --test integration_test`
Expected: existing tests pass before changing the flow.

- [ ] **Step 3: Update the tag flow**

```rust
// Parseable: build ordered candidate tags, show choices, then confirm.
// Unparsable or missing tag: keep the current select_or_customize_tag path.
```

- [ ] **Step 4: Run the validation suite**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test --lib && cargo build`
Expected: all commands pass.

