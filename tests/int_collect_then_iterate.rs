#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_collect_then_iterate_detected() {
    let mut config = Config::default();
    config.rules.collect_then_iterate.level = RuleLevel::Warn;
    let diags = test_helpers::run_on_fixture("collect_then_iterate.rs", &config);
    let rule_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "collect-then-iterate")
        .collect();
    assert_eq!(
        rule_diags.len(),
        7,
        "expected 7 collect-then-iterate diagnostics, got {}: {rule_diags:?}",
        rule_diags.len()
    );
}

#[test]
fn test_collect_then_iterate_disabled() {
    let mut config = Config::default();
    config.rules.collect_then_iterate.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("collect_then_iterate.rs", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "collect-then-iterate"),
        "Allow level should produce no diagnostics"
    );
}
