use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use serde::Deserialize;
use std::path::Path;

// --- Config ---
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub level: RuleLevel,
    pub required: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Allow,
            required: None,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub required: Option<String>,
}

pub fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if o.required.is_some() {
        cfg.required.clone_from(&o.required);
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    required: Option<String>,
}

/// Backward-compatible alias.
pub type FileHeaderRule = Rule;
/// Backward-compatible alias.
pub type FileHeaderConfig = Config;

impl Rule {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            required: config.required.clone(),
        }
    }
}

impl TextRule for Rule {
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
        let rule = Rule::new(&Config::default());
        assert!(
            rule.check_file("fn main() {}", Path::new("test.rs"))
                .is_empty()
        );
    }

    #[test]
    fn test_matching_header_passes() {
        let rule = Rule::new(&Config {
            level: RuleLevel::Deny,
            required: Some("// Copyright".to_string()),
        });
        let content = "// Copyright 2024 Acme Corp\nfn main() {}";
        assert!(rule.check_file(content, Path::new("test.rs")).is_empty());
    }

    #[test]
    fn test_missing_header_fails() {
        let rule = Rule::new(&Config {
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
        let rule = Rule::new(&Config {
            level: RuleLevel::Warn,
            required: Some("// License".to_string()),
        });
        let content = "\n\n// License: MIT\nfn main() {}";
        assert!(rule.check_file(content, Path::new("test.rs")).is_empty());
    }
}
