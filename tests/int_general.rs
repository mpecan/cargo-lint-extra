#![allow(clippy::unwrap_used)]

mod test_helpers;

use cargo_lint_extra::config::Config;

#[test]
fn test_clean_file_produces_no_diagnostics() {
    let config = Config::default();
    let diags = test_helpers::run_on_fixture("clean.rs", &config);
    assert!(
        diags.is_empty(),
        "clean file should have no diagnostics, got: {diags:?}"
    );
}
