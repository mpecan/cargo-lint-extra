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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
}

pub const fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
}

impl Rule {
    pub const fn new(config: &Config) -> Self {
        Self {
            level: config.level,
        }
    }
}

impl AstRule for Rule {
    fn name(&self) -> &'static str {
        "collect-then-iterate"
    }

    fn check_file(&self, syntax: &syn::File, _content: &str, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = CollectIterVisitor {
            level: self.level,
            file,
            diagnostics: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

struct CollectIterVisitor<'a> {
    level: RuleLevel,
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
}

fn suggestion_for_method(method: &str) -> Option<&'static str> {
    match method {
        "iter" | "into_iter" | "iter_mut" => Some("continue using the iterator chain directly"),
        "len" => Some("use `.count()` instead"),
        "is_empty" => Some("use `.next().is_none()` instead"),
        "first" => Some("use `.next()` instead"),
        "last" => Some("use `.last()` on the iterator instead"),
        _ => None,
    }
}

fn is_collect_call(expr: &syn::Expr) -> bool {
    if let syn::Expr::MethodCall(call) = expr {
        call.method == "collect"
    } else {
        false
    }
}

impl CollectIterVisitor<'_> {
    fn emit(&mut self, line: usize, method: &str, suggestion: &str) {
        let message = format!("`.collect().{method}()` allocates unnecessarily; {suggestion}",);
        self.diagnostics.push(
            Diagnostic::new("collect-then-iterate", self.level, message, self.file).with_line(line),
        );
    }
}

impl<'ast> Visit<'ast> for CollectIterVisitor<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        if let Some(suggestion) = suggestion_for_method(&method)
            && is_collect_call(&node.receiver)
        {
            let line = node.method.span().start().line;
            self.emit(line, &method, suggestion);
        }
        syn::visit::visit_expr_method_call(self, node);
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
            },
        )
    }

    fn parse_and_check_with_config(code: &str, config: &Config) -> Vec<Diagnostic> {
        let syntax = syn::parse_file(code).expect("failed to parse test code");
        let rule = Rule::new(config);
        rule.check_file(&syntax, code, Path::new("test.rs"))
    }

    #[test]
    fn test_collect_iter_detected() {
        let code = "fn f() { let _: Vec<i32> = vec![1,2,3].into_iter().collect::<Vec<_>>().iter().count(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".collect().iter()"));
        assert!(diags[0].message.contains("iterator chain directly"));
    }

    #[test]
    fn test_collect_into_iter_detected() {
        let code = "fn f() { let _v: Vec<i32> = vec![1].into_iter().collect::<Vec<_>>().into_iter().count(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("into_iter"));
    }

    #[test]
    fn test_collect_iter_mut_detected() {
        let code = "fn f() { let mut v: Vec<i32> = vec![1].into_iter().collect::<Vec<_>>(); v.iter_mut(); }";
        // This should NOT flag because `v.iter_mut()` is on a variable, not chained on collect
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "variable access should not be flagged: {diags:?}"
        );
    }

    #[test]
    fn test_collect_iter_mut_chained_detected() {
        let code =
            "fn f() { let _ = vec![1i32].into_iter().collect::<Vec<_>>().iter_mut().count(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("iter_mut"));
    }

    #[test]
    fn test_collect_len_detected() {
        let code = "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().len(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".count()"));
    }

    #[test]
    fn test_collect_is_empty_detected() {
        let code = "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().is_empty(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".next().is_none()"));
    }

    #[test]
    fn test_collect_first_detected() {
        let code = "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().first(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".next()"));
    }

    #[test]
    fn test_collect_last_detected() {
        let code = "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().last(); }";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("`.last()` on the iterator"));
    }

    #[test]
    fn test_plain_collect_without_turbofish() {
        let code = r"
fn f() {
    let v: Vec<i32> = vec![1, 2, 3];
    let _ = v.iter().collect::<Vec<_>>().len();
}
";
        let diags = parse_and_check(code);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains(".count()"));
    }

    #[test]
    fn test_collect_then_push_not_flagged() {
        let code =
            "fn f() { let mut v: Vec<i32> = vec![1].into_iter().collect::<Vec<_>>(); v.push(2); }";
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "push is not a flagged method: {diags:?}");
    }

    #[test]
    fn test_collect_then_unflagged_method_not_flagged() {
        let code = "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().contains(&1); }";
        let diags = parse_and_check(code);
        assert!(
            diags.is_empty(),
            "contains is not a flagged method: {diags:?}"
        );
    }

    #[test]
    fn test_no_collect_not_flagged() {
        let code = "fn f() { let v = vec![1,2,3]; let _ = v.iter().count(); }";
        let diags = parse_and_check(code);
        assert!(diags.is_empty(), "no collect call: {diags:?}");
    }

    #[test]
    fn test_deny_level_propagated() {
        let config = Config {
            level: RuleLevel::Deny,
        };
        let code = "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().len(); }";
        let diags = parse_and_check_with_config(code, &config);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].level, RuleLevel::Deny);
    }

    #[test]
    fn test_intermediate_method_not_flagged() {
        let code =
            "fn f() { let _ = vec![1,2,3].into_iter().collect::<Vec<_>>().as_slice().iter(); }";
        let diags = parse_and_check(code);
        // .as_slice() breaks the chain — iter() sees as_slice() as receiver, not collect()
        assert!(
            diags.is_empty(),
            "intermediate method should break detection: {diags:?}"
        );
    }
}
