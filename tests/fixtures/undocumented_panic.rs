// Test fixture for undocumented-panic rule

fn bad_unwrap() {
    let x: Option<i32> = Some(1);
    let _v = x.unwrap();
}

fn bad_expect() {
    let x: Option<i32> = Some(1);
    let _v = x.expect("oops");
}

fn bad_indexing() {
    let arr = [1, 2, 3];
    let _v = arr[0];
}

fn good_unwrap_preceding_line() {
    let x: Option<i32> = Some(1);
    // PANIC: x is always Some
    let _v = x.unwrap();
}

fn good_unwrap_inline() {
    let x: Option<i32> = Some(1);
    let _v = x.unwrap(); // PANIC: always Some
}

fn safe_alternatives() {
    let x: Option<i32> = Some(1);
    let _v = x.unwrap_or(0);
    let _v2 = x.unwrap_or_default();
}

#[test]
fn standalone_test_skipped() {
    let x: Option<i32> = Some(1);
    let _v = x.unwrap();
}

#[cfg(test)]
mod tests {
    fn test_code_skipped() {
        let x: Option<i32> = Some(1);
        let _v = x.unwrap();
    }
}
