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
    pub check_format: bool,
    pub check_concat: bool,
    pub check_to_string: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            check_format: true,
            check_concat: true,
            check_to_string: true,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub check_format: Option<bool>,
    pub check_concat: Option<bool>,
    pub check_to_string: Option<bool>,
}

pub const fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.check_format {
        cfg.check_format = v;
    }
    if let Some(v) = o.check_concat {
        cfg.check_concat = v;
    }
    if let Some(v) = o.check_to_string {
        cfg.check_to_string = v;
    }
}

/// Grouped check flags to avoid the more-than-3-bools clippy lint in the visitor.
#[derive(Clone, Copy)]
struct CheckFlags {
    format: bool,
    concat: bool,
    to_string: bool,
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    flags: CheckFlags,
}

impl Rule {
    pub const fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            flags: CheckFlags {
                format: config.check_format,
                concat: config.check_concat,
                to_string: config.check_to_string,
            },
        }
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "string-alloc-in-loop"
    }

    fn check_file(&self, syntax: &syn::File, _content: &str, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = StringAllocVisitor {
            level: self.level,
            flags: self.flags,
            file,
            diagnostics: Vec::new(),
            in_loop: false,
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

struct StringAllocVisitor<'a> {
    level: RuleLevel,
    flags: CheckFlags,
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
    in_loop: bool,
}

fn is_macro_named(mac: &syn::ExprMacro, name: &str) -> bool {
    mac.mac
        .path
        .segments
        .last()
        .is_some_and(|s| s.ident == name)
}

const fn is_string_concat_op(op: syn::BinOp) -> bool {
    matches!(op, syn::BinOp::Add(_) | syn::BinOp::AddAssign(_))
}

const fn rhs_is_reference(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Reference(_))
}

impl StringAllocVisitor<'_> {
    fn emit(&mut self, line: usize, message: String) {
        self.diagnostics.push(
            Diagnostic::new("string-alloc-in-loop", self.level, message, self.file).with_line(line),
        );
    }
}

