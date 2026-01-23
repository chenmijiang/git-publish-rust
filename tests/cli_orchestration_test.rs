use git_publish::cli::orchestration::{PublishWorkflowArgs, WorkflowResult};
use std::collections::HashMap;

#[test]
fn test_orchestration_module_exports() {
    // This test verifies the module structure exists and can be imported
    // It won't execute the workflow (would need full git repo setup)
    // Just verifies the types exist and are importable

    // If this compiles, the module structure is correct
    let _type_check = || {
        let _args: Option<PublishWorkflowArgs> = None;
        let _result: Option<()> = None;
    };
}

#[test]
fn test_workflow_result_structure() {
    // Test that WorkflowResult can be created with expected fields
    let result = WorkflowResult {
        tag: "v1.2.3".to_string(),
        branch: "main".to_string(),
        pushed: true,
    };

    assert_eq!(result.tag, "v1.2.3");
    assert_eq!(result.branch, "main");
    assert_eq!(result.pushed, true);
}

#[test]
fn test_publish_workflow_args_structure() {
    // Test that PublishWorkflowArgs contains expected configuration
    let args = PublishWorkflowArgs {
        config_path: None,
        branch: Some("main".to_string()),
        remote: Some("origin".to_string()),
        force: false,
        dry_run: false,
    };

    assert_eq!(args.branch, Some("main".to_string()));
    assert_eq!(args.remote, Some("origin".to_string()));
    assert_eq!(args.force, false);
    assert_eq!(args.dry_run, false);
}

#[test]
fn test_publish_workflow_args_all_none() {
    // Test that PublishWorkflowArgs can have all options as None
    let args = PublishWorkflowArgs {
        config_path: None,
        branch: None,
        remote: None,
        force: false,
        dry_run: false,
    };

    assert_eq!(args.config_path, None);
    assert_eq!(args.branch, None);
    assert_eq!(args.remote, None);
}

#[test]
fn test_publish_workflow_args_force_and_dry_run() {
    // Test that force and dry_run flags work correctly
    let args = PublishWorkflowArgs {
        config_path: Some("/path/to/config".to_string()),
        branch: Some("develop".to_string()),
        remote: Some("upstream".to_string()),
        force: true,
        dry_run: true,
    };

    assert_eq!(args.force, true);
    assert_eq!(args.dry_run, true);
    assert_eq!(args.config_path, Some("/path/to/config".to_string()));
}

#[test]
fn test_workflow_result_not_pushed() {
    // Test WorkflowResult when tag was not pushed
    let result = WorkflowResult {
        tag: "v2.0.0".to_string(),
        branch: "develop".to_string(),
        pushed: false,
    };

    assert_eq!(result.tag, "v2.0.0");
    assert_eq!(result.pushed, false);
}

#[test]
fn test_branch_selection_with_explicit_branch() {
    // When branch is specified in args, it should be returned
    let result = git_publish::cli::orchestration::select_branch_for_workflow(
        Some("main".to_string()),
        &HashMap::from([
            ("main".to_string(), "v{version}".to_string()),
            ("develop".to_string(), "release-{version}".to_string()),
        ]),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "main".to_string());
}

#[test]
fn test_branch_selection_validation() {
    // When branch is specified but not in config, should error
    let result = git_publish::cli::orchestration::select_branch_for_workflow(
        Some("invalid".to_string()),
        &HashMap::from([("main".to_string(), "v{version}".to_string())]),
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not configured"));
}
