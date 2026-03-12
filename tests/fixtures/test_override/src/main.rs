// This is a fixture file for testing test-code-specific rule overrides.
// The production code has a long line that should be flagged.
// The #[cfg(test)] module also has a long line that should use test rules.

fn production_function() {
    let _x = "this is a production line that is long enough to exceed the soft limit of one hundred and twenty characters easily here";
}

#[cfg(test)]
mod tests {
    fn test_something() {
        let _x = "this is a test line that is long enough to exceed the soft limit of one hundred and twenty characters easily here too";
    }
}
