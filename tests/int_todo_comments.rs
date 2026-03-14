#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;

#[test]
fn test_todo_comments_detected() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("todos.rs", &config);
    assert_eq!(
        diags.iter().filter(|d| d.rule == "todo-comments").count(),
        3,
        "expected 3 todo comment diagnostics"
    );
}
