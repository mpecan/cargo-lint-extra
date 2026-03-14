use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::AstRule;
use serde::Deserialize;
use std::path::Path;
use syn::UseTree;
use syn::visit::Visit;

// --- Config ---
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub level: RuleLevel,
    pub allowed_crates: Vec<String>,
    pub allow_in_tests: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            allowed_crates: Vec::new(),
            allow_in_tests: true,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub allowed_crates: Option<Vec<String>>,
    pub allow_in_tests: Option<bool>,
}

pub fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = &o.allowed_crates {
        cfg.allowed_crates.clone_from(v);
    }
    if let Some(v) = o.allow_in_tests {
        cfg.allow_in_tests = v;
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    allowed_crates: Vec<String>,
    allow_in_tests: bool,
}

impl Rule {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            allowed_crates: config.allowed_crates.clone(),
            allow_in_tests: config.allow_in_tests,
        }
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "glob-imports"
    }

    fn check_file(&self, syntax: &syn::File, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = GlobVisitor {
            level: self.level,
            allowed_crates: &self.allowed_crates,
            allow_in_tests: self.allow_in_tests,
            in_cfg_test: false,
            file,
            diagnostics: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

struct GlobVisitor<'a> {
    level: RuleLevel,
    allowed_crates: &'a [String],
    allow_in_tests: bool,
    in_cfg_test: bool,
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
}

impl GlobVisitor<'_> {
    fn check_use_tree(&mut self, tree: &UseTree, prefix: &str, line: usize) {
        match tree {
            UseTree::Glob(_) => {
                let full_path = if prefix.is_empty() {
                    "*".to_string()
                } else {
                    format!("{prefix}::*")
                };

                if self.allow_in_tests && self.in_cfg_test {
                    return;
                }

                if self.is_allowed(prefix) {
                    return;
                }

                self.diagnostics.push(
                    Diagnostic::new(
                        "glob-imports",
                        self.level,
                        format!(
                            "glob import `use {full_path}` makes it hard to track symbol origins"
                        ),
                        self.file,
                    )
                    .with_line(line),
                );
            }
            UseTree::Path(use_path) => {
                let segment = use_path.ident.to_string();
                let new_prefix = if prefix.is_empty() {
                    segment
                } else {
                    format!("{prefix}::{segment}")
                };
                self.check_use_tree(&use_path.tree, &new_prefix, line);
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.check_use_tree(item, prefix, line);
                }
            }
            UseTree::Name(_) | UseTree::Rename(_) => {}
        }
    }

    fn is_allowed(&self, prefix: &str) -> bool {
        self.allowed_crates
            .iter()
            .any(|allowed| prefix == allowed || prefix.starts_with(&format!("{allowed}::")))
    }
}

fn has_cfg_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("test") {
                found = true;
            }
            Ok(())
        });
        found
    })
}

impl<'ast> Visit<'ast> for GlobVisitor<'_> {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let line = node.use_token.span.start().line;
        self.check_use_tree(&node.tree, "", line);
        syn::visit::visit_item_use(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let was_in_cfg_test = self.in_cfg_test;
        if has_cfg_test_attr(&node.attrs) {
            self.in_cfg_test = true;
        }
        syn::visit::visit_item_mod(self, node);
        self.in_cfg_test = was_in_cfg_test;
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn parse_and_check(code: &str) -> Vec<Diagnostic> {
        parse_and_check_with_config(code, &Config::default())
    }

    fn parse_and_check_with_config(code: &str, config: &Config) -> Vec<Diagnostic> {
        let syntax = syn::parse_file(code).expect("failed to parse test code");
        let rule = Rule::new(config);
        rule.check_file(&syntax, Path::new("test.rs"))
    }

    #[test]
    fn test_basic_glob_detected() {
        let diags = parse_and_check("use std::collections::*;");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("std::collections::*"));
    }

    #[test]
    fn test_non_glob_passes() {
        let diags = parse_and_check("use std::collections::HashMap;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_nested_glob_in_group() {
        let diags = parse_and_check("use std::{io::*, fmt};");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("std::io::*"));
    }

    #[test]
    fn test_multiple_globs_in_group() {
        let diags = parse_and_check("use std::{io::*, fmt::*};");
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn test_allowlist_exact_match() {
        let config = Config {
            allowed_crates: vec!["std::prelude".to_string()],
            ..Config::default()
        };
        let diags = parse_and_check_with_config("use std::prelude::*;", &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allowlist_prefix_match() {
        let config = Config {
            allowed_crates: vec!["std".to_string()],
            ..Config::default()
        };
        let diags = parse_and_check_with_config("use std::collections::*;", &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_allowlist_no_match() {
        let config = Config {
            allowed_crates: vec!["tokio".to_string()],
            ..Config::default()
        };
        let diags = parse_and_check_with_config("use std::collections::*;", &config);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_allow_in_tests_cfg_test_module() {
        let code = r"
use std::collections::*;

#[cfg(test)]
mod tests {
    use std::fmt::*;
}
";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("std::collections::*"));
    }

    #[test]
    fn test_allow_in_tests_disabled() {
        let config = Config {
            allow_in_tests: false,
            ..Config::default()
        };
        let code = r"
use std::collections::*;

#[cfg(test)]
mod tests {
    use std::fmt::*;
}
";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn test_deny_level_propagated() {
        let config = Config {
            level: RuleLevel::Deny,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("use std::collections::*;", &config);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_rename_import_not_flagged() {
        let diags = parse_and_check("use std::collections::HashMap as Map;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_deeply_nested_glob() {
        let diags = parse_and_check("use a::b::c::d::*;");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("a::b::c::d::*"));
    }

    #[test]
    fn test_cfg_test_scope_restored() {
        let code = r"
#[cfg(test)]
mod tests {
    use std::fmt::*;
}

mod production {
    use std::io::*;
}
";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("std::io::*"));
    }
}
