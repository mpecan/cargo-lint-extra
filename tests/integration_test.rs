#![allow(clippy::unwrap_used)]

use cargo_lint_extra::config::{
    Config, FileLengthConfig, LineLengthOverride, TestConfig, TestRulesOverrides,
};
use cargo_lint_extra::diagnostic::RuleLevel;
use cargo_lint_extra::engine::Engine;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn run_on_fixture_dir(
    fixture_dir: &str,
    test_name: &str,
    config: &Config,
) -> Vec<cargo_lint_extra::diagnostic::Diagnostic> {
    let engine = Engine::new(config);
    let path = fixture_path(fixture_dir);
    let tmp = std::env::temp_dir().join(format!("cargo-lint-extra-int-{test_name}"));
    let _ = std::fs::remove_dir_all(&tmp);
    copy_dir_recursive(&path, &tmp);
    let result = engine.run(&tmp);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            std::fs::copy(&src_path, &dst_path).unwrap();
        }
    }
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

// --- File-length soft/hard limit integration tests ---

#[test]
fn test_file_length_soft_limit_warns() {
    let mut config = Config::default();
    config.rules.file_length = FileLengthConfig {
        soft_limit: 4,
        hard_limit: 10,
        ..FileLengthConfig::default()
    };
    // clean.rs has 5 lines, exceeding the soft limit of 4
    let diags = run_on_fixture("clean.rs", "file-length-soft", &config);
    let fl: Vec<_> = diags.iter().filter(|d| d.rule == "file-length").collect();
    assert_eq!(fl.len(), 1, "expected 1 file-length diagnostic, got {fl:?}");
    assert_eq!(fl[0].level, RuleLevel::Warn);
    assert!(fl[0].message.contains("soft limit"));
}

#[test]
fn test_file_length_hard_limit_denies() {
    let mut config = Config::default();
    config.rules.file_length = FileLengthConfig {
        soft_limit: 2,
        hard_limit: 3,
        ..FileLengthConfig::default()
    };
    let diags = run_on_fixture("clean.rs", "file-length-hard", &config);
    let fl: Vec<_> = diags.iter().filter(|d| d.rule == "file-length").collect();
    assert_eq!(fl.len(), 1, "expected 1 file-length diagnostic, got {fl:?}");
    assert_eq!(fl[0].level, RuleLevel::Deny);
    assert!(fl[0].message.contains("hard limit"));
}

// --- Test override integration tests --- // cargo-lint-extra:allow(redundant-comments)

fn config_with_test_line_length_override(soft_limit: usize) -> Config {
    Config {
        test: Some(TestConfig {
            rules: TestRulesOverrides {
                line_length: Some(LineLengthOverride {
                    soft_limit: Some(soft_limit),
                    ..LineLengthOverride::default()
                }),
                ..TestRulesOverrides::default()
            },
            ..TestConfig::default()
        }),
        ..Config::default()
    }
}

#[test]
fn test_override_test_file_uses_test_rules() {
    // With a high soft_limit for tests, test file should have no line-length diagnostics
    let config = config_with_test_line_length_override(200);
    let diags = run_on_fixture_dir("test_override", "override-test-file", &config);
    let test_file_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("tests/"))
        .filter(|d| d.rule == "line-length")
        .collect();
    assert!(
        test_file_diags.is_empty(),
        "test file should have no line-length diagnostics with relaxed limit, got: {test_file_diags:?}"
    );
}

#[test]
fn test_override_prod_file_still_flagged() {
    // Prod code should still be flagged even with relaxed test limit
    let config = config_with_test_line_length_override(200);
    let diags = run_on_fixture_dir("test_override", "override-prod-flagged", &config);
    let has_prod_diags = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("src/"))
        .any(|d| d.rule == "line-length");
    assert!(
        has_prod_diags,
        "production code should still have line-length diagnostics"
    );
}

