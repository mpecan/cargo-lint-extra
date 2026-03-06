use super::*;
use crate::config::InlineCommentsConfig;
use crate::diagnostic::RuleLevel;

fn check_code(max_ratio: f64, max_consecutive: usize, code: &str) -> Vec<Diagnostic> {
    let rule = InlineCommentsRule::new(&InlineCommentsConfig {
        level: RuleLevel::Warn,
        max_ratio,
        max_consecutive,
    });
    rule.check_file(code, Path::new("test.rs"))
}

fn assert_clean(max_ratio: f64, max_consecutive: usize, code: &str) {
    let diags = check_code(max_ratio, max_consecutive, code);
    assert!(diags.is_empty(), "expected no diagnostics: {diags:?}");
}

#[test]
fn test_high_comment_ratio_flagged() {
    let diags = check_code(
        0.3,
        10,
        "
fn over_commented() {
    // set up x
    // set up y
    // set up z
    // compute result
    let result = 1 + 2;
}
",
    );
    assert_eq!(diags.len(), 1, "expected exactly 1 diagnostic: {diags:?}");
    assert!(diags[0].message.contains("80%"), "{}", diags[0].message);
    assert!(diags[0].message.contains("4/5"), "{}", diags[0].message);
    assert_eq!(diags[0].line, Some(2));
}

#[test]
fn test_consecutive_comments_flagged() {
    let diags = check_code(
        1.0,
        3,
        "
fn many_consecutive() {
    let x = 1;
    // comment 1
    // comment 2
    // comment 3
    // comment 4
    let y = 2;
    let z = 3;
    let w = 4;
}
",
    );
    assert_eq!(diags.len(), 1, "expected exactly 1 diagnostic: {diags:?}");
    assert!(
        diags[0].message.contains("4 consecutive"),
        "{}",
        diags[0].message
    );
    assert_eq!(diags[0].line, Some(4));
}

#[test]
fn test_ratio_only_not_consecutive() {
    // Scattered comments: 4/8 = 50% ratio, max run = 1
    let diags = check_code(
        0.3,
        10,
        "
fn scattered() {
    // a
    let a = 1;
    // b
    let b = 2;
    // c
    let c = 3;
    // d
    let d = 4;
}
",
    );
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert!(
        diags[0].message.contains("inline comments"),
        "{}",
        diags[0].message
    );
    assert!(
        !diags[0].message.contains("consecutive"),
        "{}",
        diags[0].message
    );
}

#[test]
fn test_consecutive_only_not_ratio() {
    // 4 consecutive, ratio 40% < 90%: only consecutive should fire
    let diags = check_code(
        0.9,
        3,
        "
fn mostly_code() {
    let a = 1;
    let b = 2;
    let c = 3;
    // comment 1
    // comment 2
    // comment 3
    // comment 4
    let d = 4;
    let e = 5;
    let f = 6;
}
",
    );
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert!(
        diags[0].message.contains("consecutive"),
        "{}",
        diags[0].message
    );
}

#[test]
fn test_doc_comments_not_counted() {
    // Outer doc comments (///) should not be counted
    assert_clean(
        0.3,
        3,
        "
/// Doc comment 1
/// Doc comment 2
/// Doc comment 3
fn documented() {
    let x = 1;
    let y = 2;
    let z = x + y;
    println!(\"{z}\");
}
",
    );
    // Inner doc comments (//!) should not be counted
    assert_clean(
        0.3,
        3,
        "
fn with_inner_docs() {
    //! inner doc 1
    //! inner doc 2
    //! inner doc 3
    //! inner doc 4
    let x = 1;
    let y = x + 1;
    let z = y + 1;
    println!(\"{z}\");
}
",
    );
}

#[test]
fn test_nested_functions_separate_scopes() {
    let diags = check_code(
        0.3,
        10,
        "
fn outer() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    fn inner() {
        // comment 1
        // comment 2
        // comment 3
        // comment 4
        let x = 1;
    }
}
",
    );
    let ratio_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("inline comments"))
        .collect();
    assert_eq!(
        ratio_diags.len(),
        1,
        "only inner fn flagged: {ratio_diags:?}"
    );
    assert_eq!(ratio_diags[0].line, Some(7));
    // Multi-line signatures still detect body comments
    let diags = check_code(
        0.3,
        10,
        "
fn multi_line(
    x: i32,
    y: i32,
) {
    // comment 1
    // comment 2
    // comment 3
    // comment 4
    let z = x + y;
}
",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("inline comments")),
        "{diags:?}"
    );
}

