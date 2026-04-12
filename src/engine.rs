use crate::config::{Config, TestConfig};
use crate::diagnostic::{Diagnostic, RuleLevel};
use crate::rule_registry::{self, RulesConfig};
use crate::rules::{AstRule, TextRule};
use crate::suppression::SuppressionMap;
use crate::test_detection::TestLineRanges;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Mutex;

pub struct Engine {
    prod_text_rules: Vec<Box<dyn TextRule>>,
    prod_ast_rules: Vec<Box<dyn AstRule>>,
    test_text_rules: Vec<Box<dyn TextRule>>,
    test_ast_rules: Vec<Box<dyn AstRule>>,
    exclude: Vec<String>,
    test_config: Option<TestConfig>,
}

type TextRules = Vec<Box<dyn TextRule>>;
type AstRules = Vec<Box<dyn AstRule>>;

fn build_rules(rules: &RulesConfig) -> (TextRules, AstRules) {
    (
        rule_registry::build_text_rules(rules),
        rule_registry::build_ast_rules(rules),
    )
}

impl Engine {
    pub fn new(config: &Config) -> Self {
        let (prod_text_rules, prod_ast_rules) = build_rules(&config.rules);

        let (test_text_rules, test_ast_rules, test_config) = config.test.as_ref().map_or_else(
            || (Vec::new(), Vec::new(), None),
            |test_cfg| {
                let test_rules = config.resolved_test_rules();
                let (tt, ta) = build_rules(&test_rules);
                (tt, ta, Some(test_cfg.clone()))
            },
        );

        Self {
            prod_text_rules,
            prod_ast_rules,
            test_text_rules,
            test_ast_rules,
            exclude: config.global.exclude.clone(),
            test_config,
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
            let file_diags = self.check_file(file, root);
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

    fn check_file(&self, file: &Path, root: &Path) -> Vec<Diagnostic> {
        let Ok(content) = std::fs::read_to_string(file) else {
            return Vec::new();
        };

        let relative = file.strip_prefix(root).unwrap_or(file);
        let relative_str = relative.to_string_lossy();

        if let Some(test_cfg) = &self.test_config {
            if test_cfg.is_test_file(&relative_str) {
                return Self::run_rules_on_content(
                    &self.test_text_rules,
                    &self.test_ast_rules,
                    &content,
                    file,
                );
            }

            if test_cfg.detect_cfg_test {
                let test_ranges = TestLineRanges::from_content(&content);
                if !test_ranges.is_empty() {
                    return self.check_mixed_file(&content, file, &test_ranges);
                }
            }
        }

        Self::run_rules_on_content(&self.prod_text_rules, &self.prod_ast_rules, &content, file)
    }

    fn check_mixed_file(
        &self,
        content: &str,
        file: &Path,
        test_ranges: &TestLineRanges,
    ) -> Vec<Diagnostic> {
        // Run prod rules — keep diagnostics NOT in test ranges
        let prod_diags =
            Self::run_rules_on_content(&self.prod_text_rules, &self.prod_ast_rules, content, file);
        let mut result: Vec<_> = prod_diags
            .into_iter()
            .filter(|d| !d.line.is_some_and(|l| test_ranges.is_test_line(l)))
            .collect();

        // Run test rules — keep diagnostics IN test ranges
        let test_diags =
            Self::run_rules_on_content(&self.test_text_rules, &self.test_ast_rules, content, file);
        result.extend(
            test_diags
                .into_iter()
                .filter(|d| d.line.is_some_and(|l| test_ranges.is_test_line(l))),
        );

        result
    }

    fn run_rules_on_content(
        text_rules: &[Box<dyn TextRule>],
        ast_rules: &[Box<dyn AstRule>],
        content: &str,
        file: &Path,
    ) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for rule in text_rules {
            for (i, line) in content.lines().enumerate() {
                if let Some(diag) = rule.check_line(line, i + 1, file) {
                    diags.push(diag);
                }
            }
            diags.extend(rule.check_file(content, file));
        }

        if !ast_rules.is_empty() {
            match syn::parse_file(content) {
                Ok(syntax) => {
                    for rule in ast_rules {
                        diags.extend(rule.check_file(&syntax, content, file));
                    }
                }
                Err(err) => {
                    diags.push(
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

        let suppressions = SuppressionMap::from_content(content);
        if !suppressions.is_empty() {
            diags.retain(|diag| !suppressions.is_suppressed(diag.line, &diag.rule));
        }

        diags
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_default_config() {
        let config = Config::default();
        let engine = Engine::new(&config);
        assert_eq!(engine.prod_text_rules.len(), 5);
        assert_eq!(engine.prod_ast_rules.len(), 5);
        assert!(engine.test_text_rules.is_empty());
        assert!(engine.test_ast_rules.is_empty());
        assert!(engine.test_config.is_none());
    }

    #[test]
    fn test_engine_with_test_config() {
        let config = Config {
            test: Some(TestConfig::default()),
            ..Config::default()
        };
        let engine = Engine::new(&config);
        assert_eq!(engine.prod_text_rules.len(), 5);
        // Test rules should mirror prod when no overrides
        assert_eq!(engine.test_text_rules.len(), 5);
        assert_eq!(engine.test_ast_rules.len(), 5);
        assert!(engine.test_config.is_some());
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
