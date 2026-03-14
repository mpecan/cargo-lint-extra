// Glob import that should trigger
use std::collections::*;

// Non-glob imports (should not trigger)
use std::fmt::Display;
use std::io::Read;

// Nested glob in group
use std::{io::*, fmt};

// Suppressed glob
// cargo-lint-extra:allow(glob-imports)
use std::path::*;

fn main() {}
