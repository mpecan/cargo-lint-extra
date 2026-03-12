fn main() {
    let x = "this line is absurdly long and exceeds the hard limit of two hundred characters, which means it should trigger an error-level diagnostic and cause the tool to exit with code 1 instead of 0 because it is a deny";
}
