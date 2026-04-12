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
        "verbose-result-handling"
    }

    fn check_file(&self, syntax: &syn::File, _content: &str, file: &Path) -> Vec<Diagnostic> {
        let mut visitor = VerboseMatchVisitor {
            level: self.level,
            file,
            diagnostics: Vec::new(),
        };
        visitor.visit_file(syntax);
        visitor.diagnostics
    }
}

struct VerboseMatchVisitor<'a> {
    level: RuleLevel,
    file: &'a Path,
    diagnostics: Vec<Diagnostic>,
}

impl VerboseMatchVisitor<'_> {
    fn emit(&mut self, line: usize, message: &str) {
        self.diagnostics.push(
            Diagnostic::new(
                "verbose-result-handling",
                self.level,
                message.to_string(),
                self.file,
            )
            .with_line(line),
        );
    }
}

impl<'ast> Visit<'ast> for VerboseMatchVisitor<'_> {
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        if let Some(message) = detect_simplification(node) {
            let line = node.match_token.span.start().line;
            self.emit(line, message);
        }
        syn::visit::visit_expr_match(self, node);
    }
}

// --- Top-level dispatch ---

fn detect_simplification(node: &syn::ExprMatch) -> Option<&'static str> {
    if node.arms.len() != 2 {
        return None;
    }
    if node.arms.iter().any(|a| a.guard.is_some()) {
        return None;
    }
    if is_try_operator_pattern(&node.arms) {
        return Some("`match` on `Result` can be replaced with the `?` operator");
    }
    if is_if_let_pattern(&node.arms) {
        return Some("`match` with an empty branch can be replaced with `if let`");
    }
    if is_map_combinator_pattern(&node.arms) {
        return Some("`match` that wraps a transformation can be replaced with `.map()`");
    }
    None
}

fn is_try_operator_pattern(arms: &[syn::Arm]) -> bool {
    let ok_arm = arms.iter().find(|a| is_ok_identity_arm(a));
    let err_arm = arms.iter().find(|a| is_err_return_arm(a));
    ok_arm.is_some() && err_arm.is_some()
}

fn is_ok_identity_arm(arm: &syn::Arm) -> bool {
    let Some(bound) = extract_tuple_variant_binding(&arm.pat, "Ok") else {
        return false;
    };
    let body = unwrap_block(&arm.body);
    path_is_ident(body, bound)
}

fn is_err_return_arm(arm: &syn::Arm) -> bool {
    let Some(bound) = extract_tuple_variant_binding(&arm.pat, "Err") else {
        return false;
    };
    let body = unwrap_block(&arm.body);
    let syn::Expr::Return(ret) = body else {
        return false;
    };
    let Some(ret_expr) = ret.expr.as_deref() else {
        return false;
    };
    expr_is_err_wrapping_ident_or_into(ret_expr, bound)
}

fn expr_is_err_wrapping_ident_or_into(expr: &syn::Expr, bound: &syn::Ident) -> bool {
    let syn::Expr::Call(call) = expr else {
        return false;
    };
    if !expr_is_path_named(&call.func, "Err") || call.args.len() != 1 {
        return false;
    }
    let arg = &call.args[0];
    if path_is_ident(arg, bound) {
        return true;
    }
    // Accept `e.into()` as well
    if let syn::Expr::MethodCall(mc) = arg
        && mc.method == "into"
        && mc.args.is_empty()
        && path_is_ident(&mc.receiver, bound)
    {
        return true;
    }
    false
}

fn is_if_let_pattern(arms: &[syn::Arm]) -> bool {
    let has_substantive = arms.iter().any(is_substantive_binding_arm);
    let has_empty = arms.iter().any(is_trivial_none_or_err_arm);
    has_substantive && has_empty
}

fn is_substantive_binding_arm(arm: &syn::Arm) -> bool {
    let is_some = extract_tuple_variant_binding(&arm.pat, "Some").is_some();
    let is_ok = extract_tuple_variant_binding(&arm.pat, "Ok").is_some();
    if !is_some && !is_ok {
        return false;
    }
    !is_empty_body(&arm.body)
}

