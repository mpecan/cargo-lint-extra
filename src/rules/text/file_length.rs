use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use serde::Deserialize;
use std::path::Path;

// --- Config ---
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub level: RuleLevel,
    pub soft_limit: usize,
    pub hard_limit: usize,
    /// Deprecated alias for `soft_limit`. If set (non-zero), overrides `soft_limit`.
    #[doc(hidden)]
    #[serde(default)]
    pub max: usize,
}

/// Default `soft_limit` value, used to detect whether `soft_limit` was explicitly set.
const DEFAULT_SOFT_LIMIT: usize = 500;

impl Config {
    /// After deserialization, migrate the deprecated `max` field to `soft_limit`.
    /// Prints a deprecation warning to stderr. If both `max` and a non-default
    /// `soft_limit` are set, `soft_limit` takes precedence.
    pub fn migrate_deprecated(&mut self) {
        if self.max > 0 {
            eprintln!(
                "warning: 'max' in [rules.file-length] is deprecated, use 'soft_limit' instead"
            );
            if self.soft_limit == DEFAULT_SOFT_LIMIT {
                self.soft_limit = self.max;
            }
            self.max = 0;
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            soft_limit: 500,
            hard_limit: 1000,
            max: 0,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub soft_limit: Option<usize>,
    pub hard_limit: Option<usize>,
}

pub const fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.soft_limit {
        cfg.soft_limit = v;
    }
    if let Some(v) = o.hard_limit {
        cfg.hard_limit = v;
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    soft_limit: usize,
    hard_limit: usize,
}

/// Backward-compatible alias.
pub type FileLengthRule = Rule;
/// Backward-compatible alias.
pub type FileLengthConfig = Config;

impl Rule {
    pub const fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            soft_limit: config.soft_limit,
            hard_limit: config.hard_limit,
        }
    }

    /// Resolve the effective severity: hard limit always Deny;
    /// soft limit is Warn unless the configured level promotes it.
    const fn effective_level(&self, base: RuleLevel) -> RuleLevel {
        match (base, self.level) {
            (_, RuleLevel::Deny) => RuleLevel::Deny,
            (other, _) => other,
        }
    }
}

impl TextRule for Rule {
    fn name(&self) -> &'static str {
        "file-length"
    }

    fn check_file(&self, content: &str, file: &Path) -> Vec<Diagnostic> {
        let line_count = content.lines().count();
        if line_count > self.hard_limit {
            vec![Diagnostic::new(
                self.name(),
                RuleLevel::Deny,
                format!(
                    "file is {line_count} lines (hard limit {})",
                    self.hard_limit
                ),
                file,
            )]
        } else if line_count > self.soft_limit {
            vec![Diagnostic::new(
                self.name(),
                self.effective_level(RuleLevel::Warn),
                format!(
                    "file is {line_count} lines (soft limit {})",
                    self.soft_limit
                ),
                file,
            )]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_rule() -> Rule {
        Rule::new(&Config::default())
    }

    #[test]
    fn test_short_file_passes() {
        let rule = default_rule();
        let content = "line\n".repeat(100);
        assert!(rule.check_file(&content, Path::new("test.rs")).is_empty());
    }

    #[test]
    fn test_exact_soft_limit_passes() {
        let rule = default_rule();
        let content = "line\n".repeat(500);
        assert!(rule.check_file(&content, Path::new("test.rs")).is_empty());
    }

    #[test]
    fn test_over_soft_limit_warns() {
        let rule = default_rule();
        let content = "line\n".repeat(501);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "file-length");
        assert_eq!(diags[0].level, RuleLevel::Warn);
        assert!(diags[0].message.contains("soft limit"));
    }

    #[test]
    fn test_exact_hard_limit_warns() {
        let rule = default_rule();
        let content = "line\n".repeat(1000);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Warn);
    }

    #[test]
    fn test_over_hard_limit_denies() {
        let rule = default_rule();
        let content = "line\n".repeat(1001);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
        assert!(diags[0].message.contains("hard limit"));
    }

    #[test]
    fn test_level_deny_promotes_soft_limit_to_error() {
        let rule = Rule::new(&Config {
            level: RuleLevel::Deny,
            ..Config::default()
        });
        let content = "line\n".repeat(501);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_level_warn_keeps_soft_limit_as_warning() {
        let rule = Rule::new(&Config {
            level: RuleLevel::Warn,
            ..Config::default()
        });
        let content = "line\n".repeat(501);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags[0].level, RuleLevel::Warn);
    }

    #[test]
    fn test_custom_limits() {
        let rule = Rule::new(&Config {
            soft_limit: 10,
            hard_limit: 20,
            ..Config::default()
        });
        assert!(
            rule.check_file(&"line\n".repeat(10), Path::new("test.rs"))
                .is_empty()
        );
        let diags = rule.check_file(&"line\n".repeat(11), Path::new("test.rs"));
        assert_eq!(diags[0].level, RuleLevel::Warn);
        let diags = rule.check_file(&"line\n".repeat(21), Path::new("test.rs"));
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }
}
