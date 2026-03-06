use super::*;

fn analyze(line: &str) -> LineInfo {
    analyze_line(line, &mut ScanContext::default())
}

#[test]
fn test_analyze_line_basic() {
    let info = analyze("fn foo() {");
    assert_eq!(info.opens, 1);
    assert_eq!(info.closes, 0);
    assert!(info.has_fn);
    assert!(!info.ends_with_semi);
}

#[test]
fn test_analyze_line_string_braces() {
    let info = analyze(r#"let s = "{ }";"#);
    assert_eq!(info.opens, 0);
    assert_eq!(info.closes, 0);
}

#[test]
fn test_analyze_line_block_comment() {
    let info = analyze("/* { } */ let x = 1;");
    assert_eq!(info.opens, 0);
    assert_eq!(info.closes, 0);
}

#[test]
fn test_analyze_line_line_comment() {
    let info = analyze("let x = 1; // {");
    assert_eq!(info.opens, 0);
    assert_eq!(info.closes, 0);
}

#[test]
fn test_analyze_line_fn_in_string() {
    let info = analyze(r#"let s = "fn foo";"#);
    assert!(!info.has_fn, "fn inside string should not be detected");
}

#[test]
fn test_analyze_line_fn_prefix() {
    let info = analyze("let define = 1;");
    assert!(!info.has_fn, "fn as part of identifier should not match");
}

#[test]
fn test_analyze_line_semicolon() {
    let info = analyze("fn foo();");
    assert!(info.ends_with_semi);
}

#[test]
fn test_analyze_line_fn_in_block_comment() {
    let info = analyze("/* fn foo() */ let x = 1;");
    assert!(
        !info.has_fn,
        "fn inside block comment should not be detected"
    );
}

#[test]
fn test_analyze_line_raw_string_braces() {
    let info = analyze(r##"let s = r#"{ fn }"#;"##);
    assert_eq!(info.opens, 0, "braces in raw string should be ignored");
    assert_eq!(info.closes, 0);
    assert!(!info.has_fn, "fn in raw string should not be detected");
}

#[test]
fn test_analyze_line_raw_string_no_hashes() {
    let info = analyze(r#"let s = r"{ fn }";"#);
    assert_eq!(info.opens, 0, "braces in raw string should be ignored");
    assert!(!info.has_fn);
}

#[test]
fn test_analyze_multi_line_block_comment() {
    let mut ctx = ScanContext::default();
    let info1 = analyze_line("let x = 1; /*", &mut ctx);
    assert!(matches!(ctx, ScanContext::BlockComment));
    assert_eq!(info1.opens, 0);

    let info2 = analyze_line("{ fn foo() }", &mut ctx);
    assert!(matches!(ctx, ScanContext::BlockComment));
    assert_eq!(
        info2.opens, 0,
        "braces in multi-line block comment should be ignored"
    );
    assert!(
        !info2.has_fn,
        "fn in multi-line block comment should not be detected"
    );

    let info3 = analyze_line("*/ let y = 2;", &mut ctx);
    assert!(matches!(ctx, ScanContext::Code));
    assert_eq!(info3.opens, 0);
}

#[test]
fn test_analyze_multi_line_string() {
    let mut ctx = ScanContext::default();
    let info1 = analyze_line(r#"let s = ""#, &mut ctx);
    assert!(matches!(ctx, ScanContext::String { .. }));
    assert_eq!(info1.opens, 0);

    let info2 = analyze_line("{ fn foo() }", &mut ctx);
    assert!(matches!(ctx, ScanContext::String { .. }));
    assert_eq!(
        info2.opens, 0,
        "braces in multi-line string should be ignored"
    );
    assert!(
        !info2.has_fn,
        "fn in multi-line string should not be detected"
    );

    let info3 = analyze_line(r#"";"#, &mut ctx);
    assert!(matches!(ctx, ScanContext::Code));
    assert_eq!(info3.opens, 0);
}

#[test]
#[allow(clippy::needless_raw_string_hashes)]
fn test_analyze_multi_line_raw_string() {
    let mut ctx = ScanContext::default();
    let _info1 = analyze_line(r##"let s = r#""##, &mut ctx);
    assert!(matches!(ctx, ScanContext::RawString { .. }));

    let info2 = analyze_line("{ fn foo() }", &mut ctx);
    assert!(matches!(ctx, ScanContext::RawString { .. }));
    assert_eq!(
        info2.opens, 0,
        "braces in multi-line raw string should be ignored"
    );

    let _info3 = analyze_line(r##""#;"##, &mut ctx);
    assert!(matches!(ctx, ScanContext::Code));
}
