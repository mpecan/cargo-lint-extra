#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;

#[test]
fn test_suppression_inline_line_length() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("suppressed.rs", &config);
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
    let diags = test_helpers::run_on_fixture("suppressed.rs", &config);
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
    let diags = test_helpers::run_on_fixture("suppressed.rs", &config);
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
