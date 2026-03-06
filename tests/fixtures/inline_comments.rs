// This fixture triggers inline-comments diagnostics.

/// Doc comment on function (should be fine)
fn over_commented() {
    // set up x variable
    // set up y variable
    // set up z variable
    // compute the result
    // prepare the output
    // finalize everything
    let x = 1;
    let y = 2;
    let result = x + y;
    println!("{result}");
}

fn consecutive_block() {
    let x = 1;
    // step one
    // step two
    // step three
    // step four
    let y = x + 1;
    let z = y + 1;
    let w = z + 1;
    println!("{w}");
}

fn clean_function() {
    let a = 1;
    let b = 2;
    let c = a + b;
    println!("{c}");
}
