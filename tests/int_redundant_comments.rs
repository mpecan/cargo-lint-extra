#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

// cargo-lint-extra:allow(redundant-comments)

#[test]
fn test_redundant_comments_detected() {
    let config = Config::default();
    let diags =
        test_helpers::run_on_fixture("redundant_comments.rs", "redundant-comments", &config);
    let rc: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "redundant-comments")
        .collect();
    // 4 redundant comments: increment counter, return result, create vector, set name
    assert_eq!(
        rc.len(),
        4,
        "expected 4 redundant-comments diagnostics, got {}: {rc:?}",
        rc.len()
    );
}

#[test]
fn test_redundant_comments_disabled() {
    let mut config = Config::default();
    config.rules.redundant_comments.level = RuleLevel::Allow;
    let diags =
        test_helpers::run_on_fixture("redundant_comments.rs", "redundant-disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "redundant-comments"),
        "disabled redundant-comments rule should produce no diagnostics"
    );
}

#[test]
fn test_redundant_comments_suppression() {
    let config = Config::default();
    let diags =
        test_helpers::run_on_fixture("redundant_comments.rs", "redundant-suppression", &config);
    // The "return the value" comment at line 30 is suppressed via cargo-lint-extra:allow
    let rc: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "redundant-comments")
        .collect();
    // Without suppression there would be 5 diagnostics; the suppressed one reduces it to 4
    assert_eq!(
        rc.len(),
        4,
        "expected 4 redundant-comments diagnostics (1 suppressed), got {}: {rc:?}",
        rc.len()
    );
    assert!(
        !rc.iter().any(|d| d.line == Some(30)),
        "suppressed redundant comment at line 30 should not be flagged"
    );
}
