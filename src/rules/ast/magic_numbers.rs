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
        if let Some((_, expr)) = &node.discriminant {
            let was = self.in_enum_discriminant;
            self.in_enum_discriminant = true;
            self.visit_expr(expr);
            self.in_enum_discriminant = was;
        }
        for field in &node.fields {
            self.visit_field(field);
        }
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
#[path = "magic_numbers_tests.rs"]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
