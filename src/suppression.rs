use std::collections::{HashMap, HashSet};

const MARKER: &str = "cargo-lint-extra:allow";
const WILDCARD: &str = "";

/// Maps line numbers to sets of suppressed rule names.
/// An empty string in the set means "all rules suppressed".
pub struct SuppressionMap {
    map: HashMap<usize, HashSet<String>>,
}

impl SuppressionMap {
    /// Parse suppression comments from file content.
    /// Lines are 1-indexed to match diagnostic line numbers.
    pub fn from_content(content: &str) -> Self {
        if !content.contains(MARKER) {
            return Self {
                map: HashMap::new(),
            };
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut map: HashMap<usize, HashSet<String>> = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            let Some(rules) = parse_suppression(line) else {
                continue;
            };
            let line_num = i + 1; // 1-indexed

            if is_inline_suppression(line) {
                map.entry(line_num).or_default().extend(rules);
            } else if let Some(next_idx) = find_next_code_line(&lines, i + 1) {
                apply_suppression(&lines, next_idx, rules, &mut map);
            }
        }

        Self { map }
    }

    /// Returns true if the map contains no suppressions.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Check if a diagnostic at the given line for the given rule is suppressed.
    /// File-level diagnostics (line = None) are never suppressed.
    pub fn is_suppressed(&self, line: Option<usize>, rule: &str) -> bool {
        line.is_some_and(|line_num| {
            self.map
                .get(&line_num)
                .is_some_and(|rules| rules.contains(WILDCARD) || rules.contains(rule))
        })
    }
}

fn apply_suppression(
    lines: &[&str],
    next_idx: usize,
    rules: HashSet<String>,
    map: &mut HashMap<usize, HashSet<String>>,
) {
    if is_block_start(lines[next_idx]) {
        if let Some(end_idx) = find_block_end(lines, next_idx) {
            for ln in (next_idx + 1)..=(end_idx + 1) {
                map.entry(ln).or_default().extend(rules.clone());
            }
        }
    } else {
        map.entry(next_idx + 1).or_default().extend(rules);
    }
}

/// Extract rule names from a suppression comment, if present.
fn parse_suppression(line: &str) -> Option<HashSet<String>> {
    let comment_start = line.find("//")?;
    let after_slashes = line[comment_start + 2..].trim_start();
    let rest = after_slashes.strip_prefix(MARKER)?;
    let rest = rest.trim_start();

    rest.strip_prefix('(').map_or_else(
        || Some(HashSet::from([String::new()])),
        |inner_rest| {
            let inner = inner_rest.split(')').next().unwrap_or("");
            if inner.trim().is_empty() {
                Some(HashSet::from([String::new()]))
            } else {
                Some(
                    inner
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            }
        },
    )
}

/// Returns true if the suppression comment is inline (has code before `//`).
fn is_inline_suppression(line: &str) -> bool {
    line.find("//")
        .is_some_and(|idx| !line[..idx].trim().is_empty())
}

/// Find the next non-blank, non-suppression-comment line index (0-indexed).
fn find_next_code_line(lines: &[&str], start: usize) -> Option<usize> {
    for (i, line) in lines.iter().enumerate().skip(start) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed
            .strip_prefix("//")
            .is_some_and(|after| after.trim_start().starts_with(MARKER))
        {
            continue;
        }
        return Some(i);
    }
    None
}

/// Check if a line starts a block item (fn, mod, impl, struct, enum, trait).
fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim();
    let keywords = [
        "fn ", "mod ", "impl ", "impl<", "struct ", "enum ", "trait ",
    ];
    let prefixes = [
        "pub ",
        "pub(crate) ",
        "pub(super) ",
        "async ",
        "unsafe ",
        "const ",
    ];

    if trimmed.starts_with('#') {
        return false;
    }

    let mut s = trimmed;
    loop {
        let mut found = false;
        for prefix in &prefixes {
            if let Some(rest) = s.strip_prefix(prefix) {
                s = rest;
                found = true;
                break;
            }
        }
        if !found {
            break;
        }
    }

    keywords.iter().any(|kw| s.starts_with(kw))
}

