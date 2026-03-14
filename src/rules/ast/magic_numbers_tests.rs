use super::*;
use crate::diagnostic::{Diagnostic, RuleLevel};
use std::path::Path;

fn parse_and_check(code: &str) -> Vec<Diagnostic> {
    parse_and_check_with_config(code, &Config::default())
}

fn parse_and_check_with_config(code: &str, config: &Config) -> Vec<Diagnostic> {
    let syntax = syn::parse_file(code).expect("failed to parse test code");
    let rule = Rule::new(config);
    rule.check_file(&syntax, "", Path::new("test.rs"))
}

#[test]
fn test_allowed_numbers_not_flagged() {
    let code = "fn f() { let a = 0; let b = 1; let c = 2; let d = 10; let e = 100; }";
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config(code, &config);
    assert!(
        diags.is_empty(),
        "default allowed numbers should not be flagged: {diags:?}"
    );
}

#[test]
fn test_magic_number_flagged() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 42; }", &config);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("42"));
    assert!(diags[0].message.contains("magic number"));
}

#[test]
fn test_const_skipped_by_default() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("const MAX: u32 = 42;", &config);
    assert!(
        diags.is_empty(),
        "numbers in const should be skipped by default"
    );
}

#[test]
fn test_static_skipped_by_default() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("static LIMIT: u64 = 30;", &config);
    assert!(
        diags.is_empty(),
        "numbers in static should be skipped by default"
    );
}

#[test]
fn test_enum_discriminant_skipped_by_default() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let code = "enum Color { Red = 3, Green = 5, Blue = 7, }";
    let diags = parse_and_check_with_config(code, &config);
    assert!(
        diags.is_empty(),
        "enum discriminants should be skipped by default"
    );
}

#[test]
fn test_range_skipped_by_default() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let code = "fn f() { for i in 0..42 { let _ = i; } }";
    let diags = parse_and_check_with_config(code, &config);
    assert!(
        diags.is_empty(),
        "numbers in ranges should be skipped by default"
    );
}

#[test]
fn test_test_fn_skipped_by_default() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let code = "#[test]\nfn my_test() { let x = 999; }";
    let diags = parse_and_check_with_config(code, &config);
    assert!(
        diags.is_empty(),
        "numbers in test functions should be skipped by default"
    );
}

#[test]
fn test_cfg_test_mod_skipped_by_default() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let code = "#[cfg(test)]\nmod tests { fn helper() { let x = 999; } }";
    let diags = parse_and_check_with_config(code, &config);
    assert!(
        diags.is_empty(),
        "numbers in cfg(test) modules should be skipped by default"
    );
}

#[test]
fn test_custom_allowlist() {
    let config = Config {
        level: RuleLevel::Warn,
        allowed: vec!["42".to_string()],
        ..Config::default()
    };
    let code = "fn f() { let x = 42; let y = 99; }";
    let diags = parse_and_check_with_config(code, &config);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("99"));
}

#[test]
fn test_underscored_int_literal() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 1_000; }", &config);
    assert!(
        diags.is_empty(),
        "1_000 should normalize to 1000 and be allowed"
    );
}

#[test]
fn test_float_literal_flagged() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 3.14; }", &config);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("3.14"));
}

#[test]
fn test_deny_level_propagated() {
    let config = Config {
        level: RuleLevel::Deny,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 42; }", &config);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, RuleLevel::Deny);
}

#[test]
fn test_default_level_is_allow() {
    let diags = parse_and_check("fn f() { let x = 42; }");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, RuleLevel::Allow);
}

#[test]
fn test_ignore_const_disabled() {
    let config = Config {
        level: RuleLevel::Warn,
        ignore_const: false,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("const MAX: u32 = 42;", &config);
    assert_eq!(
        diags.len(),
        1,
        "const numbers should be flagged when ignore_const=false"
    );
}

#[test]
fn test_hex_literal_flagged() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 0xff; }", &config);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("255"));
}

#[test]
fn test_binary_literal_allowed() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 0b1010; }", &config);
    assert!(diags.is_empty(), "0b1010 = 10, which is in the allowlist");
}

#[test]
fn test_octal_literal_flagged() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 0o77; }", &config);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("63"));
}

#[test]
fn test_typed_literal_normalized() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let diags = parse_and_check_with_config("fn f() { let x = 42u32; }", &config);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("42"));
}

#[test]
fn test_enum_variant_field_not_skipped() {
    let config = Config {
        level: RuleLevel::Warn,
        ..Config::default()
    };
    let code = "enum E { A([u8; 42]), B = 3 }";
    let diags = parse_and_check_with_config(code, &config);
    assert_eq!(
        diags.len(),
        1,
        "array length in variant field should be flagged, discriminant should not: {diags:?}"
    );
    assert!(diags[0].message.contains("42"));
}

#[test]
fn test_ignore_range_disabled() {
    let config = Config {
        level: RuleLevel::Warn,
        ignore_range: false,
        ..Config::default()
    };
    let code = "fn f() { for i in 0..42 { let _ = i; } }";
    let diags = parse_and_check_with_config(code, &config);
    assert_eq!(
        diags.len(),
        1,
        "range numbers should be flagged when ignore_range=false"
    );
    assert!(diags[0].message.contains("42"));
}
