#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_inline_comments_detected() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("inline_comments.rs", "inline-comments", &config);
    let inline: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "inline-comments")
        .collect();
    // over_commented: ratio (60%) + consecutive (6)
    // consecutive_block: ratio (44%) + consecutive (4)
    // clean_function: no diagnostics
    assert_eq!(
        inline.len(),
        4,
        "expected 4 inline-comments diagnostics (2 ratio + 2 consecutive), got {}: {inline:?}",
        inline.len()
    );
    assert_eq!(
        inline
            .iter()
            .filter(|d| d.message.contains("inline comments"))
            .count(),
        2,
        "expected 2 ratio diagnostics"
    );
    assert_eq!(
        inline
            .iter()
            .filter(|d| d.message.contains("consecutive"))
            .count(),
        2,
        "expected 2 consecutive diagnostics"
    );
}

#[test]
fn test_inline_comments_disabled() {
    let mut config = Config::default();
    config.rules.inline_comments.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("inline_comments.rs", "inline-disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "inline-comments"),
        "disabled inline-comments rule should produce no diagnostics"
    );
}
