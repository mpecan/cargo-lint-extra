#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::{Config, LineLengthOverride, TestConfig, TestRulesOverrides};

// cargo-lint-extra:allow(redundant-comments)

fn config_with_test_line_length_override(soft_limit: usize) -> Config {
    Config {
        test: Some(TestConfig {
            rules: TestRulesOverrides {
                line_length: Some(LineLengthOverride {
                    soft_limit: Some(soft_limit),
                    ..LineLengthOverride::default()
                }),
                ..TestRulesOverrides::default()
            },
            ..TestConfig::default()
        }),
        ..Config::default()
    }
}

#[test]
fn test_override_test_file_uses_test_rules() {
    // With a high soft_limit for tests, test file should have no line-length diagnostics
    let config = config_with_test_line_length_override(200);
    let diags = test_helpers::run_on_fixture_dir("test_override", &config);
    let test_file_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("tests/"))
        .filter(|d| d.rule == "line-length")
        .collect();
    assert!(
        test_file_diags.is_empty(),
        "test file should have no line-length diagnostics with relaxed limit, got: {test_file_diags:?}"
    );
}

#[test]
fn test_override_prod_file_still_flagged() {
    // Prod code should still be flagged even with relaxed test limit
    let config = config_with_test_line_length_override(200);
    let diags = test_helpers::run_on_fixture_dir("test_override", &config);
    let has_prod_diags = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("src/"))
        .any(|d| d.rule == "line-length");
    assert!(
        has_prod_diags,
        "production code should still have line-length diagnostics"
    );
}

#[test]
fn test_override_cfg_test_block_uses_test_rules() {
    // In a mixed file (src/main.rs with #[cfg(test)]), the #[cfg(test)] block
    // should use test rules (relaxed), while prod code uses prod rules.
    let config = config_with_test_line_length_override(200);
    let diags = test_helpers::run_on_fixture_dir("test_override", &config);

    let src_main_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("src/main.rs"))
        .filter(|d| d.rule == "line-length")
        .collect();

    // Only the prod line should be flagged (the one in production_function)
    // The line inside #[cfg(test)] mod tests should NOT be flagged
    assert_eq!(
        src_main_diags.len(),
        1,
        "expected 1 line-length diagnostic in src/main.rs (prod only), got {}: {src_main_diags:?}",
        src_main_diags.len()
    );
    // Verify it's on a prod line (line 6 is the long line in production_function)
    assert!(
        src_main_diags[0].line.is_some_and(|l| l < 9),
        "diagnostic should be on a production code line"
    );
}

#[test]
fn test_override_no_test_config_no_split() {
    // Without test config, all files are treated uniformly
    let config = Config::default();
    let diags = test_helpers::run_on_fixture_dir("test_override", &config);
    let all_line_length: Vec<_> = diags.iter().filter(|d| d.rule == "line-length").collect();
    // Both prod and test lines should be flagged with default rules
    assert!(
        all_line_length.len() >= 2,
        "without test config, all long lines should be flagged, got {}: {all_line_length:?}",
        all_line_length.len()
    );
}

#[test]
fn test_override_suffix_pattern() {
    // Test with a custom suffix pattern *_test.rs
    let config = Config {
        test: Some(TestConfig {
            patterns: vec!["*_test.rs".to_string()],
            detect_cfg_test: false,
            rules: TestRulesOverrides {
                line_length: Some(LineLengthOverride {
                    soft_limit: Some(200),
                    ..LineLengthOverride::default()
                }),
                ..TestRulesOverrides::default()
            },
        }),
        ..Config::default()
    };
    let diags = test_helpers::run_on_fixture_dir("test_override", &config);
    // test_main.rs doesn't match *_test.rs (it's test_main, not main_test)
    // so it should be flagged with prod rules
    let has_test_file_diags = diags
        .iter()
        .filter(|d| d.file.to_string_lossy().contains("test_main.rs"))
        .any(|d| d.rule == "line-length");
    assert!(
        has_test_file_diags,
        "test_main.rs should be flagged since it doesn't match *_test.rs pattern"
    );
}
