// Test fixture for string-alloc-in-loop rule

fn format_in_for() {
    let mut out = String::new();
    for i in 0..10 {
        out.push_str(&format!("{}", i));
    }
}

fn format_in_while() {
    let mut i = 0;
    while i < 10 {
        let _ = format!("{}", i);
        i += 1;
    }
}

fn format_in_loop() {
    let mut i = 0;
    loop {
        if i == 5 {
            break;
        }
        let _ = format!("{}", i);
        i += 1;
    }
}

fn to_string_in_loop() {
    for i in 0..10 {
        let _ = i.to_string();
    }
}

fn concat_in_loop() {
    let mut s = String::new();
    let t = String::from("x");
    for _ in 0..10 {
        s = s + &t;
    }
}

fn add_assign_in_loop() {
    let mut s = String::new();
    for _ in 0..10 {
        s += &String::from("x");
    }
}

fn format_in_nested_for() {
    for _ in 0..3 {
        for _ in 0..3 {
            let _ = format!("x");
        }
    }
}

// --- Clean examples (must NOT be flagged) ---

fn format_outside_loop() {
    let _ = format!("hello");
}

fn closure_inside_loop() {
    for _ in 0..3 {
        let g = || format!("x");
        let _ = g();
    }
}

fn fn_inside_loop() {
    for _ in 0..3 {
        fn helper() -> String {
            format!("x")
        }
        let _ = helper();
    }
}

fn numeric_add_in_loop() {
    let mut sum = 0;
    for i in 0..10 {
        sum = sum + i;
    }
    let _ = sum;
}

fn println_in_loop() {
    for i in 0..3 {
        println!("{}", i);
    }
}
