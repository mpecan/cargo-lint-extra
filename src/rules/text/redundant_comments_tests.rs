use super::*;

// --- tokenize tests ---

#[test]
fn test_tokenize_simple() {
    let words = tokenize("increment the counter");
    assert_eq!(words, vec!["increment", "the", "counter"]);
}

#[test]
fn test_tokenize_with_snake_case() {
    let words = tokenize("set the user_name");
    assert_eq!(words, vec!["set", "the", "user", "name"]);
}

#[test]
fn test_tokenize_with_camel_case() {
    let words = tokenize("call getUserName");
    assert_eq!(words, vec!["call", "get", "user", "name"]);
}

// --- strip_stop_words tests ---

#[test]
fn test_strip_stop_words() {
    let words: Vec<String> = vec!["increment", "the", "counter"]
        .into_iter()
        .map(Into::into)
        .collect();
    let filtered = strip_stop_words(&words);
    assert_eq!(filtered, vec!["increment", "counter"]);
}

#[test]
fn test_strip_stop_words_all_stops() {
    let words: Vec<String> = vec!["the", "a", "is"].into_iter().map(Into::into).collect();
    let filtered = strip_stop_words(&words);
    assert!(filtered.is_empty());
}

// --- split_identifier tests ---

#[test]
fn test_split_identifier_snake_case() {
    assert_eq!(split_identifier("user_name"), vec!["user", "name"]);
}

#[test]
fn test_split_identifier_camel_case() {
    assert_eq!(split_identifier("getUserName"), vec!["get", "user", "name"]);
}

#[test]
fn test_split_identifier_simple() {
    assert_eq!(split_identifier("counter"), vec!["counter"]);
}

// --- word_overlap tests ---

#[test]
fn test_word_overlap_full() {
    let comment = vec!["increment".into(), "counter".into()];
    let code: HashSet<String> = ["increment", "counter", "1"]
        .iter()
        .map(|s| (*s).into())
        .collect();
    let overlap = word_overlap(&comment, &code);
    assert!((overlap - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_word_overlap_partial() {
    let comment = vec!["set".into(), "name".into()];
    let code: HashSet<String> = ["self", "name", "value"]
        .iter()
        .map(|s| (*s).into())
        .collect();
    let overlap = word_overlap(&comment, &code);
    assert!((overlap - 0.5).abs() < 0.01);
}

#[test]
fn test_word_overlap_none() {
    let comment = vec!["hello".into(), "world".into()];
    let code: HashSet<String> = ["foo", "bar"].iter().map(|s| (*s).into()).collect();
    let overlap = word_overlap(&comment, &code);
    assert!((overlap).abs() < f64::EPSILON);
}

// --- is_directive_comment tests ---

#[test]
fn test_directive_comments() {
    assert!(is_directive_comment("safety: pointer is valid"));
    assert!(is_directive_comment("panic: this should never happen"));
    assert!(is_directive_comment("todo fix this later"));
    assert!(is_directive_comment("fixme broken"));
    assert!(is_directive_comment("cargo-lint-extra:allow(line-length)"));
    assert!(!is_directive_comment("increment the counter"));
}

// --- is_regular_comment tests ---

#[test]
fn test_regular_comment() {
    assert!(is_regular_comment("// increment counter"));
    assert!(!is_regular_comment("/// doc comment"));
    assert!(!is_regular_comment("//! inner doc"));
    assert!(!is_regular_comment("let x = 1;"));
}

// --- find_next_code_line tests ---

#[test]
fn test_find_next_code_line_skips_blanks_and_comments() {
    let lines = vec!["// comment", "", "// another", "let x = 1;"];
    assert_eq!(find_next_code_line(&lines, 1), Some((3, "let x = 1;")));
}

#[test]
fn test_find_next_code_line_none() {
    let lines = vec!["// comment", "", "// another"];
    assert_eq!(find_next_code_line(&lines, 1), None);
}

// --- full rule tests ---

fn run_rule(content: &str) -> Vec<Diagnostic> {
    let config = RedundantCommentsConfig::default();
    let rule = RedundantCommentsRule::new(&config);
    rule.check_file(content, Path::new("test.rs"))
}

#[test]
fn test_redundant_increment_counter() {
    let content = "// increment the counter\ncounter += 1;\n";
    let diags = run_rule(content);
    assert_eq!(diags.len(), 1, "got: {diags:?}");
    assert_eq!(diags[0].rule, "redundant-comments");
    assert_eq!(diags[0].line, Some(1));
}

#[test]
fn test_redundant_return_result() {
    let content = "// return the result\nreturn result;\n";
    let diags = run_rule(content);
    assert_eq!(diags.len(), 1, "got: {diags:?}");
}

#[test]
fn test_redundant_create_new_vector() {
    let content = "// create new vector\nlet vector = Vec::new();\n";
    let diags = run_rule(content);
    assert_eq!(diags.len(), 1, "got: {diags:?}");
}

#[test]
fn test_redundant_set_name() {
    let content = "// set the name\nself.name = name;\n";
    let diags = run_rule(content);
    assert_eq!(diags.len(), 1, "got: {diags:?}");
}

#[test]
fn test_doc_comment_skipped() {
    let content = "/// increment the counter\ncounter += 1;\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_safety_directive_skipped() {
    let content = "// SAFETY: pointer is valid\nunsafe { *ptr };\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_todo_directive_skipped() {
    let content = "// TODO: fix this\nbroken_code();\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_explanatory_comment_not_flagged() {
    let content = "// This handles the edge case where the buffer might overflow\nself.flush();\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_short_comment_not_flagged() {
    let content = "// ok\nresult\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_long_comment_not_flagged() {
    let content = "// this is a very long comment that explains many things about how the system works and why we need to do this particular thing in this particular way at this point\ndo_stuff();\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_comment_with_no_following_code() {
    let content = "// trailing comment\n";
    let diags = run_rule(content);
    assert!(diags.is_empty());
}

#[test]
fn test_multiple_redundant_comments() {
    let content = "\
// increment the counter
counter += 1;
// return the result
return result;
";
    let diags = run_rule(content);
    assert_eq!(diags.len(), 2, "got: {diags:?}");
}

#[test]
fn test_comment_skips_blank_lines_to_code() {
    let content = "// set the name\n\nself.name = name;\n";
    let diags = run_rule(content);
    assert_eq!(diags.len(), 1, "got: {diags:?}");
}

#[test]
fn test_custom_threshold() {
    let config = RedundantCommentsConfig {
        level: RuleLevel::Warn,
        similarity_threshold: 1.0,
        min_words: 2,
    };
    let rule = RedundantCommentsRule::new(&config);
    // "increment the counter" → after stops: ["increment", "counter"]
    // code "counter += 1" → ["counter", "1"]
    // overlap: 1/2 = 0.5, which is < 1.0
    let content = "// increment the counter\ncounter += 1;\n";
    let diags = rule.check_file(content, Path::new("test.rs"));
    assert!(diags.is_empty());
}
