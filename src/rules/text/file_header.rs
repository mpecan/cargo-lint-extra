use crate::config::FileHeaderConfig;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use std::path::Path;

pub struct FileHeaderRule {
    level: RuleLevel,
    required: Option<String>,
}

impl FileHeaderRule {
    pub fn new(config: &FileHeaderConfig) -> Self {
        Self {
            level: config.level,
            required: config.required.clone(),
        }
    }
}

impl TextRule for FileHeaderRule {
    fn name(&self) -> &'static str {
        "file-header"
    }

    fn check_file(&self, content: &str, file: &Path) -> Vec<Diagnostic> {
        let Some(required) = &self.required else {
            return Vec::new();
        };

        let first_non_empty = content.lines().find(|line| !line.trim().is_empty());

        match first_non_empty {
            Some(line) if line.contains(required.as_str()) => Vec::new(),
            _ => vec![
                Diagnostic::new(
                    self.name(),
                    self.level,
                    format!("missing required file header: {required}"),
                    file,
                )
                .with_line(1),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_required_header_passes() {
        let rule = FileHeaderRule::new(&FileHeaderConfig::default());
        assert!(
            rule.check_file("fn main() {}", Path::new("test.rs"))
                .is_empty()
        );
    }

    #[test]
    fn test_matching_header_passes() {
        let rule = FileHeaderRule::new(&FileHeaderConfig {
            level: RuleLevel::Deny,
            required: Some("// Copyright".to_string()),
        });
        let content = "// Copyright 2024 Acme Corp\nfn main() {}";
        assert!(rule.check_file(content, Path::new("test.rs")).is_empty());
    }

    #[test]
    fn test_missing_header_fails() {
        let rule = FileHeaderRule::new(&FileHeaderConfig {
            level: RuleLevel::Deny,
            required: Some("// Copyright".to_string()),
        });
        let content = "fn main() {}";
        let diags = rule.check_file(content, Path::new("test.rs"));
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("Copyright"));
    }

    #[test]
    fn test_skips_empty_lines() {
        let rule = FileHeaderRule::new(&FileHeaderConfig {
            level: RuleLevel::Warn,
            required: Some("// License".to_string()),
        });
        let content = "\n\n// License: MIT\nfn main() {}";
        assert!(rule.check_file(content, Path::new("test.rs")).is_empty());
    }
}
