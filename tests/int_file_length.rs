#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::{Config, FileLengthConfig};
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_file_length_soft_limit_warns() {
    let mut config = Config::default();
    config.rules.file_length = FileLengthConfig {
        soft_limit: 4,
        hard_limit: 10,
        ..FileLengthConfig::default()
    };
    // clean.rs has 5 lines, exceeding the soft limit of 4
    let diags = test_helpers::run_on_fixture("clean.rs", "file-length-soft", &config);
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
    let diags = test_helpers::run_on_fixture("clean.rs", "file-length-hard", &config);
    let fl: Vec<_> = diags.iter().filter(|d| d.rule == "file-length").collect();
    assert_eq!(fl.len(), 1, "expected 1 file-length diagnostic, got {fl:?}");
    assert_eq!(fl[0].level, RuleLevel::Deny);
    assert!(fl[0].message.contains("hard limit"));
}
