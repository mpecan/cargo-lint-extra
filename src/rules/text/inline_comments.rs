use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use serde::Deserialize;
use std::path::Path;

// --- Config ---
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub level: RuleLevel,
    pub max_ratio: f64,
    pub max_consecutive: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: RuleLevel::Warn,
            max_ratio: 0.3,
            max_consecutive: 3,
        }
    }
}

// --- Test Override ---
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Override {
    pub level: Option<RuleLevel>,
    pub max_ratio: Option<f64>,
    pub max_consecutive: Option<usize>,
}

pub const fn apply_override(cfg: &mut Config, o: &Override) {
    if let Some(v) = o.level {
        cfg.level = v;
    }
    if let Some(v) = o.max_ratio {
        cfg.max_ratio = v;
    }
    if let Some(v) = o.max_consecutive {
        cfg.max_consecutive = v;
    }
}

// --- Rule ---
pub struct Rule {
    level: RuleLevel,
    max_ratio: f64,
    max_consecutive: usize,
}

/// Backward-compatible alias.
pub type InlineCommentsRule = Rule;

impl Rule {
    pub const fn new(config: &Config) -> Self {
        Self {
            level: config.level,
            max_ratio: config.max_ratio,
            max_consecutive: config.max_consecutive,
        }
    }
}

struct FunctionScope {
    fn_line: usize,
    entry_depth: usize,
    comment_lines: usize,
    code_lines: usize,
    consecutive_comments: usize,
    max_consecutive_comments: usize,
    first_long_run_line: Option<usize>,
}

impl FunctionScope {
    const fn new(fn_line: usize, entry_depth: usize) -> Self {
        Self {
            fn_line,
            entry_depth,
            comment_lines: 0,
            code_lines: 0,
            consecutive_comments: 0,
            max_consecutive_comments: 0,
            first_long_run_line: None,
        }
    }

    const fn add_comment(&mut self, line_number: usize, max_consecutive: usize) {
        self.comment_lines += 1;
        self.consecutive_comments += 1;
        if self.consecutive_comments > self.max_consecutive_comments {
            self.max_consecutive_comments = self.consecutive_comments;
            if self.consecutive_comments > max_consecutive && self.first_long_run_line.is_none() {
                self.first_long_run_line = Some(line_number - (self.consecutive_comments - 1));
            }
        }
    }

    const fn add_code(&mut self) {
        self.code_lines += 1;
        self.consecutive_comments = 0;
    }

    const fn reset_consecutive(&mut self) {
        self.consecutive_comments = 0;
    }

    const fn total_meaningful(&self) -> usize {
        self.comment_lines + self.code_lines
    }

    #[allow(clippy::cast_precision_loss)]
    fn comment_ratio(&self) -> f64 {
        let total = self.total_meaningful();
        if total == 0 {
            0.0
        } else {
            self.comment_lines as f64 / total as f64
        }
    }
}

/// What multi-line context the scanner is currently inside (persists across lines).
#[derive(Default)]
enum ScanContext {
    #[default]
    Code,
    BlockComment,
    String {
        escape_next: bool,
    },
    RawString {
        closing_hashes: usize,
    },
}

impl TextRule for Rule {
    fn name(&self) -> &'static str {
        "inline-comments"
    }

    fn check_file(&self, content: &str, file: &Path) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut scope_stack: Vec<FunctionScope> = Vec::new();
        let mut brace_depth: usize = 0;
        let mut pending_fn: Option<usize> = None;
        let mut scan_ctx = ScanContext::default();

        for (idx, line) in content.lines().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            let line_info = analyze_line(trimmed, &mut scan_ctx);

            if line_info.has_fn {
                pending_fn = Some(line_number);
            }

            if line_info.ends_with_semi && pending_fn.is_some() && line_info.opens == 0 {
                pending_fn = None;
            }

            for _ in 0..line_info.opens {
                if let Some(fn_line) = pending_fn.take() {
                    scope_stack.push(FunctionScope::new(fn_line, brace_depth));
                }
                brace_depth += 1;
            }

            if let Some(scope) = scope_stack.last_mut()
                && brace_depth > scope.entry_depth
                && line_info.opens == 0
                && line_info.closes == 0
            {
                classify_line(trimmed, scope, line_number, self.max_consecutive);
            }

            for _ in 0..line_info.closes {
                brace_depth = brace_depth.saturating_sub(1);
                if scope_stack
                    .last()
                    .is_some_and(|s| brace_depth == s.entry_depth)
                    && let Some(scope) = scope_stack.pop()
                {
                    self.evaluate_scope(&scope, file, &mut diagnostics);
                }
            }
        }

        while let Some(scope) = scope_stack.pop() {
            self.evaluate_scope(&scope, file, &mut diagnostics);
        }

        diagnostics
    }
}

