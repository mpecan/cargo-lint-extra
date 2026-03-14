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
    pub max_clones_per_fn: usize,
    pub max_clone_ratio: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            max_clones_per_fn: 5,
            max_clone_ratio: 0.1,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub max_clones_per_fn: Option<usize>,
    pub max_clone_ratio: Option<f64>,
}

pub const fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.max_clones_per_fn {
        cfg.max_clones_per_fn = v;
    }
    if let Some(v) = o.max_clone_ratio {
        cfg.max_clone_ratio = v;
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    max_clones_per_fn: usize,
    max_clone_ratio: f64,
}

/// Backward-compatible alias.
pub type CloneDensityRule = Rule;
/// Backward-compatible alias.
pub type CloneDensityConfig = Config;

impl Rule {
    pub const fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            max_clones_per_fn: config.max_clones_per_fn,
            max_clone_ratio: config.max_clone_ratio,
        }
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "clone-density"
    }

    fn check_file(&self, syntax: &syn::File, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = FileVisitor {
            level: self.level,
            max_clones_per_fn: self.max_clones_per_fn,
            max_clone_ratio: self.max_clone_ratio,
            file,
            diagnostics: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

struct FileVisitor<'a> {
    level: RuleLevel,
    max_clones_per_fn: usize,
    max_clone_ratio: f64,
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
}

impl FileVisitor<'_> {
    fn check_function(&mut self, name: &str, line: usize, block: &syn::Block) {
        // Skip tiny functions where ratio is meaningless
        const MIN_STATEMENTS_FOR_RATIO: usize = 10;

        let mut counter = CloneCounter::default();
        counter.visit_block(block);

        if counter.statement_count == 0 {
            return;
        }

        let exceeds_count = counter.clone_count > self.max_clones_per_fn;

        #[allow(clippy::cast_precision_loss)]
        let ratio = counter.clone_count as f64 / counter.statement_count as f64;
        let exceeds_ratio =
            counter.statement_count >= MIN_STATEMENTS_FOR_RATIO && ratio > self.max_clone_ratio;

        if exceeds_count || exceeds_ratio {
            self.diagnostics.push(
                Diagnostic::new(
                    "clone-density",
                    self.level,
                    format!(
                        "function `{name}` has {} .clone() calls in {} statements \
                         (ratio: {ratio:.3}, limit: {} calls or {:.2} ratio)",
                        counter.clone_count,
                        counter.statement_count,
                        self.max_clones_per_fn,
                        self.max_clone_ratio,
                    ),
                    self.file,
                )
                .with_line(line),
            );
        }
    }
}

impl<'ast> Visit<'ast> for FileVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        let line = node.sig.fn_token.span.start().line;
        self.check_function(&name, line, &node.block);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        let line = node.sig.fn_token.span.start().line;
        self.check_function(&name, line, &node.block);
        syn::visit::visit_impl_item_fn(self, node);
    }
}

#[derive(Default)]
struct CloneCounter {
    clone_count: usize,
    statement_count: usize,
}

