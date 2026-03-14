use serde::Deserialize;
use std::path::{Path, PathBuf};

pub use crate::config_test_overrides::TestConfig;
pub use crate::rule_registry::{RulesConfig, TestRulesOverrides};

// Re-export per-rule config types for backward compatibility
pub use crate::rules::ast::allow_audit::Config as AllowAuditConfig;
pub use crate::rules::ast::clone_density::Config as CloneDensityConfig;
pub use crate::rules::ast::glob_imports::Config as GlobImportsConfig;
pub use crate::rules::ast::magic_numbers::Config as MagicNumbersConfig;
pub use crate::rules::text::file_header::Config as FileHeaderConfig;
pub use crate::rules::text::file_length::Config as FileLengthConfig;
pub use crate::rules::text::inline_comments::Config as InlineCommentsConfig;
pub use crate::rules::text::line_length::Config as LineLengthConfig;
pub use crate::rules::text::redundant_comments::Config as RedundantCommentsConfig;
pub use crate::rules::text::todo_comments::Config as TodoCommentsConfig;

// Re-export per-rule override types for backward compatibility
pub use crate::rules::ast::allow_audit::Override as AllowAuditOverride;
pub use crate::rules::ast::clone_density::Override as CloneDensityOverride;
pub use crate::rules::ast::glob_imports::Override as GlobImportsOverride;
pub use crate::rules::ast::magic_numbers::Override as MagicNumbersOverride;
pub use crate::rules::text::file_header::Override as FileHeaderOverride;
pub use crate::rules::text::file_length::Override as FileLengthOverride;
pub use crate::rules::text::inline_comments::Override as InlineCommentsOverride;
pub use crate::rules::text::line_length::Override as LineLengthOverride;
pub use crate::rules::text::redundant_comments::Override as RedundantCommentsOverride;
pub use crate::rules::text::todo_comments::Override as TodoCommentsOverride;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub global: GlobalConfig,
    pub rules: RulesConfig,
    pub test: Option<TestConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    pub exclude: Vec<String>,
}

pub const CONFIG_FILE_NAME: &str = ".cargo-lint-extra.toml";

impl Config {
    /// Build a `RulesConfig` for test files by cloning the base rules and
    /// applying any overrides from the `[test]` section.
    pub fn resolved_test_rules(&self) -> RulesConfig {
        let Some(test) = &self.test else {
            return self.rules.clone();
        };
        let mut rules = self.rules.clone();
        crate::rule_registry::apply_test_overrides(&mut rules, &test.rules);
        rules
    }

    /// # Errors
    /// Returns an error if the config file cannot be read or parsed.
    pub fn load(start_dir: &Path) -> Result<Self, String> {
        if let Some(path) = Self::find_config_file(start_dir) {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("failed to read config {}: {e}", path.display()))?;
            let mut config: Self = toml::from_str(&content)
                .map_err(|e| format!("failed to parse config {}: {e}", path.display()))?;
            config.rules.file_length.migrate_deprecated();
            Ok(config)
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
    use crate::diagnostic::RuleLevel;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.rules.line_length.soft_limit, 120);
        assert_eq!(config.rules.line_length.hard_limit, 200);
        assert_eq!(config.rules.file_length.soft_limit, 500);
        assert_eq!(config.rules.file_length.hard_limit, 1000);
        assert!(config.rules.line_length.url_exception);
        assert!(config.rules.todo_comments.allow_with_issue);
        assert_eq!(config.rules.todo_comments.keywords.len(), 4);
        assert_eq!(config.rules.file_header.level, RuleLevel::Allow);
        assert_eq!(config.rules.allow_audit.level, RuleLevel::Allow);
        assert_eq!(config.rules.inline_comments.level, RuleLevel::Warn);
        assert!((config.rules.inline_comments.max_ratio - 0.3).abs() < f64::EPSILON);
        assert_eq!(config.rules.inline_comments.max_consecutive, 3);
        assert_eq!(config.rules.redundant_comments.level, RuleLevel::Warn);
        assert!((config.rules.redundant_comments.similarity_threshold - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.rules.redundant_comments.min_words, 2);
        assert_eq!(config.rules.clone_density.level, RuleLevel::Warn);
        assert_eq!(config.rules.clone_density.max_clones_per_fn, 5);
        assert!((config.rules.clone_density.max_clone_ratio - 0.1).abs() < f64::EPSILON);
        assert_eq!(config.rules.glob_imports.level, RuleLevel::Warn);
        assert!(config.rules.glob_imports.allowed_crates.is_empty());
        assert!(config.rules.glob_imports.allow_in_tests);
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
        assert_eq!(config.rules.file_length.soft_limit, 500);
        assert_eq!(config.rules.file_length.hard_limit, 1000);
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

    #[test]
    fn test_deprecated_max_migrates_to_soft_limit() {
        let dir = std::env::temp_dir().join("cargo-lint-extra-test-deprecated-max");
        fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join(CONFIG_FILE_NAME);
        fs::write(
            &config_path,
            "
[rules.file-length]
max = 400
",
        )
        .unwrap();
        let config = Config::load(&dir).unwrap();
        assert_eq!(config.rules.file_length.soft_limit, 400);
        assert_eq!(config.rules.file_length.hard_limit, 1000);
        fs::remove_file(config_path).ok();
    }

    #[test]
    fn test_deprecated_max_conflict_soft_limit_wins() {
        let dir = std::env::temp_dir().join("cargo-lint-extra-test-max-conflict");
        fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join(CONFIG_FILE_NAME);
        fs::write(
            &config_path,
            "
[rules.file-length]
max = 300
soft_limit = 400
",
        )
        .unwrap();
        let config = Config::load(&dir).unwrap();
        // soft_limit was explicitly set (not default), so it wins over max
        assert_eq!(config.rules.file_length.soft_limit, 400);
        fs::remove_file(config_path).ok();
    }
}
