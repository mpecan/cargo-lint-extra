#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_verbose_result_handling_detected() {
    let mut config = Config::default();
    config.rules.verbose_result_handling.level = RuleLevel::Warn;
    let diags = test_helpers::run_on_fixture("verbose_result_handling.rs", &config);
    let rule_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "verbose-result-handling")
        .collect();
    assert_eq!(
        rule_diags.len(),
        6,
        "expected 6 verbose-result-handling diagnostics, got {}: {rule_diags:?}",
        rule_diags.len()
    );
}

#[test]
fn test_verbose_result_handling_disabled() {
    let mut config = Config::default();
    config.rules.verbose_result_handling.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("verbose_result_handling.rs", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "verbose-result-handling"),
        "Allow level should produce no diagnostics"
    );
}
