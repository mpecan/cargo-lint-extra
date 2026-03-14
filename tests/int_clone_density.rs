#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::{CloneDensityConfig, Config};
use cargo_lint_extra::diagnostic::RuleLevel;

#[test]
fn test_clone_density_detected() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("clone_density.rs", "clone-density", &config);
    let clone_diags: Vec<_> = diags.iter().filter(|d| d.rule == "clone-density").collect();
    assert_eq!(
        clone_diags.len(),
        2,
        "expected 2 clone-density diagnostics (too_many_clones + clone_heavy_method), got {}: {clone_diags:?}",
        clone_diags.len()
    );
    assert!(
        clone_diags
            .iter()
            .any(|d| d.message.contains("too_many_clones"))
    );
    assert!(
        clone_diags
            .iter()
            .any(|d| d.message.contains("clone_heavy_method"))
    );
}

#[test]
fn test_clone_density_disabled() {
    let mut config = Config::default();
    config.rules.clone_density.level = RuleLevel::Allow;
    let diags = test_helpers::run_on_fixture("clone_density.rs", "clone-density-disabled", &config);
    assert!(
        !diags.iter().any(|d| d.rule == "clone-density"),
        "disabled clone-density rule should produce no diagnostics"
    );
}

#[test]
fn test_clone_density_custom_thresholds() {
    let mut config = Config::default();
    config.rules.clone_density = CloneDensityConfig {
        level: RuleLevel::Warn,
        max_clones_per_fn: 10,
        max_clone_ratio: 1.0,
    };
    let diags = test_helpers::run_on_fixture("clone_density.rs", "clone-density-relaxed", &config);
    let clone_diags: Vec<_> = diags.iter().filter(|d| d.rule == "clone-density").collect();
    assert!(
        clone_diags.is_empty(),
        "relaxed thresholds should produce no diagnostics, got: {clone_diags:?}"
    );
}
