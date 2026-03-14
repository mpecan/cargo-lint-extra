use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::AstRule;
use serde::Deserialize;
use std::path::Path;
use syn::visit::Visit;

// --- Config ---
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub level: RuleLevel,
    pub allowed: Vec<String>,
    pub ignore_test: bool,
    pub ignore_const: bool,
    pub ignore_enum: bool,
    pub ignore_range: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Allow,
            allowed: vec![
                "0".to_string(),
                "1".to_string(),
                "2".to_string(),
                "10".to_string(),
                "100".to_string(),
                "1000".to_string(),
            ],
            ignore_test: true,
            ignore_const: true,
            ignore_enum: true,
            ignore_range: true,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub allowed: Option<Vec<String>>,
    pub ignore_test: Option<bool>,
    pub ignore_const: Option<bool>,
    pub ignore_enum: Option<bool>,
    pub ignore_range: Option<bool>,
}

pub fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = &o.allowed {
        cfg.allowed.clone_from(v);
    }
    if let Some(v) = o.ignore_test {
        cfg.ignore_test = v;
    }
    if let Some(v) = o.ignore_const {
        cfg.ignore_const = v;
    }
    if let Some(v) = o.ignore_enum {
        cfg.ignore_enum = v;
    }
    if let Some(v) = o.ignore_range {
        cfg.ignore_range = v;
    }
}

// --- Rule ---
#[allow(clippy::struct_excessive_bools)]
pub struct Rule {
    level: RuleLevel,
    allowed: Vec<String>,
    ignore_test: bool,
    ignore_const: bool,
    ignore_enum: bool,
    ignore_range: bool,
}

impl Rule {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            allowed: config.allowed.clone(),
            ignore_test: config.ignore_test,
            ignore_const: config.ignore_const,
            ignore_enum: config.ignore_enum,
            ignore_range: config.ignore_range,
        }
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "magic-numbers"
    }

    fn check_file(&self, syntax: &syn::File, _content: &str, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = MagicNumberVisitor {
            level: self.level,
            allowed: &self.allowed,
            ignore_test: self.ignore_test,
            ignore_const: self.ignore_const,
            ignore_enum: self.ignore_enum,
            ignore_range: self.ignore_range,
            in_const: false,
            in_enum_discriminant: false,
            in_range: false,
            in_test: false,
            file,
            diagnostics: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

#[allow(clippy::struct_excessive_bools)]
struct MagicNumberVisitor<'a> {
    level: RuleLevel,
    allowed: &'a [String],
    ignore_test: bool,
    ignore_const: bool,
    ignore_enum: bool,
    ignore_range: bool,
    in_const: bool,
    in_enum_discriminant: bool,
    in_range: bool,
    in_test: bool,
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
}

impl MagicNumberVisitor<'_> {
    fn is_allowed(&self, value: &str) -> bool {
        self.allowed.iter().any(|a| a == value)
    }

    const fn should_skip(&self) -> bool {
        (self.ignore_const && self.in_const)
            || (self.ignore_enum && self.in_enum_discriminant)
            || (self.ignore_range && self.in_range)
            || (self.ignore_test && self.in_test)
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

fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("test"))
}

