use crate::config::LineLengthConfig;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use std::path::Path;

pub struct LineLengthRule {
    soft_limit: usize,
    hard_limit: usize,
    url_exception: bool,
}

impl LineLengthRule {
    pub const fn new(config: &LineLengthConfig) -> Self {
        Self {
            soft_limit: config.soft_limit,
            hard_limit: config.hard_limit,
            url_exception: config.url_exception,
        }
    }

    fn line_is_exempt(&self, line: &str) -> bool {
        if self.url_exception {
            let trimmed = line.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                return true;
            }
            if (trimmed.starts_with("//") || trimmed.starts_with("///"))
                && (trimmed.contains("http://") || trimmed.contains("https://"))
            {
                return true;
            }
        }
        false
    }
}

impl TextRule for LineLengthRule {
    fn name(&self) -> &'static str {
        "line-length"
    }

    fn check_line(&self, line: &str, line_number: usize, file: &Path) -> Option<Diagnostic> {
        let char_count = line.chars().count();
        if self.line_is_exempt(line) {
            return None;
        }
        if char_count > self.hard_limit {
            Some(
                Diagnostic::new(
                    self.name(),
                    RuleLevel::Deny,
                    format!(
                        "line is {char_count} chars (hard limit {})",
                        self.hard_limit
                    ),
                    file,
                )
                .with_line(line_number),
            )
        } else if char_count > self.soft_limit {
            Some(
                Diagnostic::new(
                    self.name(),
                    RuleLevel::Warn,
                    format!(
                        "line is {char_count} chars (soft limit {})",
                        self.soft_limit
                    ),
                    file,
                )
                .with_line(line_number),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn default_rule() -> LineLengthRule {
        LineLengthRule::new(&LineLengthConfig::default())
    }

    #[test]
    fn test_short_line_passes() {
        let rule = default_rule();
        let result = rule.check_line("fn main() {}", 1, Path::new("test.rs"));
        assert!(result.is_none());
    }

    #[test]
    fn test_over_soft_limit_warns() {
        let rule = default_rule();
        let long_line = "x".repeat(121);
        let result = rule.check_line(&long_line, 1, Path::new("test.rs"));
        assert!(result.is_some());
        let diag = result.unwrap();
        assert_eq!(diag.rule, "line-length");
        assert_eq!(diag.level, RuleLevel::Warn);
        assert!(diag.message.contains("soft limit"));
    }

    #[test]
    fn test_over_hard_limit_denies() {
        let rule = default_rule();
        let long_line = "x".repeat(201);
        let result = rule.check_line(&long_line, 1, Path::new("test.rs"));
        assert!(result.is_some());
        let diag = result.unwrap();
        assert_eq!(diag.level, RuleLevel::Deny);
        assert!(diag.message.contains("hard limit"));
    }

    #[test]
    fn test_exact_soft_limit_passes() {
        let rule = default_rule();
        let line = "x".repeat(120);
        assert!(rule.check_line(&line, 1, Path::new("test.rs")).is_none());
    }

    #[test]
    fn test_exact_hard_limit_warns() {
        let rule = default_rule();
        let line = "x".repeat(200);
        let result = rule.check_line(&line, 1, Path::new("test.rs"));
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RuleLevel::Warn);
    }

    #[test]
    fn test_url_exception() {
        let rule = default_rule();
        let line = format!("// see https://example.com/{}", "x".repeat(200));
        assert!(rule.check_line(&line, 1, Path::new("test.rs")).is_none());
    }

    #[test]
    fn test_url_exception_disabled() {
        let rule = LineLengthRule::new(&LineLengthConfig {
            url_exception: false,
            ..LineLengthConfig::default()
        });
        let line = format!("// see https://example.com/{}", "x".repeat(200));
        assert!(rule.check_line(&line, 1, Path::new("test.rs")).is_some());
    }

    #[test]
    fn test_custom_limits() {
        let rule = LineLengthRule::new(&LineLengthConfig {
            soft_limit: 80,
            hard_limit: 100,
            ..LineLengthConfig::default()
        });
        assert!(
            rule.check_line(&"x".repeat(80), 1, Path::new("test.rs"))
                .is_none()
        );
        let result = rule.check_line(&"x".repeat(90), 1, Path::new("test.rs"));
        assert_eq!(result.unwrap().level, RuleLevel::Warn);
        let result = rule.check_line(&"x".repeat(101), 1, Path::new("test.rs"));
        assert_eq!(result.unwrap().level, RuleLevel::Deny);
    }
}