#[test]
fn test_braces_in_non_code_contexts() {
    // Closures don't create scope
    assert_clean(
        0.3,
        10,
        "
fn with_closure() {
    let f = || { 1 + 2 };
    let x = f();
    let y = x + 1;
    let z = y + 1;
}
",
    );
    // Strings
    assert_clean(
        0.3,
        3,
        r#"
fn string_braces() {
    let a = "{ }";
    let b = "{{}}";
    let c = a.len();
    let d = b.len();
}
"#,
    );
    // Block comments
    assert_clean(
        0.3,
        3,
        "
fn block_comment_braces() {
    /* { } */
    let x = 1;
    let y = 2;
    let z = 3;
}
",
    );
}

#[test]
fn test_multi_line_constructs_with_braces() {
    // Multi-line block comment
    assert_clean(
        0.3,
        3,
        "
fn multi_line_comment() {
    /*
    {
    }
    */
    let x = 1;
    let y = 2;
    let z = 3;
    let w = 4;
}
",
    );
    // Multi-line string
    assert_clean(
        0.3,
        3,
        r#"
fn multi_line_string() {
    let s = "
    {
    }
    ";
    let x = s.len();
    let y = x + 1;
    let z = y + 1;
    let w = z + 1;
}
"#,
    );
    // Raw string
    assert_clean(
        0.3,
        3,
        r##"
fn raw_string_test() {
    let s = r#"{ fn foo() }"#;
    let x = s.len();
    let y = x + 1;
    let z = y + 1;
}
"##,
    );
}

#[test]
fn test_tiny_functions_and_trait_declarations_skipped() {
    assert_clean(
        0.3,
        3,
        "
fn tiny() {
    // comment
    // comment
    let x = 1;
}
",
    );
    assert_clean(
        0.3,
        3,
        "
trait Foo {
    fn bar();
    fn baz(x: i32) -> i32;
}
",
    );
}

#[test]
fn test_clean_code_no_diagnostics() {
    assert_clean(
        0.3,
        3,
        "
fn clean() {
    let x = 1;
    let y = 2;
    let z = x + y;
    println!(\"{z}\");
}
",
    );
    // impl blocks should not be counted as functions
    assert_clean(
        0.3,
        3,
        "
struct Foo;
impl Foo {
    fn method(&self) {
        let x = 1;
        let y = 2;
        let z = 3;
        let w = 4;
    }
}
",
    );
}

#[test]
fn test_fn_keyword_variants() {
    for (prefix, label) in [
        ("async fn", "async"),
        ("const fn", "const"),
        ("pub unsafe fn", "unsafe"),
    ] {
        let code = format!(
            "
{prefix} over_commented() {{
    // a
    // b
    // c
    // d
    let x = 1;
}}
"
        );
        let diags = check_code(0.3, 10, &code);
        assert_eq!(diags.len(), 1, "{label} fn should be detected: {diags:?}");
    }
}

#[test]
fn test_blank_lines_and_nested_blocks() {
    // Blank lines break consecutive counting
    assert_clean(
        1.0,
        3,
        "
fn blank_separated() {
    let a = 1;
    // group 1a
    // group 1b
    // group 1c

    // group 2a
    // group 2b
    // group 2c
    let b = 2;
    let c = 3;
    let d = 4;
}
",
    );
    // Comments inside if blocks still counted
    let diags = check_code(
        0.3,
        10,
        "
fn nested_comments() {
    let x = 1;
    if x > 0 {
        // a
        // b
        // c
        // d
        let y = 2;
    }
}
",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("inline comments")),
        "{diags:?}"
    );
}

#[test]
fn test_trailing_comments_and_fn_pointers_are_code() {
    // Trailing comments count as code lines
    assert_clean(
        0.3,
        3,
        "
fn trailing() {
    let x = 1; // set up x
    let y = 2; // set up y
    let z = 3; // set up z
    let w = 4; // set up w
}
",
    );
    // fn pointer types don't create scopes
    assert_clean(
        0.3,
        3,
        "
fn real_function() {
    let f: fn() -> i32 = some_fn;
    let x = f();
    let y = x + 1;
    let z = y + 1;
}
",
    );
}

#[test]
fn test_boundary_values_at_exact_thresholds() {
    // 2/6 = 0.333 exactly at max_ratio, should NOT trigger (uses >)
    assert_clean(
        2.0 / 6.0,
        10,
        "
fn at_threshold() {
    // a
    // b
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
}
",
    );
    // 3 consecutive with max=3, should NOT trigger (uses >)
    assert_clean(
        1.0,
        3,
        "
fn at_max() {
    let a = 1;
    // one
    // two
    // three
    let b = 2;
    let c = 3;
}
",
    );
}
