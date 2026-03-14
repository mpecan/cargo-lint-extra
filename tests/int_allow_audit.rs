#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_allow_audit_detected() {
    let mut config = Config::default();
    config.rules.allow_audit.level = RuleLevel::Warn;
    let diags = test_helpers::run_on_fixture("allow_attrs.rs", &config);
    assert_eq!(
        diags.iter().filter(|d| d.rule == "allow-audit").count(),
        2,
        "expected 2 allow-audit diagnostics"
    );
}