/// Find the closing brace of a block starting at `start_idx` (0-indexed).
fn find_block_end(lines: &[&str], start_idx: usize) -> Option<usize> {
    let mut depth: i32 = 0;
    let mut in_block_comment = false;
    let mut found_open = false;

    for (i, line) in lines.iter().enumerate().skip(start_idx) {
        let bytes = line.as_bytes();
        if let Some(result) =
            scan_line_braces(bytes, &mut depth, &mut in_block_comment, &mut found_open, i)
        {
            return Some(result);
        }
    }
    None
}

fn scan_line_braces(
    bytes: &[u8],
    depth: &mut i32,
    in_block_comment: &mut bool,
    found_open: &mut bool,
    line_idx: usize,
) -> Option<usize> {
    let mut j = 0;
    while j < bytes.len() {
        if *in_block_comment {
            if j + 1 < bytes.len() && bytes[j] == b'*' && bytes[j + 1] == b'/' {
                *in_block_comment = false;
                j += 2;
                continue;
            }
            j += 1;
            continue;
        }

        if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'/' {
            break;
        }
        if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'*' {
            *in_block_comment = true;
            j += 2;
            continue;
        }

        if bytes[j] == b'"' {
            j = skip_string(bytes, j + 1);
            continue;
        }
        if bytes[j] == b'\'' {
            j = skip_char_or_lifetime(bytes, j + 1);
            continue;
        }

        if bytes[j] == b'{' {
            *depth += 1;
            *found_open = true;
        } else if bytes[j] == b'}' {
            *depth -= 1;
            if *found_open && *depth == 0 {
                return Some(line_idx);
            }
        }
        j += 1;
    }
    None
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut j = start;
    while j < bytes.len() {
        if bytes[j] == b'\\' {
            j += 2;
            continue;
        }
        if bytes[j] == b'"' {
            return j + 1;
        }
        j += 1;
    }
    j
}