impl Rule {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn evaluate_scope(
        &self,
        scope: &FunctionScope,
        file: &Path,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        const MIN_LINES: usize = 4;

        if scope.total_meaningful() < MIN_LINES {
            return;
        }

        let ratio = scope.comment_ratio();
        if ratio > self.max_ratio {
            let pct = (ratio * 100.0).round() as usize;
            let max_pct = (self.max_ratio * 100.0).round() as usize;
            diagnostics.push(
                Diagnostic::new(
                    self.name(),
                    self.level,
                    format!(
                        "function has {pct}% inline comments ({}/{} lines), max allowed is {max_pct}%",
                        scope.comment_lines,
                        scope.total_meaningful()
                    ),
                    file,
                )
                .with_line(scope.fn_line),
            );
        }

        if scope.max_consecutive_comments > self.max_consecutive
            && let Some(run_start) = scope.first_long_run_line
        {
            diagnostics.push(
                Diagnostic::new(
                    self.name(),
                    self.level,
                    format!(
                        "{} consecutive comment lines (max allowed {})",
                        scope.max_consecutive_comments, self.max_consecutive
                    ),
                    file,
                )
                .with_line(run_start),
            );
        }
    }
}

fn classify_line(
    trimmed: &str,
    scope: &mut FunctionScope,
    line_number: usize,
    max_consecutive: usize,
) {
    if trimmed.is_empty() {
        scope.reset_consecutive();
        return;
    }

    if is_inline_comment(trimmed) {
        scope.add_comment(line_number, max_consecutive);
    } else {
        scope.add_code();
    }
}

fn is_inline_comment(trimmed: &str) -> bool {
    trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!")
}

struct LineInfo {
    opens: usize,
    closes: usize,
    has_fn: bool,
    ends_with_semi: bool,
}

/// Analyze a line for brace opens/closes, fn keyword presence, and trailing semicolons.
/// Handles strings (including raw strings), char literals, line comments, and block comments.
/// Cross-line state (block comments, strings) is tracked via `ScanContext`.
#[allow(clippy::too_many_lines)]
fn analyze_line(line: &str, ctx: &mut ScanContext) -> LineInfo {
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut has_fn = false;

    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        match ctx {
            ScanContext::String { escape_next } => {
                if *escape_next {
                    *escape_next = false;
                    i += 1;
                    continue;
                }
                if ch == '\\' {
                    *escape_next = true;
                } else if ch == '"' {
                    *ctx = ScanContext::Code;
                }
                i += 1;
                continue;
            }
            ScanContext::RawString { closing_hashes } => {
                if ch == '"' {
                    let needed = *closing_hashes;
                    let mut hash_count = 0;
                    while i + 1 + hash_count < len && chars[i + 1 + hash_count] == '#' {
                        hash_count += 1;
                    }
                    if hash_count >= needed {
                        *ctx = ScanContext::Code;
                        i += 1 + needed;
                        continue;
                    }
                }
                i += 1;
                continue;
            }
            ScanContext::BlockComment => {
                if ch == '*' && i + 1 < len && chars[i + 1] == '/' {
                    *ctx = ScanContext::Code;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }
            ScanContext::Code => {}
        }

        // Normal code context
        match ch {
            '"' => {
                *ctx = ScanContext::String { escape_next: false };
                i += 1;
                continue;
            }
            'r' => {
                let mut hash_count = 0;
                let mut j = i + 1;
                while j < len && chars[j] == '#' {
                    hash_count += 1;
                    j += 1;
                }
                if j < len && chars[j] == '"' {
                    *ctx = ScanContext::RawString {
                        closing_hashes: hash_count,
                    };
                    i = j + 1;
                    continue;
                }
            }
            '\'' => {
                if i + 2 < len && (chars[i + 1] == '\\' || chars[i + 2] == '\'') {
                    i += 1;
                    while i < len {
                        if chars[i] == '\\' {
                            i += 2;
                            continue;
                        }
                        if chars[i] == '\'' {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }
                i += 1;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                continue;
            }
            '/' if i + 1 < len && chars[i + 1] == '/' => break,
            '/' if i + 1 < len && chars[i + 1] == '*' => {
                *ctx = ScanContext::BlockComment;
                i += 2;
                continue;
            }
            '{' => opens += 1,
            '}' => closes += 1,
            _ => {}
        }

        if ch == 'f' && i + 1 < len && chars[i + 1] == 'n' {
            let before_ok = i == 0 || (!chars[i - 1].is_alphanumeric() && chars[i - 1] != '_');
            let after_ok = i + 2 >= len || (!chars[i + 2].is_alphanumeric() && chars[i + 2] != '_');
            if before_ok && after_ok {
                has_fn = true;
            }
        }

        i += 1;
    }

    let ends_with_semi = line.trim_end().ends_with(';');

    LineInfo {
        opens,
        closes,
        has_fn,
        ends_with_semi,
    }
}

#[cfg(test)]
#[path = "inline_comments_parser_tests.rs"]
mod parser_tests;

#[cfg(test)]
#[path = "inline_comments_rule_tests.rs"]
mod rule_tests;