impl<'ast> Visit<'ast> for MagicNumberVisitor<'_> {
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        let was = self.in_const;
        self.in_const = true;
        syn::visit::visit_item_const(self, node);
        self.in_const = was;
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        let was = self.in_const;
        self.in_const = true;
        syn::visit::visit_item_static(self, node);
        self.in_const = was;
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        let was = self.in_const;
        self.in_const = true;
        syn::visit::visit_impl_item_const(self, node);
        self.in_const = was;
    }

    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst) {
        let was = self.in_const;
        self.in_const = true;
        syn::visit::visit_trait_item_const(self, node);
        self.in_const = was;
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        let was = self.in_enum_discriminant;
        if node.discriminant.is_some() {
            self.in_enum_discriminant = true;
        }
        syn::visit::visit_variant(self, node);
        self.in_enum_discriminant = was;
    }

    fn visit_expr_range(&mut self, node: &'ast syn::ExprRange) {
        let was = self.in_range;
        self.in_range = true;
        syn::visit::visit_expr_range(self, node);
        self.in_range = was;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let was = self.in_test;
        if has_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_item_fn(self, node);
        self.in_test = was;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let was = self.in_test;
        if has_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_impl_item_fn(self, node);
        self.in_test = was;
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let was = self.in_test;
        if has_cfg_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_item_mod(self, node);
        self.in_test = was;
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        if !self.should_skip() {
            match &node.lit {
                syn::Lit::Int(lit) => {
                    let value = lit.base10_digits().to_string();
                    if !self.is_allowed(&value) {
                        let line = lit.span().start().line;
                        self.diagnostics.push(
                            Diagnostic::new(
                                "magic-numbers",
                                self.level,
                                format!("magic number `{value}` should be a named constant"),
                                self.file,
                            )
                            .with_line(line),
                        );
                    }
                }
                syn::Lit::Float(lit) => {
                    let value = lit.base10_digits().to_string();
                    if !self.is_allowed(&value) {
                        let line = lit.span().start().line;
                        self.diagnostics.push(
                            Diagnostic::new(
                                "magic-numbers",
                                self.level,
                                format!("magic number `{value}` should be a named constant"),
                                self.file,
                            )
                            .with_line(line),
                        );
                    }
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_lit(self, node);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn parse_and_check(code: &str) -> Vec<Diagnostic> {
        parse_and_check_with_config(code, &Config::default())
    }

    fn parse_and_check_with_config(code: &str, config: &Config) -> Vec<Diagnostic> {
        let syntax = syn::parse_file(code).expect("failed to parse test code");
        let rule = Rule::new(config);
        rule.check_file(&syntax, "", Path::new("test.rs"))
    }

    #[test]
    fn test_allowed_numbers_not_flagged() {
        let code = "fn f() { let a = 0; let b = 1; let c = 2; let d = 10; let e = 100; }";
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "default allowed numbers should not be flagged: {diags:?}"
        );
    }

    #[test]
    fn test_magic_number_flagged() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("fn f() { let x = 42; }", &config);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("42"));
        assert!(diags[0].message.contains("magic number"));
    }

    #[test]
    fn test_const_skipped_by_default() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("const MAX: u32 = 42;", &config);
        assert!(
            diags.is_empty(),
            "numbers in const should be skipped by default"
        );
    }

    #[test]
    fn test_static_skipped_by_default() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("static LIMIT: u64 = 30;", &config);
        assert!(
            diags.is_empty(),
            "numbers in static should be skipped by default"
        );
    }

    #[test]
    fn test_enum_discriminant_skipped_by_default() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let code = "enum Color { Red = 3, Green = 5, Blue = 7, }";
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "enum discriminants should be skipped by default"
        );
    }

    #[test]
    fn test_range_skipped_by_default() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let code = "fn f() { for i in 0..42 { let _ = i; } }";
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "numbers in ranges should be skipped by default"
        );
    }

    #[test]
    fn test_test_fn_skipped_by_default() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let code = "#[test]\nfn my_test() { let x = 999; }";
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "numbers in test functions should be skipped by default"
        );
    }

    #[test]
    fn test_cfg_test_mod_skipped_by_default() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let code = "#[cfg(test)]\nmod tests { fn helper() { let x = 999; } }";
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "numbers in cfg(test) modules should be skipped by default"
        );
    }

    #[test]
    fn test_custom_allowlist() {
        let config = Config {
            level: RuleLevel::Warn,
            allowed: vec!["42".to_string()],
            ..Config::default()
        };
        let code = "fn f() { let x = 42; let y = 99; }";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("99"));
    }

    #[test]
    fn test_underscored_int_literal() {
        // 1_000 should normalize to "1000" and be in the default allowlist
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("fn f() { let x = 1_000; }", &config);
        assert!(
            diags.is_empty(),
            "1_000 should normalize to 1000 and be allowed"
        );
    }

    #[test]
    fn test_float_literal_flagged() {
        let config = Config {
            level: RuleLevel::Warn,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("fn f() { let x = 3.14; }", &config);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("3.14"));
    }

    #[test]
    fn test_deny_level_propagated() {
        let config = Config {
            level: RuleLevel::Deny,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("fn f() { let x = 42; }", &config);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    /// At the unit-test level the rule runs regardless of level; the engine is what
    /// skips Allow-level rules. So we verify the diagnostic level is Allow here.
    #[test]
    fn test_default_level_is_allow() {
        let diags = parse_and_check("fn f() { let x = 42; }");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Allow);
    }

    #[test]
    fn test_ignore_const_disabled() {
        let config = Config {
            level: RuleLevel::Warn,
            ignore_const: false,
            ..Config::default()
        };
        let diags = parse_and_check_with_config("const MAX: u32 = 42;", &config);
        assert_eq!(
            diags.len(),
            1,
            "const numbers should be flagged when ignore_const=false"
        );
    }

    #[test]
    fn test_ignore_range_disabled() {
        let config = Config {
            level: RuleLevel::Warn,
            ignore_range: false,
            ..Config::default()
        };
        let code = "fn f() { for i in 0..42 { let _ = i; } }";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(
            diags.len(),
            1,
            "range numbers should be flagged when ignore_range=false"
        );
        assert!(diags[0].message.contains("42"));
    }
}
