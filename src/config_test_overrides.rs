use crate::config::{
    AllowAuditConfig, FileHeaderConfig, FileLengthConfig, InlineCommentsConfig, LineLengthConfig,
    RulesConfig, TodoCommentsConfig,
};
use crate::diagnostic::RuleLevel;
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TestRulesOverrides {
    pub line_length: Option<LineLengthOverride>,
    pub file_length: Option<FileLengthOverride>,
    pub todo_comments: Option<TodoCommentsOverride>,
    pub file_header: Option<FileHeaderOverride>,
    pub allow_audit: Option<AllowAuditOverride>,
    pub inline_comments: Option<InlineCommentsOverride>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LineLengthOverride {
    pub level: Option<RuleLevel>,
    pub soft_limit: Option<usize>,
    pub hard_limit: Option<usize>,
    pub url_exception: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FileLengthOverride {
    pub level: Option<RuleLevel>,
    pub max: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TodoCommentsOverride {
    pub level: Option<RuleLevel>,
    pub keywords: Option<Vec<String>>,
    pub allow_with_issue: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FileHeaderOverride {
    pub level: Option<RuleLevel>,
    pub required: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AllowAuditOverride {
    pub level: Option<RuleLevel>,
    pub flagged: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct InlineCommentsOverride {
    pub level: Option<RuleLevel>,
    pub max_ratio: Option<f64>,
    pub max_consecutive: Option<usize>,
}

/// Apply test overrides to a cloned `RulesConfig`, merging only `Some` fields.
pub(crate) fn apply_test_overrides(rules: &mut RulesConfig, overrides: &TestRulesOverrides) {
    if let Some(o) = &overrides.line_length {
        apply_line_length_override(&mut rules.line_length, o);
    }
    if let Some(o) = &overrides.file_length {
        apply_file_length_override(&mut rules.file_length, o);
    }
    if let Some(o) = &overrides.todo_comments {
        apply_todo_comments_override(&mut rules.todo_comments, o);
    }
    if let Some(o) = &overrides.file_header {
        apply_file_header_override(&mut rules.file_header, o);
    }
    if let Some(o) = &overrides.allow_audit {
        apply_allow_audit_override(&mut rules.allow_audit, o);
    }
    if let Some(o) = &overrides.inline_comments {
        apply_inline_comments_override(&mut rules.inline_comments, o);
    }
}

const fn apply_line_length_override(cfg: &mut LineLengthConfig, o: &LineLengthOverride) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.soft_limit {
        cfg.soft_limit = v;
    }
    if let Some(v) = o.hard_limit {
        cfg.hard_limit = v;
    }
    if let Some(v) = o.url_exception {
        cfg.url_exception = v;
    }
}

const fn apply_file_length_override(cfg: &mut FileLengthConfig, o: &FileLengthOverride) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.max {
        cfg.max = v;
    }
}

fn apply_todo_comments_override(cfg: &mut TodoCommentsConfig, o: &TodoCommentsOverride) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = &o.keywords {
        cfg.keywords.clone_from(v);
    }
    if let Some(v) = o.allow_with_issue {
        cfg.allow_with_issue = v;
    }
}

fn apply_file_header_override(cfg: &mut FileHeaderConfig, o: &FileHeaderOverride) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if o.required.is_some() {
        cfg.required.clone_from(&o.required);
    }
}

fn apply_allow_audit_override(cfg: &mut AllowAuditConfig, o: &AllowAuditOverride) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = &o.flagged {
        cfg.flagged.clone_from(v);
    }
}

const fn apply_inline_comments_override(
    cfg: &mut InlineCommentsConfig,
    o: &InlineCommentsOverride,
) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.max_ratio {
        cfg.max_ratio = v;
    }
    if let Some(v) = o.max_consecutive {
        cfg.max_consecutive = v;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::Config;

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
        assert_eq!(resolved.file_length.max, 500);
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