fn is_trivial_none_or_err_arm(arm: &syn::Arm) -> bool {
    let is_none = is_none_pattern(&arm.pat);
    let is_err_ignored = extract_tuple_variant_binding(&arm.pat, "Err").is_some()
        || is_err_wildcard_pattern(&arm.pat);
    if !is_none && !is_err_ignored {
        return false;
    }
    is_empty_body(&arm.body)
}

fn is_none_pattern(pat: &syn::Pat) -> bool {
    match pat {
        syn::Pat::Ident(pi) => pi.ident == "None",
        syn::Pat::Path(pp) => path_last_segment_equals(&pp.path, "None"),
        _ => false,
    }
}

fn is_err_wildcard_pattern(pat: &syn::Pat) -> bool {
    let syn::Pat::TupleStruct(ts) = pat else {
        return false;
    };
    if !path_last_segment_equals(&ts.path, "Err") {
        return false;
    }
    ts.elems.len() == 1 && matches!(ts.elems[0], syn::Pat::Wild(_))
}

fn is_empty_body(body: &syn::Expr) -> bool {
    match body {
        syn::Expr::Block(eb) => eb.block.stmts.is_empty(),
        syn::Expr::Tuple(et) => et.elems.is_empty(),
        _ => false,
    }
}

fn is_map_combinator_pattern(arms: &[syn::Arm]) -> bool {
    let some_map =
        arms.iter().any(|a| is_wrap_arm(a, "Some")) && arms.iter().any(is_none_passthrough_arm);
    let ok_map =
        arms.iter().any(|a| is_wrap_arm(a, "Ok")) && arms.iter().any(is_err_passthrough_arm);
    some_map || ok_map
}

fn is_wrap_arm(arm: &syn::Arm, variant: &str) -> bool {
    if extract_tuple_variant_binding(&arm.pat, variant).is_none() {
        return false;
    }
    let body = unwrap_block(&arm.body);
    let syn::Expr::Call(call) = body else {
        return false;
    };
    expr_is_path_named(&call.func, variant) && call.args.len() == 1
}

fn is_none_passthrough_arm(arm: &syn::Arm) -> bool {
    if !is_none_pattern(&arm.pat) {
        return false;
    }
    let body = unwrap_block(&arm.body);
    expr_is_path_named(body, "None")
}

fn is_err_passthrough_arm(arm: &syn::Arm) -> bool {
    let Some(bound) = extract_tuple_variant_binding(&arm.pat, "Err") else {
        return false;
    };
    let body = unwrap_block(&arm.body);
    let syn::Expr::Call(call) = body else {
        return false;
    };
    if !expr_is_path_named(&call.func, "Err") || call.args.len() != 1 {
        return false;
    }
    path_is_ident(&call.args[0], bound)
}

// --- Pattern / expression helpers ---

fn extract_tuple_variant_binding<'a>(pat: &'a syn::Pat, variant: &str) -> Option<&'a syn::Ident> {
    let syn::Pat::TupleStruct(ts) = pat else {
        return None;
    };
    if !path_last_segment_equals(&ts.path, variant) || ts.elems.len() != 1 {
        return None;
    }
    let syn::Pat::Ident(pi) = &ts.elems[0] else {
        return None;
    };
    Some(&pi.ident)
}

fn path_last_segment_equals(path: &syn::Path, name: &str) -> bool {
    path.segments.last().is_some_and(|s| s.ident == name)
}

fn expr_is_path_named(expr: &syn::Expr, name: &str) -> bool {
    let syn::Expr::Path(ep) = expr else {
        return false;
    };
    path_last_segment_equals(&ep.path, name)
}

fn path_is_ident(expr: &syn::Expr, ident: &syn::Ident) -> bool {
    let syn::Expr::Path(ep) = expr else {
        return false;
    };
    ep.path.get_ident().is_some_and(|i| i == ident)
}

fn unwrap_block(expr: &syn::Expr) -> &syn::Expr {
    if let syn::Expr::Block(eb) = expr
        && eb.block.stmts.len() == 1
        && let syn::Stmt::Expr(inner, None) = &eb.block.stmts[0]
    {
        return inner;
    }
    expr
}

#[cfg(test)]
#[path = "verbose_result_handling_tests.rs"]
mod tests;
