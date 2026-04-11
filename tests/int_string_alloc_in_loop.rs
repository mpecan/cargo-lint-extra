#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_string_alloc_in_loop_detected() {
    let mut config = Config::default();
    config.rules.string_alloc_in_loop.level = RuleLevel::Warn;
    let diags = test_helpers::run_on_fixture("string_alloc_in_loop.rs", &config);
    let rule_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "string-alloc-in-loop")
        .collect();
    assert_eq!(
        rule_diags.len(),
        7,
        "expected 7 string-alloc-in-loop diagnostics, got {}: {rule_diags:?}",
        rule_diags.len()
    );
}

#[test]
fn test_string_alloc_in_loop_disabled() {
    let mut config = Config::default();
    config.rules.string_alloc_in_loop.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("string_alloc_in_loop.rs", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "string-alloc-in-loop"),
        "Allow level should produce no diagnostics"
    );
}
