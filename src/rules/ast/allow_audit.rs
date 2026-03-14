use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::AstRule;
use serde::Deserialize;
use std::path::Path;
use syn::visit::Visit;

// --- Config ---
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub level: RuleLevel,
    pub flagged: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Allow,
            flagged: vec![
                "dead_code".to_string(),
                "unused_variables".to_string(),
                "unused_imports".to_string(),
            ],
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub flagged: Option<Vec<String>>,
}

pub fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = &o.flagged {
        cfg.flagged.clone_from(v);
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    flagged: Vec<String>,
}

/// Backward-compatible alias.
pub type AllowAuditRule = Rule;
/// Backward-compatible alias.
pub type AllowAuditConfig = Config;

impl Rule {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            flagged: config.flagged.clone(),
        }
    }
}

struct AllowVisitor<'a> {
    level: RuleLevel,
    flagged: &'a [String],
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
}

impl<'ast> Visit<'ast> for AllowVisitor<'_> {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if attr.path().is_ident("allow")
            && let syn::Meta::List(meta_list) = &attr.meta
        {
            let tokens_str = meta_list.tokens.to_string();
            for flagged_lint in self.flagged {
                if tokens_str.contains(flagged_lint.as_str()) {
                    let line = attr.pound_token.span.start().line;
                    self.diagnostics.push(
                        Diagnostic::new(
                            "allow-audit",
                            self.level,
                            format!("#[allow({flagged_lint})] found"),
                            self.file,
                        )
                        .with_line(line),
                    );
                }
            }
        }
        syn::visit::visit_attribute(self, attr);
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "allow-audit"
    }

    fn check_file(&self, syntax: &syn::File, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = AllowVisitor {
            level: self.level,
            flagged: &self.flagged,
            file,
            diagnostics: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn parse_and_check(code: &str) -> Vec<Diagnostic> {
        let syntax = syn::parse_file(code).expect("failed to parse test code");
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let rule = Rule::new(&config);
        rule.check_file(&syntax, Path::new("test.rs"))
    }

    #[test]
    fn test_no_allow_attribute() {
        let diags = parse_and_check("fn main() {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allow_dead_code_flagged() {
        let diags = parse_and_check("#[allow(dead_code)]\nfn unused() {}");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("dead_code"));
    }

    #[test]
    fn test_allow_unused_variables_flagged() {
        let diags = parse_and_check("#[allow(unused_variables)]\nfn main() { let x = 1; }");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("unused_variables"));
    }

    #[test]
    fn test_allow_non_flagged_passes() {
        let diags = parse_and_check("#[allow(clippy::too_many_arguments)]\nfn main() {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_deny_attribute_not_flagged() {
        let diags = parse_and_check("#[deny(dead_code)]\nfn main() {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_multiple_allows() {
        let diags = parse_and_check(
            "#[allow(dead_code)]\nfn a() {}\n#[allow(unused_imports)]\nuse std::io;",
        );
        assert_eq!(diags.len(), 2);
    }
}
