#[allow(dead_code)]
fn unused_function() {}

#[allow(unused_variables)]
fn with_unused() {
    let x = 1;
}

#[allow(clippy::too_many_arguments)]
fn many_args() {}