/// Skip a char literal (`'x'`, `'\n'`) or a lifetime (`'a`).
/// Lifetimes are an identifier char followed by a non-quote, so we
/// distinguish them by checking whether a closing `'` appears within
/// the next few bytes (char literals are at most `'\X'` = 3 bytes).
fn skip_char_or_lifetime(bytes: &[u8], start: usize) -> usize {
    if start >= bytes.len() {
        return start;
    }
    if bytes[start] == b'\\' && start + 2 < bytes.len() && bytes[start + 2] == b'\'' {
        return start + 3;
    }
    if start + 1 < bytes.len() && bytes[start + 1] == b'\'' {
        return start + 2;
    }
    start
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_suppression() {
        let content = "let x = 1; // cargo-lint-extra:allow(line-length)\n";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(1), "line-length"));
        assert!(!map.is_suppressed(Some(1), "todo-comments"));
    }

    #[test]
    fn test_next_line_suppression() {
        let content = "\
// cargo-lint-extra:allow(todo-comments)
// TODO: this is fine
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(2), "todo-comments"));
        assert!(!map.is_suppressed(Some(1), "todo-comments"));
    }

    #[test]
    fn test_block_suppression() {
        let content = "\
// cargo-lint-extra:allow(inline-comments)
fn my_function() {
    let x = 1; // inline comment
    let y = 2; // another
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(2), "inline-comments"));
        assert!(map.is_suppressed(Some(3), "inline-comments"));
        assert!(map.is_suppressed(Some(4), "inline-comments"));
        assert!(map.is_suppressed(Some(5), "inline-comments"));
    }

    #[test]
    fn test_block_suppression_pub_fn() {
        let content = "\
// cargo-lint-extra:allow(line-length)
pub fn long_function() {
    let x = 1;
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(2), "line-length"));
        assert!(map.is_suppressed(Some(3), "line-length"));
        assert!(map.is_suppressed(Some(4), "line-length"));
    }

    #[test]
    fn test_block_suppression_impl() {
        let content = "\
// cargo-lint-extra:allow(inline-comments)
impl Foo {
    fn bar() {} // comment
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(2), "inline-comments"));
        assert!(map.is_suppressed(Some(3), "inline-comments"));
        assert!(map.is_suppressed(Some(4), "inline-comments"));
    }

    #[test]
    fn test_multiple_rules() {
        let content = "let x = 1; // cargo-lint-extra:allow(line-length, todo-comments)\n";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(1), "line-length"));
        assert!(map.is_suppressed(Some(1), "todo-comments"));
        assert!(!map.is_suppressed(Some(1), "file-header"));
    }

    #[test]
    fn test_wildcard_suppression_no_parens() {
        let content = "let x = 1; // cargo-lint-extra:allow\n";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(1), "line-length"));
        assert!(map.is_suppressed(Some(1), "todo-comments"));
        assert!(map.is_suppressed(Some(1), "anything"));
    }

    #[test]
    fn test_wildcard_suppression_empty_parens() {
        let content = "let x = 1; // cargo-lint-extra:allow()\n";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(1), "line-length"));
        assert!(map.is_suppressed(Some(1), "anything"));
    }

    #[test]
    fn test_no_line_not_suppressible() {
        let content = "// cargo-lint-extra:allow(file-length)\nfn foo() {}\n";
        let map = SuppressionMap::from_content(content);
        assert!(!map.is_suppressed(None, "file-length"));
    }

    #[test]
    fn test_blank_line_skipping() {
        let lines = [
            "// cargo-lint-extra:allow(todo-comments)",
            "",
            "// TODO: still suppressed",
        ];
        let content = lines.join("\n");
        let map = SuppressionMap::from_content(&content);
        assert!(map.is_suppressed(Some(3), "todo-comments"));
    }

    #[test]
    fn test_braces_in_string_not_counted() {
        let content = "\
// cargo-lint-extra:allow(inline-comments)
fn foo() {
    let s = \"}\"; // comment
    let t = 1;
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(4), "inline-comments"));
        assert!(map.is_suppressed(Some(5), "inline-comments"));
    }

    #[test]
    fn test_braces_in_comment_not_counted() {
        let content = "\
// cargo-lint-extra:allow(inline-comments)
fn foo() {
    // }
    let t = 1;
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(4), "inline-comments"));
        assert!(map.is_suppressed(Some(5), "inline-comments"));
    }

    #[test]
    fn test_mod_block_suppression() {
        let content = "\
// cargo-lint-extra:allow(line-length)
mod inner {
    fn x() {}
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(2), "line-length"));
        assert!(map.is_suppressed(Some(3), "line-length"));
        assert!(map.is_suppressed(Some(4), "line-length"));
    }

    #[test]
    fn test_lifetime_does_not_break_brace_tracking() {
        let content = "\
// cargo-lint-extra:allow(line-length)
fn foo<'a>(x: &'a str) {
    let y = x;
}
";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_suppressed(Some(2), "line-length"));
        assert!(map.is_suppressed(Some(3), "line-length"));
        assert!(map.is_suppressed(Some(4), "line-length"));
        assert!(!map.is_suppressed(Some(5), "line-length"));
    }

    #[test]
    fn test_malformed_allow_comma_not_wildcard() {
        let content = "let x = 1; // cargo-lint-extra:allow(,)\n";
        let map = SuppressionMap::from_content(content);
        assert!(!map.is_suppressed(Some(1), "line-length"));
    }

    #[test]
    fn test_empty_map() {
        let content = "fn main() {}\n";
        let map = SuppressionMap::from_content(content);
        assert!(map.is_empty());
        assert!(!map.is_suppressed(Some(1), "anything"));
    }
}
