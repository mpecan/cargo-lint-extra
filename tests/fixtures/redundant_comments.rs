fn example() {
    // increment the counter
    counter += 1;

    // return the result
    return result;

    // create new vector
    let vector = Vec::new();

    // set the name
    self.name = name;

    /// This is a doc comment — should not trigger
    let x = 1;

    // SAFETY: pointer is valid — directive, should not trigger
    unsafe { *ptr };

    // This handles the edge case where the buffer might overflow — explanatory
    self.flush();

    // TODO: fix this — directive, should not trigger
    broken_code();

    // ok
    result();

    // cargo-lint-extra:allow(redundant-comments)
    // return the value
    return value;
}
