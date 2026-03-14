use cargo_lint_extra::config::Config;
use cargo_lint_extra::diagnostic::Diagnostic;
use cargo_lint_extra::engine::Engine;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[allow(dead_code, clippy::expect_used)]
pub fn run_on_fixture_dir(fixture_dir: &str, config: &Config) -> Vec<Diagnostic> {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    copy_dir_recursive(&fixture_path(fixture_dir), tmp.path());
    let engine = Engine::new(config);
    engine.run(tmp.path())
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

#[allow(dead_code, clippy::unwrap_used, clippy::expect_used)]
pub fn run_on_fixture(name: &str, config: &Config) -> Vec<Diagnostic> {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::copy(fixture_path(name), tmp.path().join(name)).unwrap();
    let engine = Engine::new(config);
    engine.run(tmp.path())
}
