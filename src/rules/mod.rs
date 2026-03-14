pub mod ast;
pub mod text;

use crate::diagnostic::Diagnostic;
use std::path::Path;

pub trait TextRule: Send + Sync {
    fn name(&self) -> &'static str;

    fn check_line(&self, _line: &str, _line_number: usize, _file: &Path) -> Option<Diagnostic> {
        None
    }

    fn check_file(&self, _content: &str, _file: &Path) -> Vec<Diagnostic> {
        Vec::new()
    }
}

pub trait AstRule: Send + Sync {
    fn name(&self) -> &'static str;
    fn check_file(&self, syntax: &syn::File, content: &str, file: &Path) -> Vec<Diagnostic>;
}
