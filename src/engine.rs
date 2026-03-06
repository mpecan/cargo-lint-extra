use crate::config::Config;
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rules::ast::allow_audit::AllowAuditRule;
use crate::rules::text::file_header::FileHeaderRule;
use crate::rules::text::file_length::FileLengthRule;
use crate::rules::text::inline_comments::InlineCommentsRule;
use crate::rules::text::line_length::LineLengthRule;
use crate::rules::text::todo_comments::TodoCommentsRule;
use crate::rules::{AstRule, TextRule};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Mutex;

pub struct Engine {
    text_rules: Vec<Box<dyn TextRule>>,
    ast_rules: Vec<Box<dyn AstRule>>,
    exclude: Vec<String>,
}

impl Engine {
    pub fn new(config: &Config) -> Self {
        let rules = &config.rules;
        let mut text_rules: Vec<Box<dyn TextRule>> = Vec::new();
        let mut ast_rules: Vec<Box<dyn AstRule>> = Vec::new();

        if rules.line_length.level != RuleLevel::Allow {
            text_rules.push(Box::new(LineLengthRule::new(&rules.line_length)));
        }
        if rules.file_length.level != RuleLevel::Allow {
            text_rules.push(Box::new(FileLengthRule::new(&rules.file_length)));
        }
        if rules.todo_comments.level != RuleLevel::Allow {
            text_rules.push(Box::new(TodoCommentsRule::new(&rules.todo_comments)));
        }
        if rules.file_header.level != RuleLevel::Allow {
            text_rules.push(Box::new(FileHeaderRule::new(&rules.file_header)));
        }
        if rules.inline_comments.level != RuleLevel::Allow {
            text_rules.push(Box::new(InlineCommentsRule::new(&rules.inline_comments)));
        }

        if rules.allow_audit.level != RuleLevel::Allow {
            ast_rules.push(Box::new(AllowAuditRule::new(&rules.allow_audit)));
        }

        Self {
            text_rules,
            ast_rules,
            exclude: config.global.exclude.clone(),
        }
    }

    /// Run all enabled rules against Rust files under `root`.
    ///
    /// # Panics
    /// Panics if the internal diagnostics mutex is poisoned.
    pub fn run(&self, root: &Path) -> Vec<Diagnostic> {
        let mut builder = WalkBuilder::new(root);
        builder.hidden(true).git_ignore(true);

        let files: Vec<_> = builder
            .build()
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                path.extension().is_some_and(|ext| ext == "rs") && !self.is_excluded(path, root)
            })
            .map(ignore::DirEntry::into_path)
            .collect();

        let diagnostics = Mutex::new(Vec::new());

        files.par_iter().for_each(|file| {
            let Ok(content) = std::fs::read_to_string(file) else {
                return;
            };

            let mut file_diags = Vec::new();

            for rule in &self.text_rules {
                for (i, line) in content.lines().enumerate() {
                    if let Some(diag) = rule.check_line(line, i + 1, file) {
                        file_diags.push(diag);
                    }
                }
                file_diags.extend(rule.check_file(&content, file));
            }

            if !self.ast_rules.is_empty() {
                match syn::parse_file(&content) {
                    Ok(syntax) => {
                        for rule in &self.ast_rules {
                            file_diags.extend(rule.check_file(&syntax, file));
                        }
                    }
                    Err(err) => {
                        file_diags.push(
                            Diagnostic::new(
                                "parse-error",
                                RuleLevel::Warn,
                                format!("failed to parse: {err}"),
                                file,
                            )
                            .with_line(err.span().start().line),
                        );
                    }
                }
            }

            if !file_diags.is_empty() {
                #[allow(clippy::unwrap_used)]
                diagnostics.lock().unwrap().extend(file_diags);
            }
        });

        #[allow(clippy::unwrap_used)]
        let mut result = diagnostics.into_inner().unwrap();
        result.sort_by(|a, b| {
            a.file
                .cmp(&b.file)
                .then(a.line.cmp(&b.line))
                .then(a.column.cmp(&b.column))
        });
        result
    }

    fn is_excluded(&self, path: &Path, root: &Path) -> bool {
        let relative = path.strip_prefix(root).unwrap_or(path);
        let path_str = relative.to_string_lossy();
        path_str.starts_with("target")
            || self
                .exclude
                .iter()
                .any(|pattern| path_str.starts_with(pattern.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_default_config() {
        let config = Config::default();
        let engine = Engine::new(&config);
        assert_eq!(engine.text_rules.len(), 4);
        assert_eq!(engine.ast_rules.len(), 0);
    }

    #[test]
    fn test_is_excluded() {
        let config = Config::default();
        let engine = Engine::new(&config);
        let root = Path::new("/project");
        assert!(engine.is_excluded(Path::new("/project/target/debug/main.rs"), root));
        assert!(!engine.is_excluded(Path::new("/project/src/main.rs"), root));
    }
}
