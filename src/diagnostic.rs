use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleLevel {
    Allow,
    Warn,
    Deny,
}

impl std::fmt::Display for RuleLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Warn => write!(f, "warning"),
            Self::Deny => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub rule: String,
    pub level: RuleLevel,
    pub message: String,
    pub file: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl Diagnostic {
    pub fn new(
        rule: impl Into<String>,
        level: RuleLevel,
        message: impl Into<String>,
        file: impl AsRef<Path>,
    ) -> Self {
        Self {
            rule: rule.into(),
            level,
            message: message.into(),
            file: file.as_ref().to_path_buf(),
            line: None,
            column: None,
        }
    }

    #[must_use]
    pub const fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    #[must_use]
    pub const fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn format_human(&self) -> String {
        let location = match (self.line, self.column) {
            (Some(l), Some(c)) => format!("{}:{l}:{c}", self.file.display()),
            (Some(l), None) => format!("{}:{l}", self.file.display()),
            _ => format!("{}", self.file.display()),
        };
        format!(
            "{} [{}] {location}: {}",
            self.level, self.rule, self.message
        )
    }

    /// Formats this diagnostic as a JSON string.
    ///
    /// Returns an error placeholder if serialization fails (should never happen).
    pub fn format_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| String::from("{\"error\":\"serialization failed\"}"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_format_human() {
        let d = Diagnostic::new(
            "line-length",
            RuleLevel::Warn,
            "line too long",
            "src/main.rs",
        )
        .with_line(10)
        .with_column(120);
        let formatted = d.format_human();
        assert!(formatted.contains("warning"));
        assert!(formatted.contains("line-length"));
        assert!(formatted.contains("src/main.rs:10:120"));
        assert!(formatted.contains("line too long"));
    }

    #[test]
    fn test_diagnostic_format_human_no_column() {
        let d = Diagnostic::new(
            "file-length",
            RuleLevel::Deny,
            "file too long",
            "src/lib.rs",
        )
        .with_line(500);
        let formatted = d.format_human();
        assert!(formatted.contains("error [file-length] src/lib.rs:500:"));
        let d2 = Diagnostic::new(
            "file-length",
            RuleLevel::Deny,
            "file too long",
            "src/lib.rs",
        )
        .with_line(500)
        .with_column(5);
        let formatted2 = d2.format_human();
        assert!(formatted2.contains("src/lib.rs:500:5"));
    }

    #[test]
    fn test_diagnostic_format_json() {
        let d = Diagnostic::new(
            "todo-comments",
            RuleLevel::Warn,
            "found TODO",
            "src/main.rs",
        )
        .with_line(5);
        let json = d.format_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["rule"], "todo-comments");
        assert_eq!(parsed["level"], "warn");
        assert_eq!(parsed["line"], 5);
    }

    #[test]
    fn test_rule_level_display() {
        assert_eq!(format!("{}", RuleLevel::Allow), "allow");
        assert_eq!(format!("{}", RuleLevel::Warn), "warning");
        assert_eq!(format!("{}", RuleLevel::Deny), "error");
    }
}
