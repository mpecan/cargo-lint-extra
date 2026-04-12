#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

fn parse_and_check(code: &str) -> Vec<Diagnostic> {
    parse_and_check_with_config(code, &Config::default())
}

fn parse_and_check_with_config(code: &str, config: &Config) -> Vec<Diagnostic> {
    let syntax = syn::parse_file(code).expect("failed to parse test code");
    let rule = Rule::new(config);
    rule.check_file(&syntax, code, Path::new("test.rs"))
}

#[test]
fn test_try_operator_result_flagged() {
    let code = r"
fn f() -> Result<i32, String> {
    let x = match parse() { Ok(x) => x, Err(e) => return Err(e) };
    Ok(x)
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("`?` operator"));
}

#[test]
fn test_try_operator_with_into_flagged() {
    let code = r"
fn f() -> Result<i32, String> {
    let x = match parse() { Ok(x) => x, Err(e) => return Err(e.into()) };
    Ok(x)
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_try_operator_reverse_arm_order_flagged() {
    let code = r"
fn f() -> Result<i32, String> {
    let x = match parse() { Err(e) => return Err(e), Ok(x) => x };
    Ok(x)
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_try_operator_nonidentity_ok_not_flagged() {
    let code = r"
fn f() -> Result<i32, String> {
    let x = match parse() { Ok(x) => x * 2, Err(e) => return Err(e) };
    Ok(x)
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty(), "nonidentity Ok body: {diags:?}");
}

#[test]
fn test_try_operator_fallback_err_not_flagged() {
    let code = r"
fn f() -> i32 {
    match parse() { Ok(x) => x, Err(_) => 0 }
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty(), "fallback Err: {diags:?}");
}

#[test]
fn test_if_let_some_empty_none_flagged() {
    let code = r"
fn f() {
    match get() { Some(x) => { let _ = x; }, None => {} }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("`if let`"));
}

#[test]
fn test_if_let_reverse_order_flagged() {
    let code = r"
fn f() {
    match get() { None => {}, Some(x) => { let _ = x; } }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_if_let_ok_empty_err_flagged() {
    let code = r"
fn f() {
    match parse() { Ok(x) => { let _ = x; }, Err(_) => {} }
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_if_let_unit_empty_body_flagged() {
    let code = r"
fn f() {
    match get() { Some(x) => { let _ = x; }, None => () }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_if_let_nontrivial_none_not_flagged() {
    let code = r"
fn f() {
    match get() { Some(x) => { let _ = x; }, None => { let _ = 0; } }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty(), "non-empty None: {diags:?}");
}

#[test]
fn test_map_some_combinator_flagged() {
    let code = r"
fn f() -> Option<i32> {
    match get() { Some(x) => Some(x + 1), None => None }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("`.map()`"));
}

#[test]
fn test_map_result_combinator_flagged() {
    let code = r"
fn f() -> Result<i32, String> {
    match parse() { Ok(x) => Ok(x + 1), Err(e) => Err(e) }
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_map_reverse_order_flagged() {
    let code = r"
fn f() -> Option<i32> {
    match get() { None => None, Some(x) => Some(x + 1) }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert_eq!(diags.len(), 1);
}

#[test]
fn test_map_none_nondefault_not_flagged() {
    let code = r"
fn f() -> Option<i32> {
    match get() { Some(x) => Some(x + 1), None => Some(0) }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty(), "nondefault None: {diags:?}");
}

#[test]
fn test_map_err_transformed_not_flagged() {
    let code = r#"
fn f() -> Result<i32, String> {
    match parse() { Ok(x) => Ok(x + 1), Err(e) => Err(e + "!") }
}
fn parse() -> Result<i32, String> { Ok(0) }
"#;
    let diags = parse_and_check(code);
    assert!(diags.is_empty(), "transformed Err: {diags:?}");
}

#[test]
fn test_three_arm_match_not_flagged() {
    let code = r"
fn f(x: i32) { match x { 1 => {}, 2 => {}, _ => {} } }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty());
}

#[test]
fn test_match_with_guards_not_flagged() {
    let code = r"
fn f() {
    match get() { Some(x) if x > 0 => {}, _ => {} }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty());
}

#[test]
fn test_enum_match_not_flagged() {
    let code = r"
enum Color { R, G, B }
fn f(c: Color) { match c { Color::R => {}, Color::G => {}, Color::B => {} } }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty());
}

#[test]
fn test_match_with_wildcard_not_flagged() {
    let code = r"
fn f() {
    match get() { Some(x) => { let _ = x; }, _ => {} }
}
fn get() -> Option<i32> { None }
";
    let diags = parse_and_check(code);
    assert!(diags.is_empty());
}

#[test]
fn test_deny_level_propagated() {
    let config = Config {
        level: RuleLevel::Deny,
    };
    let code = r"
fn f() -> Result<i32, String> {
    let x = match parse() { Ok(x) => x, Err(e) => return Err(e) };
    Ok(x)
}
fn parse() -> Result<i32, String> { Ok(0) }
";
    let diags = parse_and_check_with_config(code, &config);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, RuleLevel::Deny);
}
