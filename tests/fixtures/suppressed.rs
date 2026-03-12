// Fixture for comment-based inline suppression

// Inline suppression: this long line should not be flagged
let suppressed_long = "this line is way too long and exceeds the default maximum of one hundred and twenty characters which should trigger a lint warning"; // cargo-lint-extra:allow(line-length)

// Next-line suppression: the item below should not be flagged
// cargo-lint-extra:allow(todo-comments)
// TODO: this is suppressed

// This one is not suppressed and should be flagged
// TODO: this should be caught

// Block suppression on a function
// cargo-lint-extra:allow(line-length)
fn suppressed_function() {
    let long_in_block = "this line is way too long and exceeds the default maximum of one hundred and twenty characters which should trigger a lint warning";
}

// Unsuppressed long line should be flagged
fn unsuppressed() {
    let long = "this line is way too long and exceeds the default maximum of one hundred and twenty characters which should trigger a lint warning from our tool";
}
