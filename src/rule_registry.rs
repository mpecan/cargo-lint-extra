/// Declarative macro that generates all central wiring for lint rules.
///
/// Each rule module must export: `Config`, `Override`, `Rule`, and `apply_override`.
/// The macro generates: `RulesConfig`, `TestRulesOverrides`, `build_text_rules`,
/// `build_ast_rules`, `set_rule_level`, `apply_test_overrides`, and `ALL_RULE_NAMES`.
macro_rules! declare_rules {
    (
        text {
            $( $t_mod:ident : $t_name:literal ),* $(,)?
        }
        ast {
            $( $a_mod:ident : $a_name:literal ),* $(,)?
        }
    ) => {
        use crate::diagnostic::RuleLevel;
        use crate::rules::{TextRule, AstRule};
        use serde::Deserialize;

        #[derive(Debug, Clone, Default, Deserialize)]
        #[serde(default, rename_all = "kebab-case")]
        pub struct RulesConfig {
            $( pub $t_mod: crate::rules::text::$t_mod::Config, )*
            $( pub $a_mod: crate::rules::ast::$a_mod::Config, )*
        }

        #[derive(Debug, Clone, Default, Deserialize)]
        #[serde(default, rename_all = "kebab-case")]
        pub struct TestRulesOverrides {
            $( pub $t_mod: Option<crate::rules::text::$t_mod::Override>, )*
            $( pub $a_mod: Option<crate::rules::ast::$a_mod::Override>, )*
        }

        pub fn build_text_rules(rules: &RulesConfig) -> Vec<Box<dyn TextRule>> {
            let mut v: Vec<Box<dyn TextRule>> = Vec::new();
            $(
                if rules.$t_mod.level != RuleLevel::Allow {
                    v.push(Box::new(
                        crate::rules::text::$t_mod::Rule::new(&rules.$t_mod),
                    ));
                }
            )*
            v
        }

        pub fn build_ast_rules(rules: &RulesConfig) -> Vec<Box<dyn AstRule>> {
            let mut v: Vec<Box<dyn AstRule>> = Vec::new();
            $(
                if rules.$a_mod.level != RuleLevel::Allow {
                    v.push(Box::new(
                        crate::rules::ast::$a_mod::Rule::new(&rules.$a_mod),
                    ));
                }
            )*
            v
        }

        pub fn set_rule_level(
            rules: &mut RulesConfig,
            rule: &str,
            level: RuleLevel,
        ) -> bool {
            match rule {
                $( $t_name => { rules.$t_mod.level = level; true } )*
                $( $a_name => { rules.$a_mod.level = level; true } )*
                _ => false,
            }
        }

        pub fn apply_test_overrides(
            rules: &mut RulesConfig,
            overrides: &TestRulesOverrides,
        ) {
            $(
                if let Some(o) = &overrides.$t_mod {
                    crate::rules::text::$t_mod::apply_override(
                        &mut rules.$t_mod, o,
                    );
                }
            )*
            $(
                if let Some(o) = &overrides.$a_mod {
                    crate::rules::ast::$a_mod::apply_override(
                        &mut rules.$a_mod, o,
                    );
                }
            )*
        }

        pub const ALL_RULE_NAMES: &[&str] = &[
            $( $t_name, )*
            $( $a_name, )*
        ];
    };
}

declare_rules! {
    text {
        line_length: "line-length",
        file_length: "file-length",
        todo_comments: "todo-comments",
        file_header: "file-header",
        inline_comments: "inline-comments",
        redundant_comments: "redundant-comments",
    }
    ast {
        allow_audit: "allow-audit",
        clone_density: "clone-density",
        glob_imports: "glob-imports",
        magic_numbers: "magic-numbers",
    }
}
