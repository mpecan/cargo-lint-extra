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
    pub url_exception: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            soft_limit: 120,
            hard_limit: 200,
            url_exception: true,
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
    pub url_exception: Option<bool>,
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
    if let Some(v) = o.url_exception {
        cfg.url_exception = v;
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    soft_limit: usize,
    hard_limit: usize,
    url_exception: bool,
}

/// Backward-compatible alias.
pub type LineLengthRule = Rule;

impl Rule {
    pub const fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            soft_limit: config.soft_limit,
            hard_limit: config.hard_limit,
            url_exception: config.url_exception,
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

impl TextRule for Rule {
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
                    self.effective_level(RuleLevel::Warn),
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
        LineLengthRule::new(&Config::default())
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
        let rule = LineLengthRule::new(&Config {
            url_exception: false,
            ..Config::default()
        });
        let line = format!("// see https://example.com/{}", "x".repeat(200));
        assert!(rule.check_line(&line, 1, Path::new("test.rs")).is_some());
    }

    #[test]
    fn test_level_deny_promotes_soft_limit_to_error() {
        let rule = LineLengthRule::new(&Config {
            level: RuleLevel::Deny,
            ..Config::default()
        });
        let long_line = "x".repeat(121);
        let diag = rule
            .check_line(&long_line, 1, Path::new("test.rs"))
            .unwrap();
        assert_eq!(diag.level, RuleLevel::Deny);
    }

    #[test]
    fn test_custom_limits() {
        let rule = LineLengthRule::new(&Config {
            soft_limit: 80,
            hard_limit: 100,
            ..Config::default()
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
