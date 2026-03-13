use crate::config::RedundantCommentsConfig;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

pub struct RedundantCommentsRule {
    level: RuleLevel,
    similarity_threshold: f64,
    min_words: usize,
}

impl RedundantCommentsRule {
    pub const fn new(config: &RedundantCommentsConfig) -> Self {
        Self {
            level: config.level,
            similarity_threshold: config.similarity_threshold,
            min_words: config.min_words,
        }
    }
}

const MAX_COMMENT_WORDS: usize = 20;

impl TextRule for RedundantCommentsRule {
    fn name(&self) -> &'static str {
        "redundant-comments"
    }

    fn check_file(&self, content: &str, file: &Path) -> Vec<Diagnostic> {
        let lines: Vec<&str> = content.lines().collect();
        let mut diagnostics = Vec::new();
        let mut code_word_cache: HashMap<usize, HashSet<String>> = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if !is_regular_comment(trimmed) {
                continue;
            }

            let comment_text = extract_comment_text(trimmed);
            if is_directive_comment(&comment_text) {
                continue;
            }

            let raw_words = tokenize(&comment_text);
            if raw_words.len() < self.min_words || raw_words.len() > MAX_COMMENT_WORDS {
                continue;
            }

            let comment_words = strip_stop_words(&raw_words);
            if comment_words.is_empty() {
                continue;
            }

            let Some((code_idx, code_line)) = find_next_code_line(&lines, i + 1) else {
                continue;
            };

            let code_words = code_word_cache
                .entry(code_idx)
                .or_insert_with(|| tokenize(code_line).into_iter().collect());
            if code_words.is_empty() {
                continue;
            }

            let overlap = word_overlap(&comment_words, code_words);
            if overlap >= self.similarity_threshold {
                let col = line.len() - line.trim_start().len() + 1;
                diagnostics.push(
                    Diagnostic::new(
                        self.name(),
                        self.level,
                        "comment restates the code \u{2014} consider removing",
                        file,
                    )
                    .with_line(i + 1)
                    .with_column(col),
                );
            }
        }

        diagnostics
    }
}

fn is_regular_comment(trimmed: &str) -> bool {
    trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!")
}

fn extract_comment_text(trimmed: &str) -> String {
    trimmed.trim_start_matches('/').trim().to_string()
}

fn is_directive_comment(text: &str) -> bool {
    const DIRECTIVES: &[&str] = &[
        "safety:",
        "panic:",
        "panics:",
        "todo",
        "fixme",
        "hack",
        "xxx",
        "cargo-lint-extra:allow",
        "note:",
        "warning:",
        "error:",
    ];
    let lower = text.to_lowercase();
    DIRECTIVES.iter().any(|d| lower.starts_with(d))
}

fn find_next_code_line<'a>(lines: &[&'a str], start: usize) -> Option<(usize, &'a str)> {
    for (idx, line) in lines.iter().enumerate().skip(start) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        return Some((idx, trimmed));
    }
    None
}

/// Tokenize text into lowercase words, splitting identifiers on `_` and
/// camelCase boundaries.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
        .flat_map(split_identifier)
        .collect()
}

static STOP_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "this", "that", "it", "its",
        "of", "to", "for", "in", "on", "at", "by", "with", "from", "as", "into", "and", "or",
        "but", "not", "if", "then", "else", "when", "do", "does", "did", "will", "would", "can",
        "could", "should", "may", "might", "we", "they", "he", "she", "their", "our", "here",
        "there", "all", "some",
    ]
    .into_iter()
    .collect()
});

fn strip_stop_words(words: &[String]) -> Vec<String> {
    words
        .iter()
        .filter(|w| !STOP_WORDS.contains(w.as_str()))
        .cloned()
        .collect()
}

fn split_identifier(word: &str) -> Vec<String> {
    let mut parts = Vec::new();

    for segment in word.split('_') {
        if segment.is_empty() {
            continue;
        }
        let mut current = String::new();
        for ch in segment.chars() {
            if ch.is_uppercase() && !current.is_empty() {
                parts.push(current.to_lowercase());
                current = String::new();
            }
            current.push(ch);
        }
        if !current.is_empty() {
            parts.push(current.to_lowercase());
        }
    }

    parts
}

#[allow(clippy::cast_precision_loss)]
fn word_overlap(comment_words: &[String], code_words: &HashSet<String>) -> f64 {
    if comment_words.is_empty() {
        return 0.0;
    }
    let matches = comment_words
        .iter()
        .filter(|w| code_words.contains(w.as_str()))
        .count();
    matches as f64 / comment_words.len() as f64
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[path = "redundant_comments_tests.rs"]
mod tests;
