use crate::rule_registry::TestRulesOverrides;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TestConfig {
    pub patterns: Vec<String>,
    pub detect_cfg_test: bool,
    pub rules: TestRulesOverrides,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            patterns: vec!["tests/".to_string(), "benches/".to_string()],
            detect_cfg_test: true,
            rules: TestRulesOverrides::default(),
        }
    }
}

impl TestConfig {
    /// Check if a relative path matches the test file patterns.
    /// Normal patterns are prefix-matched. Patterns starting with `*` are
    /// suffix-matched.
    pub fn is_test_file(&self, relative_path: &str) -> bool {
        self.patterns.iter().any(|pattern| {
            pattern.strip_prefix('*').map_or_else(
                || relative_path.starts_with(pattern),
                |suffix| relative_path.ends_with(suffix),
            )
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::{Config, LineLengthOverride};
    use crate::diagnostic::RuleLevel;

    // Re-import override types through config re-exports
    use crate::config::AllowAuditOverride;

    #[test]
    fn test_test_config_defaults() {
        let tc = TestConfig::default();
        assert_eq!(tc.patterns, vec!["tests/", "benches/"]);
        assert!(tc.detect_cfg_test);
    }

    #[test]
    fn test_is_test_file_prefix() {
        let tc = TestConfig::default();
        assert!(tc.is_test_file("tests/foo.rs"));
        assert!(tc.is_test_file("benches/bench.rs"));
        assert!(!tc.is_test_file("src/lib.rs"));
        assert!(!tc.is_test_file("src/tests.rs"));
    }

    #[test]
    fn test_is_test_file_suffix() {
        let tc = TestConfig {
            patterns: vec!["*_test.rs".to_string()],
            ..TestConfig::default()
        };
        assert!(tc.is_test_file("src/main_test.rs"));
        assert!(tc.is_test_file("foo_test.rs"));
        assert!(!tc.is_test_file("src/test_main.rs"));
        assert!(!tc.is_test_file("tests/foo.rs"));
    }

    #[test]
    fn test_resolved_test_rules_no_test_section() {
        let config = Config::default();
        let resolved = config.resolved_test_rules();
        assert_eq!(resolved.line_length.soft_limit, 120);
        assert_eq!(resolved.line_length.hard_limit, 200);
    }

    #[test]
    fn test_resolved_test_rules_override_merges() {
        let config = Config {
            test: Some(TestConfig {
                rules: TestRulesOverrides {
                    line_length: Some(LineLengthOverride {
                        soft_limit: Some(150),
                        ..LineLengthOverride::default()
                    }),
                    allow_audit: Some(AllowAuditOverride {
                        level: Some(RuleLevel::Allow),
                        ..AllowAuditOverride::default()
                    }),
                    ..TestRulesOverrides::default()
                },
                ..TestConfig::default()
            }),
            ..Config::default()
        };

        let resolved = config.resolved_test_rules();
        // Overridden fields
        assert_eq!(resolved.line_length.soft_limit, 150);
        assert_eq!(resolved.allow_audit.level, RuleLevel::Allow);
        // Inherited fields
        assert_eq!(resolved.line_length.hard_limit, 200);
        assert!(resolved.line_length.url_exception);
        assert_eq!(resolved.file_length.soft_limit, 500);
    }

    #[test]
    fn test_toml_parsing_with_test_section() {
        use crate::config::CONFIG_FILE_NAME;
        use std::fs;

        let dir = std::env::temp_dir().join("cargo-lint-extra-test-test-section");
        fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join(CONFIG_FILE_NAME);
        fs::write(
            &config_path,
            r#"
[rules.line-length]
soft_limit = 100

[test]
patterns = ["tests/", "spec/"]
detect_cfg_test = false

[test.rules.line-length]
soft_limit = 200
"#,
        )
        .unwrap();
        let config = Config::load(&dir).unwrap();
        assert_eq!(config.rules.line_length.soft_limit, 100);
        let test = config.test.as_ref().unwrap();
        assert_eq!(test.patterns, vec!["tests/", "spec/"]);
        assert!(!test.detect_cfg_test);
        let resolved = config.resolved_test_rules();
        assert_eq!(resolved.line_length.soft_limit, 200);
        // hard_limit inherited from prod default
        assert_eq!(resolved.line_length.hard_limit, 200);
        fs::remove_file(config_path).ok();
    }
}
