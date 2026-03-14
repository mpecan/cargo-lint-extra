# Contributing to cargo-lint-extra

Contributions of all kinds are welcome — bug reports, new rules, documentation improvements, and code changes.

---

## Getting started

```sh
git clone https://github.com/mpecan/cargo-lint-extra
cd cargo-lint-extra
cargo build
cargo test
```

The project requires Rust 1.93.0 or later. See `rust-toolchain.toml` for the pinned version.

---

## Project structure

| Path | Description |
|---|---|
| `src/main.rs` | CLI entry point (clap-based cargo subcommand) |
| `src/lib.rs` | Library re-exports |
| `src/config.rs` | TOML configuration loading and structs |
| `src/diagnostic.rs` | `Diagnostic` type and output formatting |
| `src/engine.rs` | File walking, parallel processing, rule orchestration |
| `src/rule_registry.rs` | `declare_rules!` macro: generates config structs, rule builders, and override plumbing |
| `src/rules/mod.rs` | `TextRule` and `AstRule` trait definitions |
| `src/rules/text/` | Text-based rule implementations (each module is self-contained: config, override, rule, tests) |
| `src/rules/ast/` | AST-based rule implementations (same self-contained structure) |
| `tests/fixtures/` | Test fixture files with known violations |
| `tests/int_*.rs` | Per-rule integration tests (one file per rule/feature) |
| `tests/test_helpers/` | Shared integration test helpers (fixture runners) |
| `tests/cli_test.rs` | CLI binary tests (exit codes, output formats) |

---

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`, `perf`

Keep commits atomic — one logical change per commit.

---

## Code quality

Before opening a PR:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

All three must pass clean. CI runs the same checks.

### Lint rules

The project enforces strict Clippy rules (see `Cargo.toml`):

- `unwrap_used`, `expect_used`, `panic`, `todo` are **denied**
- `pedantic` and `nursery` lint groups are **warned**
- Use `#[allow(...)]` sparingly and only in test code

### Limits

- **Functions:** stay under 60 lines (Clippy enforces this)
- **Line width:** 100 characters (rustfmt enforces this)

---

## Adding a new rule

Each rule module is self-contained — config, test override, rule implementation, and unit tests all live in one file.

1. Create `src/rules/text/my_rule.rs` (or `src/rules/ast/` for AST rules) with:
   - `pub struct Config` with `#[serde(default)]` and a `Default` impl
   - `pub struct Override` with optional fields for test overrides
   - `pub fn apply_override(cfg: &mut Config, o: &Override)`
   - `pub struct Rule` implementing `TextRule` or `AstRule` with a kebab-case `name()`
   - `#[cfg(test)] mod tests` with unit tests
2. Add `pub mod my_rule;` to `src/rules/{text,ast}/mod.rs`
3. Add `my_rule: "my-rule",` to the `declare_rules!` invocation in `src/rule_registry.rs`
4. Add a test fixture in `tests/fixtures/`
5. Add integration tests in a new `tests/int_my_rule.rs` file

Only steps 2 and 3 touch shared files (one line each, append-only), so parallel PRs auto-merge.

### Text rules

Implement `TextRule`:
- `name()` — return the kebab-case rule name (e.g. `"my-rule"`)
- `check_line()` — check a single line, return `Option<Diagnostic>`
- `check_file()` — optionally check the whole file, return `Vec<Diagnostic>`

### AST rules

Implement `AstRule` using [syn](https://crates.io/crates/syn):
- `name()` — return the kebab-case rule name
- `check_file()` — receive a `&syn::File` and return `Vec<Diagnostic>`
- Use `syn::visit::Visit` to walk the syntax tree

### Rule design guidelines

- **Avoid Clippy overlap.** If Clippy already covers it well, don't duplicate it.
- **Default to `Allow`** for opinionated or noisy rules. Let users opt in.
- **Default to `Warn`** for rules that catch common issues most projects benefit from.
- **Make it configurable.** If there's a threshold or keyword list, expose it in the config.

---

## Tests

Every rule should have:

- **Unit tests** — `#[cfg(test)]` module in the rule file covering positive, negative, and edge cases
- **Integration tests** — engine-level test with a fixture file in `tests/fixtures/`
- **CLI tests** — if the rule affects exit codes or output format

Test modules use `#[allow(clippy::unwrap_used)]` since unwrap is appropriate in tests.

---

## Pull requests

- Target the `main` branch
- Include tests for any changed behaviour
- Keep PRs focused — one feature or fix per PR
- Reference relevant issues in the PR description (`Closes: #N`)

---

## License

By contributing you agree that your changes will be licensed under the [MIT License](LICENSE).
