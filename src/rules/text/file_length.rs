use crate::config::FileLengthConfig;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use std::path::Path;

pub struct FileLengthRule {
    level: RuleLevel,
    soft_limit: usize,
    hard_limit: usize,
}

impl FileLengthRule {
    pub const fn new(config: &FileLengthConfig) -> Self {
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

impl TextRule for FileLengthRule {
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

    fn default_rule() -> FileLengthRule {
        FileLengthRule::new(&FileLengthConfig::default())
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
        let rule = FileLengthRule::new(&FileLengthConfig {
            level: RuleLevel::Deny,
            ..FileLengthConfig::default()
        });
        let content = "line\n".repeat(501);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_level_warn_keeps_soft_limit_as_warning() {
        let rule = FileLengthRule::new(&FileLengthConfig {
            level: RuleLevel::Warn,
            ..FileLengthConfig::default()
        });
        let content = "line\n".repeat(501);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags[0].level, RuleLevel::Warn);
    }

    #[test]
    fn test_custom_limits() {
        let rule = FileLengthRule::new(&FileLengthConfig {
            soft_limit: 10,
            hard_limit: 20,
            ..FileLengthConfig::default()
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
