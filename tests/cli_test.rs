#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::Command;

fn cargo_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cargo-lint-extra"))
}

fn fixture_dir(name: &str, test_name: &str) -> PathBuf {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    let tmp = std::env::temp_dir().join(format!("cargo-lint-extra-cli-{test_name}"));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::copy(&fixture, tmp.join(name)).unwrap();
    tmp
}

#[test]
fn test_exit_code_0_clean_file() {
    let dir = fixture_dir("clean.rs", "exit0");
    let output = Command::new(cargo_bin())
        .args(["lint-extra"])
        .arg(dir.to_str().unwrap())
        .output()
        .expect("failed to run binary");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        output.status.code(),
        Some(0),
        "clean file should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_exit_code_1_findings() {
    let dir = fixture_dir("long_lines.rs", "exit1");
    let output = Command::new(cargo_bin())
        .args(["lint-extra"])
        .arg(dir.to_str().unwrap())
        .output()
        .expect("failed to run binary");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        output.status.code(),
        Some(1),
        "file with findings should exit 1"
    );
}

#[test]
fn test_exit_code_2_invalid_path() {
    let output = Command::new(cargo_bin())
        .args(["lint-extra", "/nonexistent/path/that/does/not/exist"])
        .output()
        .expect("failed to run binary");
    assert_eq!(output.status.code(), Some(2), "invalid path should exit 2");
}

#[test]
fn test_json_output_valid() {
    let dir = fixture_dir("long_lines.rs", "json");
    let output = Command::new(cargo_bin())
        .args(["lint-extra", "--format", "json"])
        .arg(dir.to_str().unwrap())
        .output()
        .expect("failed to run binary");
    let _ = std::fs::remove_dir_all(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON output: {e}\n{stdout}"));
    assert!(parsed.is_array(), "JSON output should be an array");
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty(), "should have findings");
    for item in arr {
        assert!(item["rule"].is_string());
        assert!(item["level"].is_string());
        assert!(item["message"].is_string());
        assert!(item["file"].is_string());
    }
}

#[test]
fn test_human_output_format() {
    let dir = fixture_dir("todos.rs", "human");
    let output = Command::new(cargo_bin())
        .args(["lint-extra", "--format", "human"])
        .arg(dir.to_str().unwrap())
        .output()
        .expect("failed to run binary");
    let _ = std::fs::remove_dir_all(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("warning"),
        "human output should contain warning level"
    );
    assert!(
        stdout.contains("todo-comments"),
        "human output should contain rule name"
    );
}

#[test]
fn test_disable_flag() {
    let dir = fixture_dir("long_lines.rs", "disable");
    let output = Command::new(cargo_bin())
        .args([
            "lint-extra",
            "--disable",
            "line-length,file-length,todo-comments",
        ])
        .arg(dir.to_str().unwrap())
        .output()
        .expect("failed to run binary");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        output.status.code(),
        Some(0),
        "all rules disabled should exit 0"
    );
}

#[test]
fn test_enable_flag() {
    let dir = fixture_dir("allow_attrs.rs", "enable");
    let output = Command::new(cargo_bin())
        .args(["lint-extra", "--format", "json", "--enable", "allow-audit"])
        .arg(dir.to_str().unwrap())
        .output()
        .expect("failed to run binary");
    let _ = std::fs::remove_dir_all(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(
        arr.iter().any(|d| d["rule"] == "allow-audit"),
        "enabling allow-audit should produce diagnostics"
    );
}
