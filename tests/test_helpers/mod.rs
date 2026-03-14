use cargo_lint_extra::config::Config;
use cargo_lint_extra::engine::Engine;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[allow(dead_code)]
pub fn run_on_fixture_dir(
    fixture_dir: &str,
    test_name: &str,
    config: &Config,
) -> Vec<cargo_lint_extra::diagnostic::Diagnostic> {
    let engine = Engine::new(config);
    let path = fixture_path(fixture_dir);
    let tmp = std::env::temp_dir().join(format!("cargo-lint-extra-int-{test_name}"));
    let _ = std::fs::remove_dir_all(&tmp);
    copy_dir_recursive(&path, &tmp);
    let result = engine.run(&tmp);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

#[allow(dead_code, clippy::unwrap_used)]
pub fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            std::fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

#[allow(dead_code)]
pub fn run_on_fixture(
    name: &str,
    test_name: &str,
    config: &Config,
) -> Vec<cargo_lint_extra::diagnostic::Diagnostic> {
    let engine = Engine::new(config);
    let path = fixture_path(name);
    let tmp = std::env::temp_dir().join(format!("cargo-lint-extra-int-{test_name}"));
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::create_dir_all(&tmp);
    std::fs::copy(&path, tmp.join(name)).unwrap();
    let result = engine.run(&tmp);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}
