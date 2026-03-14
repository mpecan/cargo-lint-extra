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
    pub check_unwrap: bool,
    pub check_expect: bool,
    pub check_indexing: bool,
    pub required_comment: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Allow,
            check_unwrap: true,
            check_expect: true,
            check_indexing: false,
            required_comment: "PANIC".to_string(),
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub check_unwrap: Option<bool>,
    pub check_expect: Option<bool>,
    pub check_indexing: Option<bool>,
    pub required_comment: Option<String>,
}

pub fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.check_unwrap {
        cfg.check_unwrap = v;
    }
    if let Some(v) = o.check_expect {
        cfg.check_expect = v;
    }
    if let Some(v) = o.check_indexing {
        cfg.check_indexing = v;
    }
    if let Some(v) = &o.required_comment {
        cfg.required_comment.clone_from(v);
    }
}

/// Grouped check flags to avoid more-than-3-bools lint in the visitor.
struct CheckFlags {
    unwrap: bool,
    expect: bool,
    indexing: bool,
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    flags: CheckFlags,
    required_comment: String,
}

impl Rule {
    pub fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            flags: CheckFlags {
                unwrap: config.check_unwrap,
                expect: config.check_expect,
                indexing: config.check_indexing,
            },
            required_comment: config.required_comment.clone(),
        }
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "undocumented-panic"
    }

    fn check_file(&self, syntax: &syn::File, content: &str, file: &Path) -> Vec<Diagnostic> {
        let lines: Vec<&str> = content.lines().collect();
        let mut visitor = PanicVisitor {
            level: self.level,
            flags: &self.flags,
            required_comment: &self.required_comment,
            lines: &lines,
            file,
            diagnostics: Vec::new(),
            in_test: false,
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

fn has_justification_comment(lines: &[&str], line_number: usize, prefix: &str) -> bool {
    if line_number == 0 || line_number > lines.len() {
        return false;
    }
    let current_line = lines[line_number - 1];
    let pattern = format!("// {prefix}:");
    if current_line.contains(&pattern) {
        return true;
    }
    if line_number >= 2 {
        let prev_line = lines[line_number - 2].trim();
        if prev_line.starts_with(&format!("// {prefix}:")) {
            return true;
        }
    }
    false
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

struct PanicVisitor<'a> {
    level: RuleLevel,
    flags: &'a CheckFlags,
    required_comment: &'a str,
    lines: &'a [&'a str],
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
    in_test: bool,
}

impl PanicVisitor<'_> {
    fn emit(&mut self, line: usize, message: String) {
        self.diagnostics.push(
            Diagnostic::new("undocumented-panic", self.level, message, self.file).with_line(line),
        );
    }
}

impl<'ast> Visit<'ast> for PanicVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if !self.in_test {
            let method = node.method.to_string();
            let line = node.method.span().start().line;
            let no_justification =
                !has_justification_comment(self.lines, line, self.required_comment);