impl<'ast> Visit<'ast> for StringAllocVisitor<'_> {
    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_for_loop(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_while(&mut self, node: &'ast syn::ExprWhile) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_while(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_loop(&mut self, node: &'ast syn::ExprLoop) {
        let was_in_loop = self.in_loop;
        self.in_loop = true;
        syn::visit::visit_expr_loop(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let was_in_loop = self.in_loop;
        self.in_loop = false;
        syn::visit::visit_expr_closure(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let was_in_loop = self.in_loop;
        self.in_loop = false;
        syn::visit::visit_item_fn(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let was_in_loop = self.in_loop;
        self.in_loop = false;
        syn::visit::visit_impl_item_fn(self, node);
        self.in_loop = was_in_loop;
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if self.in_loop && self.flags.format && is_macro_named(node, "format") {
            let line = node
                .mac
                .path
                .segments
                .last()
                .map_or(1, |s| s.ident.span().start().line);
            self.emit(
                line,
                "`format!()` inside loop allocates a new String each iteration; \
                 consider `write!()` into a pre-allocated `String`"
                    .to_string(),
            );
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_loop
            && self.flags.to_string
            && node.method == "to_string"
            && node.args.is_empty()
        {
            let line = node.method.span().start().line;
            self.emit(
                line,
                "`.to_string()` inside loop allocates each iteration; \
                 consider hoisting it out or using `write!()`"
                    .to_string(),
            );
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if self.in_loop
            && self.flags.concat
            && is_string_concat_op(node.op)
            && rhs_is_reference(&node.right)
        {
            let line = match node.op {
                syn::BinOp::Add(t) => t.span.start().line,
                syn::BinOp::AddAssign(t) => t.spans[0].start().line,
                _ => 1,
            };
            self.emit(
                line,
                "string concatenation inside loop reallocates each iteration; \
                 consider `String::with_capacity()` + `push_str()`"
                    .to_string(),
            );
        }
        syn::visit::visit_expr_binary(self, node);
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
        rule.check_file(&syntax, code, Path::new("test.rs"))
    }

    #[test]
    fn test_format_in_for_loop_flagged() {
        let code = r#"fn f() { for i in 0..10 { let _ = format!("{}", i); } }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("`format!()`"));
    }

    #[test]
    fn test_format_in_while_loop_flagged() {
        let code =
            r#"fn f() { let mut i = 0; while i < 10 { let _ = format!("{}", i); i += 1; } }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_format_in_loop_flagged() {
        let code = r#"fn f() { let mut i = 0; loop { if i == 5 { break; } let _ = format!("{}", i); i += 1; } }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_format_outside_loop_not_flagged() {
        let code = r#"fn f() { let _ = format!("hello"); }"#;
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "format! outside loop: {diags:?}");
    }

    #[test]
    fn test_to_string_in_loop_flagged() {
        let code = "fn f() { for i in 0..10 { let _ = i.to_string(); } }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("`.to_string()`"));
    }

    #[test]
    fn test_to_string_outside_loop_not_flagged() {
        let code = "fn f() { let i = 5; let _ = i.to_string(); }";
        let diags = parse_and_check(code);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_concat_in_loop_flagged() {
        let code = r#"fn f() { let mut s = String::new(); let t = String::from("x"); for _ in 0..10 { s = s + &t; } }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("string concatenation"));
    }

    #[test]
    fn test_add_assign_in_loop_flagged() {
        let code =
            r#"fn f() { let mut s = String::new(); for _ in 0..10 { s += &String::from("x"); } }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_numeric_add_in_loop_not_flagged() {
        let code = "fn f() { let mut sum = 0; for i in 0..10 { sum = sum + i; } }";
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "numeric +: {diags:?}");
    }

    #[test]
    fn test_concat_outside_loop_not_flagged() {
        let code =
            r#"fn f() { let s = String::from("a"); let t = String::from("b"); let _ = s + &t; }"#;
        let diags = parse_and_check(code);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_println_in_loop_not_flagged() {
        let code = r#"fn f() { for i in 0..3 { println!("{}", i); } }"#;
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "println! is not flagged: {diags:?}");
    }

    #[test]
    fn test_closure_in_loop_resets_context() {
        let code = r#"fn f() { for _ in 0..3 { let g = || format!("x"); let _ = g(); } }"#;
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "closure resets in_loop: {diags:?}");
    }

    #[test]
    fn test_inner_fn_in_loop_resets_context() {
        let code = r#"fn f() { for _ in 0..3 { fn helper() -> String { format!("x") } let _ = helper(); } }"#;
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "inner fn resets in_loop: {diags:?}");
    }

    #[test]
    fn test_check_format_disabled() {
        let config = Config {
            check_format: false,
            ..Config::default()
        };
        let code = r#"fn f() { for _ in 0..3 { let _ = format!("x"); } }"#;
        let diags = parse_and_check_with_config(code, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_check_concat_disabled() {
        let config = Config {
            check_concat: false,
            ..Config::default()
        };
        let code =
            r#"fn f() { let mut s = String::new(); for _ in 0..3 { s += &String::from("x"); } }"#;
        let diags = parse_and_check_with_config(code, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_check_to_string_disabled() {
        let config = Config {
            check_to_string: false,
            ..Config::default()
        };
        let code = "fn f() { for i in 0..3 { let _ = i.to_string(); } }";
        let diags = parse_and_check_with_config(code, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn test_deny_level_propagated() {
        let config = Config {
            level: RuleLevel::Deny,
            ..Config::default()
        };
        let code = r#"fn f() { for _ in 0..3 { let _ = format!("x"); } }"#;
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_nested_loop_format_flagged_once() {
        let code = r#"fn f() { for _ in 0..3 { for _ in 0..3 { let _ = format!("x"); } } }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_iterator_map_with_format_not_flagged() {
        let code = r#"fn f() { let _: Vec<String> = (0..3).map(|i| format!("{}", i)).collect(); }"#;
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "iterator .map() closure is not a loop: {diags:?}"
        );
    }
}
