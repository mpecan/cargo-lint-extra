use crate::diagnostic::RuleLevel;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub global: GlobalConfig,
    pub rules: RulesConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    pub exclude: Vec<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            exclude: vec!["target".to_string()],
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RulesConfig {
    pub line_length: LineLengthConfig,
    pub file_length: FileLengthConfig,
    pub todo_comments: TodoCommentsConfig,
    pub file_header: FileHeaderConfig,
    pub allow_audit: AllowAuditConfig,
    pub inline_comments: InlineCommentsConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct LineLengthConfig {
    pub level: RuleLevel,
    pub soft_limit: usize,
    pub hard_limit: usize,
    pub url_exception: bool,
}

impl Default for LineLengthConfig {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            soft_limit: 120,
            hard_limit: 200,
            url_exception: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FileLengthConfig {
    pub level: RuleLevel,
    pub max: usize,
}

impl Default for FileLengthConfig {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            max: 500,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct TodoCommentsConfig {
    pub level: RuleLevel,
    pub keywords: Vec<String>,
    pub allow_with_issue: bool,
}

impl Default for TodoCommentsConfig {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            keywords: vec![
                "TODO".to_string(),
                "FIXME".to_string(),
                "HACK".to_string(),
                "XXX".to_string(),
            ],
            allow_with_issue: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct FileHeaderConfig {
    pub level: RuleLevel,
    pub required: Option<String>,
}

impl Default for FileHeaderConfig {
    fn default() -> Self {
        Self {
            level: RuleLevel::Allow,
            required: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AllowAuditConfig {
    pub level: RuleLevel,
    pub flagged: Vec<String>,
}

impl Default for AllowAuditConfig {
    fn default() -> Self {
        Self {
            level: RuleLevel::Allow,
            flagged: vec![
                "dead_code".to_string(),
                "unused_variables".to_string(),
                "unused_imports".to_string(),
            ],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct InlineCommentsConfig {
    pub level: RuleLevel,
    pub max_ratio: f64,
    pub max_consecutive: usize,
}

impl Default for InlineCommentsConfig {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            max_ratio: 0.3,
            max_consecutive: 3,
        }
    }
}

const CONFIG_FILE_NAME: &str = ".cargo-lint-extra.toml";

impl Config {
    /// # Errors
    /// Returns an error if the config file cannot be read or parsed.
    pub fn load(start_dir: &Path) -> Result<Self, String> {
        if let Some(path) = Self::find_config_file(start_dir) {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("failed to read config {}: {e}", path.display()))?;
            toml::from_str(&content)
                .map_err(|e| format!("failed to parse config {}: {e}", path.display()))
        } else {
            Ok(Self::default())
        }
    }

    fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
        let mut dir = start_dir;
        loop {
            let candidate = dir.join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                return Some(candidate);
            }
            dir = dir.parent()?;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.rules.line_length.soft_limit, 120);
        assert_eq!(config.rules.line_length.hard_limit, 200);
        assert_eq!(config.rules.file_length.max, 500);
        assert!(config.rules.line_length.url_exception);
        assert!(config.rules.todo_comments.allow_with_issue);
        assert_eq!(config.rules.todo_comments.keywords.len(), 4);
        assert_eq!(config.rules.file_header.level, RuleLevel::Allow);
        assert_eq!(config.rules.allow_audit.level, RuleLevel::Allow);
        assert_eq!(config.rules.inline_comments.level, RuleLevel::Warn);
        assert!((config.rules.inline_comments.max_ratio - 0.3).abs() < f64::EPSILON);
        assert_eq!(config.rules.inline_comments.max_consecutive, 3);
    }

    #[test]
    fn test_load_missing_config_uses_defaults() {
        let dir = std::env::temp_dir().join("cargo-lint-extra-test-no-config");
        fs::create_dir_all(&dir).unwrap();
        let config = Config::load(&dir).unwrap();
        assert_eq!(config.rules.line_length.soft_limit, 120);
        assert_eq!(config.rules.line_length.hard_limit, 200);
    }

    #[test]
    fn test_load_partial_config() {
        let dir = std::env::temp_dir().join("cargo-lint-extra-test-partial");
        fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join(CONFIG_FILE_NAME);
        fs::write(
            &config_path,
            r#"
[rules.line-length]
soft_limit = 100
hard_limit = 150
level = "deny"
"#,
        )
        .unwrap();
        let config = Config::load(&dir).unwrap();
        assert_eq!(config.rules.line_length.soft_limit, 100);
        assert_eq!(config.rules.line_length.hard_limit, 150);
        assert_eq!(config.rules.line_length.level, RuleLevel::Deny);
        assert_eq!(config.rules.file_length.max, 500);
        fs::remove_file(config_path).ok();
    }

    #[test]
    fn test_kebab_case_parsing() {
        let dir = std::env::temp_dir().join("cargo-lint-extra-test-kebab");
        fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join(CONFIG_FILE_NAME);
        fs::write(
            &config_path,
            r#"
[rules.todo-comments]
level = "deny"
allow_with_issue = false
keywords = ["TODO", "FIXME"]

[rules.allow-audit]
level = "warn"
flagged = ["dead_code"]
"#,
        )
        .unwrap();
        let config = Config::load(&dir).unwrap();
        assert_eq!(config.rules.todo_comments.level, RuleLevel::Deny);
        assert!(!config.rules.todo_comments.allow_with_issue);
        assert_eq!(config.rules.allow_audit.level, RuleLevel::Warn);
        assert_eq!(config.rules.allow_audit.flagged, vec!["dead_code"]);
        fs::remove_file(config_path).ok();
    }

    #[test]
    fn test_inline_comments_config_parsing() {
        let dir = std::env::temp_dir().join("cargo-lint-extra-test-inline");
        fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join(CONFIG_FILE_NAME);
        fs::write(
            &config_path,
            r#"
[rules.inline-comments]
level = "deny"
max_ratio = 0.5
max_consecutive = 5
"#,
        )
        .unwrap();
        let config = Config::load(&dir).unwrap();
        assert_eq!(config.rules.inline_comments.level, RuleLevel::Deny);
        assert!((config.rules.inline_comments.max_ratio - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.rules.inline_comments.max_consecutive, 5);
        fs::remove_file(config_path).ok();
    }
}
