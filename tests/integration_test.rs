#![allow(clippy::unwrap_used)]

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;
use cargo_lint_extra::engine::Engine;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn run_on_fixture(
    name: &str,
    test_name: &str,
    config: &Config,
) -> Vec<cargo_lint_extra::diagnostic::Diagnostic> {
    let engine = Engine::new(config);
    let path = fixture_path(name);
    let tmp = std::env::temp_dir().join(format!("cargo-lint-extra-int-{test_name}"));
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::copy(&path, tmp.join(name)).unwrap();
    let result = engine.run(&tmp);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

#[test]
fn test_clean_file_produces_no_diagnostics() {
    let config = Config::default();
    let diags = run_on_fixture("clean.rs", "clean", &config);
    assert!(
        diags.is_empty(),
        "clean file should have no diagnostics, got: {diags:?}"
    );
}

#[test]
fn test_long_lines_detected() {
    let config = Config::default();
    let diags = run_on_fixture("long_lines.rs", "long-lines", &config);
    assert_eq!(
        diags.iter().filter(|d| d.rule == "line-length").count(),
        1,
        "expected 1 long line diagnostic"
    );
}

#[test]
fn test_todo_comments_detected() {
    let config = Config::default();
    let diags = run_on_fixture("todos.rs", "todos", &config);
    assert_eq!(
        diags.iter().filter(|d| d.rule == "todo-comments").count(),
        3,
        "expected 3 todo comment diagnostics"
    );
}

#[test]
fn test_allow_audit_detected() {
    let mut config = Config::default();
    config.rules.allow_audit.level = RuleLevel::Warn;
    let diags = run_on_fixture("allow_attrs.rs", "allow-audit", &config);
    assert_eq!(
        diags.iter().filter(|d| d.rule == "allow-audit").count(),
        2,
        "expected 2 allow-audit diagnostics"
    );
}

#[test]
fn test_inline_comments_detected() {
    let config = Config::default();
    let diags = run_on_fixture("inline_comments.rs", "inline-comments", &config);
    let inline: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "inline-comments")
        .collect();
    // over_commented: ratio (60%) + consecutive (6)
    // consecutive_block: ratio (44%) + consecutive (4)
    // clean_function: no diagnostics
    assert_eq!(
        inline.len(),
        4,
        "expected 4 inline-comments diagnostics (2 ratio + 2 consecutive), got {}: {inline:?}",
        inline.len()
    );
    assert_eq!(
        inline
            .iter()
            .filter(|d| d.message.contains("inline comments"))
            .count(),
        2,
        "expected 2 ratio diagnostics"
    );
    assert_eq!(
        inline
            .iter()
            .filter(|d| d.message.contains("consecutive"))
            .count(),
        2,
        "expected 2 consecutive diagnostics"
    );
}

#[test]
fn test_inline_comments_disabled() {
    let mut config = Config::default();
    config.rules.inline_comments.level = RuleLevel::Allow;
    let diags = run_on_fixture("inline_comments.rs", "inline-disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "inline-comments"),
        "disabled inline-comments rule should produce no diagnostics"
    );
}

#[test]
fn test_disabled_rule_produces_no_diagnostics() {
    let mut config = Config::default();
    config.rules.line_length.level = RuleLevel::Allow;
    let diags = run_on_fixture("long_lines.rs", "disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "line-length"),
        "disabled rule should produce no diagnostics"
    );
}

#[test]
fn test_suppression_inline_line_length() {
    let config = Config::default();
    let diags = run_on_fixture("suppressed.rs", "suppressed-inline", &config);
    // The inline-suppressed long line should not be flagged
    let line_length_diags: Vec<_> = diags.iter().filter(|d| d.rule == "line-length").collect();
    // Only the unsuppressed long line (in unsuppressed fn) should be flagged
    assert_eq!(
        line_length_diags.len(),
        1,
        "expected 1 unsuppressed line-length diagnostic, got {}: {line_length_diags:?}",
        line_length_diags.len()
    );
}

#[test]
fn test_suppression_next_line_todo() {
    let config = Config::default();
    let diags = run_on_fixture("suppressed.rs", "suppressed-todo", &config);
    let todo_diags: Vec<_> = diags.iter().filter(|d| d.rule == "todo-comments").collect();
    // Only the unsuppressed item should be flagged
    assert_eq!(
        todo_diags.len(),
        1,
        "expected 1 unsuppressed todo-comments diagnostic, got {}: {todo_diags:?}",
        todo_diags.len()
    );
}

#[test]
fn test_suppression_block_covers_function() {
    let config = Config::default();
    let diags = run_on_fixture("suppressed.rs", "suppressed-block", &config);
    // The long line inside suppressed_function should not be flagged
    let line_length_diags: Vec<_> = diags.iter().filter(|d| d.rule == "line-length").collect();
    // Only the one in unsuppressed() should remain
    assert_eq!(
        line_length_diags.len(),
        1,
        "expected only 1 line-length diagnostic (unsuppressed fn), got {}: {line_length_diags:?}",
        line_length_diags.len()
    );
}
