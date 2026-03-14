#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_long_lines_detected() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("long_lines.rs", "long-lines", &config);
    assert_eq!(
        diags.iter().filter(|d| d.rule == "line-length").count(),
        1,
        "expected 1 long line diagnostic"
    );
}

#[test]
fn test_disabled_rule_produces_no_diagnostics() {
    let mut config = Config::default();
    config.rules.line_length.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("long_lines.rs", "disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "line-length"),
        "disabled rule should produce no diagnostics"
    );
}
