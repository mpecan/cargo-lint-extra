use crate::config::FileLengthConfig;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use std::path::Path;

pub struct FileLengthRule {
    level: RuleLevel,
    max: usize,
}

impl FileLengthRule {
    pub const fn new(config: &FileLengthConfig) -> Self {
        Self {
            level: config.level,
            max: config.max,
        }
    }
}

impl TextRule for FileLengthRule {
    fn name(&self) -> &'static str {
        "file-length"
    }

    fn check_file(&self, content: &str, file: &Path) -> Vec<Diagnostic> {
        let line_count = content.lines().count();
        if line_count > self.max {
            vec![Diagnostic::new(
                self.name(),
                self.level,
                format!("file is {line_count} lines (max {})", self.max),
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

    #[test]
    fn test_short_file_passes() {
        let rule = FileLengthRule::new(&FileLengthConfig::default());
        let content = "line\n".repeat(100);
        assert!(rule.check_file(&content, Path::new("test.rs")).is_empty());
    }

    #[test]
    fn test_long_file_fails() {
        let rule = FileLengthRule::new(&FileLengthConfig::default());
        let content = "line\n".repeat(501);
        let diags = rule.check_file(&content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].rule, "file-length");
    }

    #[test]
    fn test_exact_max_passes() {
        let rule = FileLengthRule::new(&FileLengthConfig::default());
        let content = "line\n".repeat(500);
        assert!(rule.check_file(&content, Path::new("test.rs")).is_empty());
    }

    #[test]
    fn test_custom_max() {
        let rule = FileLengthRule::new(&FileLengthConfig {
            max: 10,
            ..FileLengthConfig::default()
        });
        let content = "line\n".repeat(11);
        assert_eq!(rule.check_file(&content, Path::new("test.rs")).len(), 1);
    }
}
