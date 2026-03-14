#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::{Config, UndocumentedPanicConfig};
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_undocumented_panic_disabled_by_default() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("undocumented_panic.rs", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "undocumented-panic"),
        "undocumented-panic should be Allow by default, got: {diags:?}"
    );
}

#[test]
fn test_undocumented_panic_detected() {
    let mut config = Config::default();
    config.rules.undocumented_panic = UndocumentedPanicConfig {
        level: RuleLevel::Warn,
        check_unwrap: true,
        check_expect: true,
        check_indexing: false,
        required_comment: "PANIC".to_string(),
    };
    let diags = test_helpers::run_on_fixture("undocumented_panic.rs", &config);
    let panic_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "undocumented-panic")
        .collect();
    assert_eq!(
        panic_diags.len(),
        2,
        "expected 2 undocumented-panic diagnostics (bad_unwrap + bad_expect), got {}: {panic_diags:?}",
        panic_diags.len()
    );
    assert!(
        panic_diags.iter().any(|d| d.message.contains(".unwrap()")),
        "expected unwrap diagnostic, got: {panic_diags:?}"
    );
    assert!(
        panic_diags.iter().any(|d| d.message.contains(".expect()")),
        "expected expect diagnostic, got: {panic_diags:?}"
    );
}

#[test]
fn test_undocumented_panic_indexing_enabled() {
    let mut config = Config::default();
    config.rules.undocumented_panic = UndocumentedPanicConfig {
        level: RuleLevel::Warn,
        check_unwrap: true,
        check_expect: true,
        check_indexing: true,
        required_comment: "PANIC".to_string(),
    };
    let diags = test_helpers::run_on_fixture("undocumented_panic.rs", &config);
    let panic_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "undocumented-panic")
        .collect();
    assert_eq!(
        panic_diags.len(),
        3,
        "expected 3 undocumented-panic diagnostics (bad_unwrap + bad_expect + bad_indexing), got {}: {panic_diags:?}",
        panic_diags.len()
    );
    assert!(
        panic_diags.iter().any(|d| d.message.contains("indexing")),
        "expected indexing diagnostic, got: {panic_diags:?}"
    );
}
