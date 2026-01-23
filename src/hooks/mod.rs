//! Git hooks system for extensibility
//!
//! Allows users to run custom scripts at key workflow points:
//! - pre-tag-create: Before tag creation
//! - post-tag-create: After tag created locally
//! - post-push: After tag pushed to remote

pub mod executor;
pub mod lifecycle;

pub use executor::HookExecutor;
pub use lifecycle::{HookContext, HookType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_module_exports() {
        // Verify public API is accessible
        let _ = HookType::PreTagCreate;
    }
}
