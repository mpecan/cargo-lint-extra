// Test fixture for collect-then-iterate rule

fn collect_then_iter() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().iter().count();
}

fn collect_then_into_iter() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().into_iter().count();
}

fn collect_then_iter_mut() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().iter_mut().count();
}

fn collect_then_len() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().len();
}

fn collect_then_is_empty() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().is_empty();
}

fn collect_then_first() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().first();
}

fn collect_then_last() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().last();
}

// --- Clean examples (should NOT be flagged) ---

fn collect_then_push() {
    let mut v: Vec<i32> = vec![1].into_iter().collect();
    v.push(2);
}

fn no_collect() {
    let v = vec![1, 2, 3];
    let _ = v.iter().count();
}

fn collect_with_intermediate() {
    let _ = vec![1, 2, 3].into_iter().collect::<Vec<_>>().as_slice().iter();
}
