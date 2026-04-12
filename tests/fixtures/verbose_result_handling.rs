// Test fixture for verbose-result-handling rule

// --- Triggering (6 total) ---

fn try_op_result() -> Result<i32, String> {
    let x = match parse_int() {
        Ok(x) => x,
        Err(e) => return Err(e),
    };
    Ok(x)
}

fn try_op_with_into() -> Result<i64, String> {
    let x: i64 = match parse_int() {
        Ok(x) => x,
        Err(e) => return Err(e.into()),
    };
    Ok(x)
}

fn if_let_some() {
    match get_opt() {
        Some(x) => {
            let _ = x;
        }
        None => {}
    }
}

fn if_let_reverse() {
    match get_opt() {
        None => {}
        Some(x) => {
            let _ = x;
        }
    }
}

fn map_some() -> Option<i32> {
    match get_opt() {
        Some(x) => Some(x + 1),
        None => None,
    }
}

fn map_result() -> Result<i32, String> {
    match parse_int() {
        Ok(x) => Ok(x + 1),
        Err(e) => Err(e),
    }
}

// --- Clean (must NOT fire) ---

fn three_arms(x: i32) {
    match x {
        1 => {}
        2 => {}
        _ => {}
    }
}

fn match_with_guard() {
    match get_opt() {
        Some(x) if x > 0 => {}
        _ => {}
    }
}

fn nonidentity_ok() -> i32 {
    // Ok body is not identity; no Ok wrapper on the transform — no pattern fires
    match parse_int() {
        Ok(x) => x * 2,
        Err(_) => 0,
    }
}

fn nontrivial_none() {
    match get_opt() {
        Some(x) => {
            let _ = x;
        }
        None => {
            let _ = 0;
        }
    }
}

fn transformed_err() -> Result<i32, String> {
    // Err arm is NOT a passthrough — not flagged by Pattern 3
    match parse_int() {
        Ok(x) => Ok(x + 1),
        Err(e) => Err(format!("!{e}")),
    }
}

fn enum_match(c: Color) {
    match c {
        Color::R => {}
        Color::G => {}
        Color::B => {}
    }
}

// --- Helpers for the fixture ---

fn parse_int() -> Result<i32, String> {
    Ok(0)
}

fn get_opt() -> Option<i32> {
    None
}

enum Color {
    R,
    G,
    B,
}