impl<'ast> Visit<'ast> for CloneCounter {
    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        // Don't count inner item declarations as statements
        if matches!(node, syn::Stmt::Item(_)) {
            return;
        }
        self.statement_count += 1;
        syn::visit::visit_stmt(self, node);
    }

    // Stop traversal into nested functions/items so their clones
    // aren't attributed to the enclosing function.
    fn visit_item(&mut self, _node: &'ast syn::Item) {}

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "clone" && node.args.is_empty() {
            self.clone_count += 1;
        }
        syn::visit::visit_expr_method_call(self, node);
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
    fn test_no_clones() {
        let diags = parse_and_check("fn main() { let x = 1; let y = 2; }");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_few_clones_under_threshold() {
        // 2 clones in many statements — under both count and ratio thresholds
        let code = "\
fn main() {
    let a = String::new();
    let b = a.clone();
    let c = a.clone();
    let _d = 1;
    let _e = 2;
    let _f = 3;
    let _g = 4;
    let _h = 5;
    let _i = 6;
    let _j = 7;
    let _k = 8;
    let _l = 9;
    let _m = 10;
    let _n = 11;
    let _o = 12;
    let _p = 13;
    let _q = 14;
    let _r = 15;
    let _s = 16;
    let _t = 17;
}
";
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "2 clones in 21 statements should be fine");
    }

    #[test]
    fn test_many_clones_exceeds_count() {
        let code = "\
fn cloney() {
    let a = String::new();
    let _b = a.clone();
    let _c = a.clone();
    let _d = a.clone();
    let _e = a.clone();
    let _f = a.clone();
    let _g = a.clone();
}
";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("cloney"));
        assert!(
            diags[0]
                .message
                .contains("6 .clone() calls in 7 statements")
        );
    }

    #[test]
    fn test_high_ratio_exceeds_threshold() {
        let config = Config {
            level: RuleLevel::Warn,
            max_clones_per_fn: 100,
            max_clone_ratio: 0.1,
        };
        // Need >= 10 statements for ratio check to apply
        let code = "\
fn ratio_heavy() {
    let a = String::new();
    let _b = a.clone();
    let _c = a.clone();
    let _d = 1;
    let _e = 2;
    let _f = 3;
    let _g = 4;
    let _h = 5;
    let _i = 6;
    let _j = 7;
}
";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("ratio"));
    }

    #[test]
    fn test_ratio_skipped_for_small_functions() {
        // Small functions (< 10 statements) should not trigger ratio check
        let config = Config {
            level: RuleLevel::Warn,
            max_clones_per_fn: 100,
            max_clone_ratio: 0.1,
        };
        let code = "\
fn small_fn() {
    let a = String::new();
    let _b = a.clone();
}
";
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "small functions should not trigger ratio check"
        );
    }

    #[test]
    fn test_impl_method_checked() {
        let code = "\
struct Foo;
impl Foo {
    fn cloney(&self) {
        let a = String::new();
        let _b = a.clone();
        let _c = a.clone();
        let _d = a.clone();
        let _e = a.clone();
        let _f = a.clone();
        let _g = a.clone();
    }
}
";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("cloney"));
    }

    #[test]
    fn test_empty_function() {
        let diags = parse_and_check("fn empty() {}");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_deny_level_propagated() {
        let config = Config {
            level: RuleLevel::Deny,
            ..Config::default()
        };
        let code = "\
fn cloney() {
    let a = String::new();
    let _b = a.clone();
    let _c = a.clone();
    let _d = a.clone();
    let _e = a.clone();
    let _f = a.clone();
    let _g = a.clone();
}
";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_ratio_boundary_9_statements() {
        // 9 statements: below MIN_STATEMENTS_FOR_RATIO, ratio check skipped
        let config = Config {
            level: RuleLevel::Warn,
            max_clones_per_fn: 100,
            max_clone_ratio: 0.1,
        };
        let code = "\
fn boundary_9() {
    let a = String::new();
    let _b = a.clone();
    let _c = a.clone();
    let _d = 1;
    let _e = 2;
    let _f = 3;
    let _g = 4;
    let _h = 5;
    let _i = 6;
}
";
        let diags = parse_and_check_with_config(code, &config);
        assert!(diags.is_empty(), "9 statements should skip ratio check");
    }

    #[test]
    fn test_ratio_boundary_10_statements() {
        // 10 statements: at MIN_STATEMENTS_FOR_RATIO, ratio check applies
        let config = Config {
            level: RuleLevel::Warn,
            max_clones_per_fn: 100,
            max_clone_ratio: 0.1,
        };
        let code = "\
fn boundary_10() {
    let a = String::new();
    let _b = a.clone();
    let _c = a.clone();
    let _d = 1;
    let _e = 2;
    let _f = 3;
    let _g = 4;
    let _h = 5;
    let _i = 6;
    let _j = 7;
}
";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1, "10 statements should apply ratio check");
    }

    #[test]
    fn test_closure_clones_counted_in_parent() {
        // Clones inside closures are counted as part of the parent function
        let code = "\
fn with_closure() {
    let a = String::new();
    let _f = || {
        let _b = a.clone();
        let _c = a.clone();
        let _d = a.clone();
        let _e = a.clone();
        let _g = a.clone();
        let _h = a.clone();
    };
}
";
        let diags = parse_and_check(code);
        assert_eq!(
            diags.len(),
            1,
            "closure clones should count toward parent function"
        );
    }

    #[test]
    fn test_nested_fn_not_counted_in_parent() {
        // Clones inside a nested fn should NOT count toward the outer function
        let code = "\
fn outer() {
    let a = String::new();
    let _b = a.clone();
    fn inner() {
        let c = String::new();
        let _d = c.clone();
        let _e = c.clone();
        let _f = c.clone();
        let _g = c.clone();
        let _h = c.clone();
        let _i = c.clone();
    }
}
";
        let diags = parse_and_check(code);
        // outer has 2 statements and 1 clone — under threshold
        // inner has 7 statements and 6 clones — over threshold, but checked separately
        // via FileVisitor::visit_item_fn
        assert_eq!(diags.len(), 1, "only inner fn should trigger");
        assert!(diags[0].message.contains("inner"));
    }

    #[test]
    fn test_clone_with_args_not_counted() {
        let code = "\
fn custom_clone() {
    let a = String::new();
    let _b = a.clone_from(&String::new());
    let _c = a.clone_into();
}
";
        let diags = parse_and_check(code);
        assert!(diags.is_empty());
    }
}
