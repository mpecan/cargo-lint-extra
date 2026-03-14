#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::{Config, GlobImportsConfig};
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_glob_imports_detected() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("glob_imports.rs", &config);
    let glob_diags: Vec<_> = diags.iter().filter(|d| d.rule == "glob-imports").collect();
    // 2 glob imports: std::collections::* and std::io::* (std::path::* is suppressed)
    assert_eq!(
        glob_diags.len(),
        2,
        "expected 2 glob-imports diagnostics, got {}: {glob_diags:?}",
        glob_diags.len()
    );
}

#[test]
fn test_glob_imports_disabled() {
    let mut config = Config::default();
    config.rules.glob_imports.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("glob_imports.rs", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "glob-imports"),
        "disabled glob-imports rule should produce no diagnostics"
    );
}

#[test]
fn test_glob_imports_with_allowlist() {
    let mut config = Config::default();
    config.rules.glob_imports = GlobImportsConfig {
        level: RuleLevel::Warn,
        allowed_crates: vec!["std::collections".to_string(), "std::io".to_string()],
        allow_in_tests: true,
    };
    let diags = test_helpers::run_on_fixture("glob_imports.rs", &config);
    let glob_diags: Vec<_> = diags.iter().filter(|d| d.rule == "glob-imports").collect();
    // Both std::collections::* and std::io::* are allowed; std::path::* is suppressed
    assert!(
        glob_diags.is_empty(),
        "allowlisted glob imports should produce no diagnostics, got: {glob_diags:?}"
    );
}