#[test]
fn test_override_cfg_test_block_uses_test_rules() {
    // In a mixed file (src/main.rs with #[cfg(test)]), the #[cfg(test)] block
    // should use test rules (relaxed), while prod code uses prod rules.
    let config = config_with_test_line_length_override(200);
    let diags = run_on_fixture_dir("test_override", "override-cfg-test", &config);

    let src_main_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("src/main.rs"))
        .filter(|d| d.rule == "line-length")
        .collect();

    // Only the prod line should be flagged (the one in production_function)
    // The line inside #[cfg(test)] mod tests should NOT be flagged
    assert_eq!(
        src_main_diags.len(),
        1,
        "expected 1 line-length diagnostic in src/main.rs (prod only), got {}: {src_main_diags:?}",
        src_main_diags.len()
    );
    // Verify it's on a prod line (line 6 is the long line in production_function)
    assert!(
        src_main_diags[0].line.is_some_and(|l| l < 9),
        "diagnostic should be on a production code line"
    );
}

#[test]
fn test_override_no_test_config_no_split() {
    // Without test config, all files are treated uniformly
    let config = Config::default();
    let diags = run_on_fixture_dir("test_override", "override-no-config", &config);
    let all_line_length: Vec<_> = diags.iter().filter(|d| d.rule == "line-length").collect();
    // Both prod and test lines should be flagged with default rules
    assert!(
        all_line_length.len() >= 2,
        "without test config, all long lines should be flagged, got {}: {all_line_length:?}",
        all_line_length.len()
    );
}

#[test]
fn test_override_suffix_pattern() {
    // Test with a custom suffix pattern *_test.rs
    let config = Config {
        test: Some(TestConfig {
            patterns: vec!["*_test.rs".to_string()],
            detect_cfg_test: false,
            rules: TestRulesOverrides {
                line_length: Some(LineLengthOverride {
                    soft_limit: Some(200),
                    ..LineLengthOverride::default()
                }),
                ..TestRulesOverrides::default()
            },
        }),
        ..Config::default()
    };
    let diags = run_on_fixture_dir("test_override", "override-suffix", &config);
    // test_main.rs doesn't match *_test.rs (it's test_main, not main_test)
    // so it should be flagged with prod rules
    let has_test_file_diags = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("test_main.rs"))
        .any(|d| d.rule == "line-length");
    assert!(
        has_test_file_diags,
        "test_main.rs should be flagged since it doesn't match *_test.rs pattern"
    );
}

// --- Redundant comments integration tests --- // cargo-lint-extra:allow(redundant-comments)

#[test]
fn test_redundant_comments_detected() {
    let config = Config::default();
    let diags = run_on_fixture("redundant_comments.rs", "redundant-comments", &config);
    let rc: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "redundant-comments")
        .collect();
    // 4 redundant comments: increment counter, return result, create vector, set name
    assert_eq!(
        rc.len(),
        4,
        "expected 4 redundant-comments diagnostics, got {}: {rc:?}",
        rc.len()
    );
}

#[test]
fn test_redundant_comments_disabled() {
    let mut config = Config::default();
    config.rules.redundant_comments.level = RuleLevel::Allow;
    let diags = run_on_fixture("redundant_comments.rs", "redundant-disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "redundant-comments"),
        "disabled redundant-comments rule should produce no diagnostics"
    );
}

#[test]
fn test_redundant_comments_suppression() {
    let config = Config::default();
    let diags = run_on_fixture("redundant_comments.rs", "redundant-suppression", &config);
    // The "return the value" comment at line 30 is suppressed via cargo-lint-extra:allow
    let rc: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "redundant-comments")
        .collect();
    // Without suppression there would be 5 diagnostics; the suppressed one reduces it to 4
    assert_eq!(
        rc.len(),
        4,
        "expected 4 redundant-comments diagnostics (1 suppressed), got {}: {rc:?}",
        rc.len()
    );
    assert!(
        !rc.iter().any(|d| d.line == Some(30)),
        "suppressed redundant comment at line 30 should not be flagged"
    );
}