            if self.flags.unwrap && method == "unwrap" && node.args.is_empty() && no_justification {
                let msg = format!(
                    "`.unwrap()` call without `// {}:` justification comment",
                    self.required_comment
                );
                self.emit(line, msg);
            } else if self.flags.expect && method == "expect" && no_justification {
                let msg = format!(
                    "`.expect()` call without `// {}:` justification comment",
                    self.required_comment
                );
                self.emit(line, msg);
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_index(&mut self, node: &'ast syn::ExprIndex) {
        if !self.in_test && self.flags.indexing {
            let line = node.bracket_token.span.open().start().line;
            if !has_justification_comment(self.lines, line, self.required_comment) {
                let msg = format!(
                    "array/slice indexing without `// {}:` justification comment",
                    self.required_comment
                );
                self.emit(line, msg);
            }
        }
        syn::visit::visit_expr_index(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let was_in_test = self.in_test;
        if has_test_attr(&node.attrs) || has_cfg_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_item_fn(self, node);
        self.in_test = was_in_test;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let was_in_test = self.in_test;
        if has_test_attr(&node.attrs) || has_cfg_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_impl_item_fn(self, node);
        self.in_test = was_in_test;
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let was_in_test = self.in_test;
        if has_cfg_test_attr(&node.attrs) {
            self.in_test = true;
        }
        syn::visit::visit_item_mod(self, node);
        self.in_test = was_in_test;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn parse_and_check(code: &str) -> Vec<Diagnostic> {
        parse_and_check_with_config(
            code,
            &Config {
                level: RuleLevel::Warn,
                ..Config::default()
            },
        )
    }

    fn parse_and_check_with_config(code: &str, config: &Config) -> Vec<Diagnostic> {
        let syntax = syn::parse_file(code).expect("failed to parse test code");
        let rule = Rule::new(config);
        rule.check_file(&syntax, code, Path::new("test.rs"))
    }

    #[test]
    fn test_unwrap_detected() {
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".unwrap()"));
    }

    #[test]
    fn test_expect_detected() {
        let code = r#"fn f() { let x: Option<i32> = Some(1); let _v = x.expect("oops"); }"#;
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".expect()"));
    }

    #[test]
    fn test_justified_preceding_line() {
        let code = "fn f() {\n    let x: Option<i32> = Some(1);\n\
                    // PANIC: always Some\n    let _v = x.unwrap();\n}";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "preceding comment should suppress: {diags:?}"
        );
    }

    #[test]
    fn test_justified_inline_comment() {
        let code = "fn f() {\n    let x: Option<i32> = Some(1);\n\
                    let _v = x.unwrap(); // PANIC: always Some\n}";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "inline comment should suppress: {diags:?}"
        );
    }

    #[test]
    fn test_test_function_skipped() {
        let code = "#[test]\nfn my_test() { let x: Option<i32> = Some(1); let _v = x.unwrap(); }";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "test functions should be skipped: {diags:?}"
        );
    }

    #[test]
    fn test_cfg_test_module_skipped() {
        let code = "#[cfg(test)]\nmod tests {\n\
                    fn helper() { let x: Option<i32> = Some(1); let _v = x.unwrap(); }\n}";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "cfg(test) module should be skipped: {diags:?}"
        );
    }

    #[test]
    fn test_indexing_disabled_by_default() {
        let code = "fn f() { let arr = [1, 2, 3]; let _v = arr[0]; }";
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "indexing disabled by default: {diags:?}");
    }

    #[test]
    fn test_indexing_enabled() {
        let code = "fn f() { let arr = [1, 2, 3]; let _v = arr[0]; }";
        let config = Config {
            level: RuleLevel::Warn,
            check_indexing: true,
            ..Config::default()
        };
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("indexing"));
    }

    #[test]
    fn test_custom_prefix() {
        let code = "fn f() {\n    let x: Option<i32> = Some(1);\n    let _v = x.unwrap(); // SAFETY: always Some\n}";
        let config = Config {
            level: RuleLevel::Warn,
            required_comment: "SAFETY".to_string(),
            ..Config::default()
        };
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "custom prefix SAFETY: should suppress: {diags:?}"
        );
    }

    #[test]
    fn test_custom_prefix_wrong_keyword() {
        let code = "fn f() {\n    let x: Option<i32> = Some(1);\n    let _v = x.unwrap(); // PANIC: always Some\n}";
        let config = Config {
            level: RuleLevel::Warn,
            required_comment: "SAFETY".to_string(),
            ..Config::default()
        };
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(
            diags.len(),
            1,
            "PANIC: comment should not satisfy SAFETY: prefix"
        );
    }

    #[test]
    fn test_unwrap_or_not_flagged() {
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap_or(0); }";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "unwrap_or should not be flagged: {diags:?}"
        );
    }

    #[test]
    fn test_unwrap_or_default_not_flagged() {
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap_or_default(); }";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "unwrap_or_default should not be flagged: {diags:?}"
        );
    }

    #[test]
    fn test_unwrap_or_else_not_flagged() {
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap_or_else(|| 0); }";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "unwrap_or_else should not be flagged: {diags:?}"
        );
    }

    #[test]
    fn test_deny_level_propagated() {
        let config = Config {
            level: RuleLevel::Deny,
            ..Config::default()
        };
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap(); }";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_allow_level_disabled() {
        let config = Config {
            level: RuleLevel::Allow,
            ..Config::default()
        };
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap(); }";
        // Engine skips Allow-level rules, but we test the rule directly
        let syntax = syn::parse_file(code).expect("parse");
        let rule = Rule::new(&config);
        let diags = rule.check_file(&syntax, code, Path::new("test.rs"));
        // Rule itself doesn't know about Allow — the engine skips it.
        // But we still verify the diagnostics exist (the engine filters them).
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_check_unwrap_disabled() {
        let config = Config {
            level: RuleLevel::Warn,
            check_unwrap: false,
            ..Config::default()
        };
        let code = "fn f() { let x: Option<i32> = Some(1); let _v = x.unwrap(); }";
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "check_unwrap=false should suppress: {diags:?}"
        );
    }

    #[test]
    fn test_cfg_test_fn_skipped() {
        let code = "\
#[cfg(test)]
fn test_helper() {
    let x: Option<i32> = Some(1);
    let _v = x.unwrap();
}
";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "#[cfg(test)] fn should be skipped: {diags:?}"
        );
    }

    #[test]
    fn test_check_expect_disabled() {
        let config = Config {
            level: RuleLevel::Warn,
            check_expect: false,
            ..Config::default()
        };
        let code = r#"fn f() { let x: Option<i32> = Some(1); let _v = x.expect("msg"); }"#;
        let diags = parse_and_check_with_config(code, &config);
        assert!(
            diags.is_empty(),
            "check_expect=false should suppress: {diags:?}"
        );
    }
}
