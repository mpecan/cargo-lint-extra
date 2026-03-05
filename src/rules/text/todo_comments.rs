use crate::config::TodoCommentsConfig;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::TextRule;
use std::path::Path;

pub struct TodoCommentsRule {
    level: RuleLevel,
    keywords: Vec<String>,
    allow_with_issue: bool,
}

impl TodoCommentsRule {
    pub fn new(config: &TodoCommentsConfig) -> Self {
        Self {
            level: config.level,
            keywords: config.keywords.clone(),
            allow_with_issue: config.allow_with_issue,
        }
    }

    fn is_issue_reference(rest: &str) -> bool {
        if let Some(inner) = rest.strip_prefix('(')
            && let Some(paren_pos) = inner.find(')')
        {
            let inside = &inner[..paren_pos];
            return inside.starts_with('#') || inside.contains('-');
        }
        false
    }
}

impl TextRule for TodoCommentsRule {
    fn name(&self) -> &'static str {
        "todo-comments"
    }

    fn check_line(&self, line: &str, line_number: usize, file: &Path) -> Option<Diagnostic> {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") {
            return None;
        }

        let comment_text = trimmed.trim_start_matches('/').trim();

        for keyword in &self.keywords {
            let Some(pos) = comment_text.find(keyword.as_str()) else {
                continue;
            };
            let after_keyword = &comment_text[pos + keyword.len()..];
            if pos > 0 {
                let before = comment_text.as_bytes()[pos - 1];
                if before.is_ascii_alphanumeric() {
                    continue;
                }
            }
            if let Some(next_char) = after_keyword.chars().next()
                && next_char.is_ascii_alphanumeric()
                && next_char != '('
            {
                continue;
            }
            if self.allow_with_issue && Self::is_issue_reference(after_keyword) {
                return None;
            }
            return Some(
                Diagnostic::new(
                    self.name(),
                    self.level,
                    format!("found {keyword} comment"),
                    file,
                )
                .with_line(line_number),
            );
        }
        None
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn default_rule() -> TodoCommentsRule {
        TodoCommentsRule::new(&TodoCommentsConfig::default())
    }

    #[test]
    fn test_no_todo_passes() {
        let rule = default_rule();
        assert!(
            rule.check_line("// this is fine", 1, Path::new("test.rs"))
                .is_none()
        );
    }

    #[test]
    fn test_todo_detected() {
        let rule = default_rule();
        let result = rule.check_line("// TODO: fix this", 1, Path::new("test.rs"));
        assert!(result.is_some());
        assert!(result.unwrap().message.contains("TODO"));
    }

    #[test]
    fn test_fixme_detected() {
        let rule = default_rule();
        assert!(
            rule.check_line("// FIXME: broken", 1, Path::new("test.rs"))
                .is_some()
        );
    }

    #[test]
    fn test_hack_detected() {
        let rule = default_rule();
        assert!(
            rule.check_line("// HACK: workaround", 1, Path::new("test.rs"))
                .is_some()
        );
    }

    #[test]
    fn test_xxx_detected() {
        let rule = default_rule();
        assert!(
            rule.check_line("// XXX: needs review", 1, Path::new("test.rs"))
                .is_some()
        );
    }

    #[test]
    fn test_todo_with_issue_allowed() {
        let rule = default_rule();
        assert!(
            rule.check_line("// TODO(#123): fix later", 1, Path::new("test.rs"))
                .is_none()
        );
    }

    #[test]
    fn test_todo_with_jira_issue_allowed() {
        let rule = default_rule();
        assert!(
            rule.check_line("// TODO(JIRA-456): fix later", 1, Path::new("test.rs"))
                .is_none()
        );
    }

    #[test]
    fn test_todo_with_issue_disallowed() {
        let rule = TodoCommentsRule::new(&TodoCommentsConfig {
            allow_with_issue: false,
            ..TodoCommentsConfig::default()
        });
        assert!(
            rule.check_line("// TODO(#123): fix later", 1, Path::new("test.rs"))
                .is_some()
        );
    }

    #[test]
    fn test_non_comment_ignored() {
        let rule = default_rule();
        assert!(
            rule.check_line("let todo = 5;", 1, Path::new("test.rs"))
                .is_none()
        );
    }
}
