// Test fixture for magic-numbers rule

fn magic() {
    let x = 42;
    let y = 3.14;
    let z = 255;
    let w = 60;
}

fn allowed_numbers() {
    let a = 0;
    let b = 1;
    let c = 2;
    let d = 10;
    let e = 100;
    let f = 1000;
}

const MAX_RETRIES: u32 = 5;
static TIMEOUT: u64 = 30;

enum Color {
    Red = 3,
    Green = 5,
    Blue = 7,
}

fn ranges() {
    for i in 0..42 {
        let _ = i;
    }
}

#[cfg(test)]
mod tests {
    fn test_helper() {
        let x = 999;
    }
}
