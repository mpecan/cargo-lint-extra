use std::ops::Range;
use syn::spanned::Spanned;

/// Line ranges (1-based) of `#[cfg(test)]` blocks in a file.
pub struct TestLineRanges {
    ranges: Vec<Range<usize>>,
}

impl TestLineRanges {
    /// Parse content and extract `#[cfg(test)]` item line ranges.
    /// Skips expensive syn parsing if the file doesn't contain `cfg(test)`.
    pub fn from_content(content: &str) -> Self {
        if !content.contains("cfg(test)") {
            return Self { ranges: Vec::new() };
        }
        syn::parse_file(content).map_or_else(
            |_| Self { ranges: Vec::new() },
            |syntax| Self::from_syntax(&syntax),
        )
    }

    /// Extract `#[cfg(test)]` item line ranges from a pre-parsed `syn::File`.
    pub fn from_syntax(syntax: &syn::File) -> Self {
        let mut ranges = Vec::new();
        for item in &syntax.items {
            let attrs = item_attrs(item);
            if has_cfg_test_attr(attrs) {
                let attr_start = attrs
                    .first()
                    .map_or_else(|| item_span(item).start().line, |a| a.span().start().line);
                let end = item_end_line(item);
                ranges.push(attr_start..end + 1);
            }
        }
        Self { ranges }
    }

    /// Check if a given 1-based line number falls within a `#[cfg(test)]` block.
    pub fn is_test_line(&self, line: usize) -> bool {
        self.ranges.iter().any(|r| r.contains(&line))
    }

    pub const fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(i) => &i.attrs,
        syn::Item::Enum(i) => &i.attrs,
        syn::Item::ExternCrate(i) => &i.attrs,
        syn::Item::Fn(i) => &i.attrs,
        syn::Item::ForeignMod(i) => &i.attrs,
        syn::Item::Impl(i) => &i.attrs,
        syn::Item::Macro(i) => &i.attrs,
        syn::Item::Mod(i) => &i.attrs,
        syn::Item::Static(i) => &i.attrs,
        syn::Item::Struct(i) => &i.attrs,
        syn::Item::Trait(i) => &i.attrs,
        syn::Item::TraitAlias(i) => &i.attrs,
        syn::Item::Type(i) => &i.attrs,
        syn::Item::Union(i) => &i.attrs,
        syn::Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

fn item_span(item: &syn::Item) -> proc_macro2::Span {
    item.span()
}

fn item_end_line(item: &syn::Item) -> usize {
    // For modules with braces, use the brace close span for accurate end line.
    // For other items, fall back to the item's span end line.
    match item {
        syn::Item::Mod(m) => {
            if let Some((brace, _)) = &m.content {
                brace.span.close().end().line
            } else {
                item_span(item).end().line
            }
        }
        syn::Item::Fn(f) => f.block.brace_token.span.close().end().line,
        syn::Item::Impl(i) => i.brace_token.span.close().end().line,
        syn::Item::Struct(s) => {
            if let syn::Fields::Named(fields) = &s.fields {
                fields.brace_token.span.close().end().line
            } else {
                item_span(item).end().line
            }
        }
        _ => item_span(item).end().line,
    }
}

fn has_cfg_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("test") {
                found = true;
            }
            Ok(())
        });
        found
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cfg_test_module() {
        let content = "\
fn prod_function() {
    let x = 1;
}

#[cfg(test)]
mod tests {
    fn test_something() {
        assert!(true);
    }
}
";
        let ranges = TestLineRanges::from_content(content);
        assert!(!ranges.is_empty());
        // prod function lines
        assert!(!ranges.is_test_line(1));
        assert!(!ranges.is_test_line(2));
        assert!(!ranges.is_test_line(3));
        assert!(!ranges.is_test_line(4));
        // cfg(test) module: attribute on line 5, closing brace on line 10
        assert!(ranges.is_test_line(5));
        assert!(ranges.is_test_line(6));
        assert!(ranges.is_test_line(7));
        assert!(ranges.is_test_line(8));
        assert!(ranges.is_test_line(9));
        assert!(ranges.is_test_line(10));
        // after the block
        assert!(!ranges.is_test_line(11));
    }

    #[test]
    fn test_detect_multiple_cfg_test_blocks() {
        let content = "\
fn prod() {}

#[cfg(test)]
fn test_helper() {}

fn more_prod() {}

#[cfg(test)]
mod tests {
    fn t() {}
}
";
        let ranges = TestLineRanges::from_content(content);
        assert!(!ranges.is_empty());
        assert!(!ranges.is_test_line(1));
        assert!(ranges.is_test_line(3));
        assert!(ranges.is_test_line(4));
        assert!(!ranges.is_test_line(6));
        assert!(ranges.is_test_line(8));
        assert!(ranges.is_test_line(9));
        assert!(ranges.is_test_line(10));
        assert!(ranges.is_test_line(11));
    }

    #[test]
    fn test_no_cfg_test() {
        let content = "\
fn main() {
    println!(\"hello\");
}
";
        let ranges = TestLineRanges::from_content(content);
        assert!(ranges.is_empty());
        assert!(!ranges.is_test_line(1));
    }

    #[test]
    fn test_cfg_test_on_individual_function() {
        let content = "\
fn prod() {}

#[cfg(test)]
fn test_only_helper() {
    let x = 1;
}
";
        let ranges = TestLineRanges::from_content(content);
        assert!(!ranges.is_empty());
        assert!(!ranges.is_test_line(1));
        assert!(ranges.is_test_line(3));
        assert!(ranges.is_test_line(4));
        assert!(ranges.is_test_line(5));
        assert!(ranges.is_test_line(6));
    }

    #[test]
    fn test_from_syntax() {
        let content = "\
fn prod() {}

#[cfg(test)]
mod tests {}
";
        let syntax = syn::parse_file(content).unwrap();
        let ranges = TestLineRanges::from_syntax(&syntax);
        assert!(!ranges.is_empty());
        assert!(!ranges.is_test_line(1));
        assert!(ranges.is_test_line(3));
        assert!(ranges.is_test_line(4));
    }
}
