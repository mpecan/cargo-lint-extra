#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::{Config, MagicNumbersConfig};
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_magic_numbers_detected() {
    let mut config = Config::default();
    config.rules.magic_numbers = MagicNumbersConfig {
        level: RuleLevel::Warn,
        ..MagicNumbersConfig::default()
    };
    let diags = test_helpers::run_on_fixture("magic_numbers.rs", &config);
    let magic_diags: Vec<_> = diags.iter().filter(|d| d.rule == "magic-numbers").collect();
    // 4 flagged: 42, 3.14, 255, 60 — const/static/enum/range/test are skipped by default
    assert_eq!(
        magic_diags.len(),
        4,
        "expected 4 magic-numbers diagnostics (42, 3.14, 255, 60), got {}: {magic_diags:?}",
        magic_diags.len()
    );
    assert!(magic_diags.iter().any(|d| d.message.contains("42")));
    assert!(magic_diags.iter().any(|d| d.message.contains("3.14")));
    assert!(magic_diags.iter().any(|d| d.message.contains("255")));
    assert!(magic_diags.iter().any(|d| d.message.contains("60")));
}

#[test]
fn test_magic_numbers_disabled_by_default() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("magic_numbers.rs", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "magic-numbers"),
        "default level is Allow, should produce no diagnostics"
    );
}

#[test]
fn test_magic_numbers_custom_allowed() {
    let mut config = Config::default();
    let mut allowed = MagicNumbersConfig::default().allowed;
    allowed.push("42".to_string());
    config.rules.magic_numbers = MagicNumbersConfig {
        level: RuleLevel::Warn,
        allowed,
        ..MagicNumbersConfig::default()
    };
    let diags = test_helpers::run_on_fixture("magic_numbers.rs", &config);
    let magic_diags: Vec<_> = diags.iter().filter(|d| d.rule == "magic-numbers").collect();
    // 42 is now allowed, so 3 remaining: 3.14, 255, 60
    assert_eq!(
        magic_diags.len(),
        3,
        "expected 3 magic-numbers diagnostics (3.14, 255, 60), got {}: {magic_diags:?}",
        magic_diags.len()
    );
    assert!(!magic_diags.iter().any(|d| d.message.contains("`42`")));
}
