fn main() {
    let x: Option<i32> = Some(42);
    let y = x.unwrap_or(0);
    println!("value: {}", y);
}
