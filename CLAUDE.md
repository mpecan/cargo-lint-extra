# CLAUDE.md — cargo-lint-extra

## Constitution

1. **Transparent in all** — users know exactly what they get. Rules are predictable, diagnostics are clear, defaults are documented.
2. **Simplicity is king** — solve the problem with the least complexity. No premature abstractions, no over-engineering.
3. **If it is not tested, it is not shipped** — every rule has unit tests, integration tests, and fixture files. No exceptions.
4. **Utility is what we want** — every feature must serve a real need. If Clippy or rustfmt already handles it, don't duplicate it.
5. **User first** — sensible defaults, zero-config experience, clear error messages.
6. **No gatekeeping** — contributions of all kinds are welcome. Keep the codebase approachable.

## Project overview

`cargo-lint-extra` is a configurable Rust linter that catches things rustfmt and Clippy don't: overly long lines, stale TODOs, oversized files, excessive inline comments, and suppressed warnings that deserve review.

- **Binary:** `cargo lint-extra` (clap-based cargo subcommand)
- **Config:** `.cargo-lint-extra.toml` (TOML, searched upward from target dir)
- **Output:** human-readable or JSON, exit code 0/1/2

## Architecture

| Module | Role |
|---|---|
| `src/main.rs` | CLI entry point, argument parsing, output formatting |
| `src/lib.rs` | Library re-exports |
| `src/config.rs` | TOML config loading with `#[serde(default)]` |
| `src/diagnostic.rs` | `Diagnostic` type and formatting (human/JSON) |
| `src/engine.rs` | File walking (`ignore`), parallel processing (`rayon`), rule orchestration |
| `src/suppression.rs` | Comment-based inline suppression (`// cargo-lint-extra:allow(...)`) |
| `src/rules/mod.rs` | `TextRule` and `AstRule` trait definitions |
| `src/rules/text/` | Text-based rules (line-by-line + whole-file) |
| `src/rules/ast/` | AST-based rules (via `syn`) |
| `tests/fixtures/` | Fixture files with known violations |
| `tests/integration_test.rs` | Engine-level integration tests |
| `tests/cli_test.rs` | CLI binary tests (exit codes, output) |

### Rule traits

- **`TextRule`** — `check_line(line, line_number, file)` and `check_file(content, file)`. Operates on raw text.
- **`AstRule`** — `check_file(syntax, file)`. Receives a parsed `syn::File`. Use `syn::visit::Visit` to walk the tree.

Both traits require `Send + Sync` for parallel execution.

### Engine flow

1. Walk files with `ignore` (respects `.gitignore`), always excludes `target/`
2. Apply `global.exclude` prefix patterns
3. Process files in parallel with `rayon`
4. Run text rules, then AST rules (AST parsing only when AST rules are enabled)
5. Filter out diagnostics suppressed by `// cargo-lint-extra:allow(...)` comments
6. Sort diagnostics by file, line, column

## Code standards

### Enforced limits

| Limit | Value | Enforced by |
|---|---|---|
| Line width | 100 chars | `.rustfmt.toml` |
| Function length | 60 lines | `clippy.toml` |
| Cognitive complexity | 15 | `clippy.toml` |
| Function arguments | 5 | `clippy.toml` |
| File length | 500 lines | `cargo-lint-extra` itself |

### Clippy rules (`Cargo.toml`)

- **Denied:** `unwrap_used`, `expect_used`, `panic`, `todo`
- **Warned:** `pedantic`, `nursery` groups
- **Allowed:** `module_name_repetitions`, `must_use_candidate`

Use `#[allow(...)]` only in test code. Prefer returning `Result` or using pattern matching over unwrapping.

### Formatting

- `rustfmt` with `max_width = 100`, edition 2024
- Run `cargo fmt` before committing

### Before every change

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

All three must pass clean.

## Conventions

### Commits

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`, `perf`. One logical change per commit.

### Rule naming

- Rule names are **kebab-case** (e.g., `inline-comments`, `allow-audit`)
- Config sections use kebab-case: `[rules.my-rule]`
- Config fields use snake_case: `max_ratio`, `soft_limit`

### Rule defaults

- **`Warn`** for rules that catch common issues most projects benefit from
- **`Allow`** for opinionated or noisy rules — let users opt in
- Every config field must have a `Default` impl with sensible values

### Error handling

- No `.unwrap()` or `.expect()` in production code (Clippy denies these)
- Use `Result` propagation or graceful fallbacks
- Exception: `Mutex::lock().unwrap()` is acceptable with a `#[allow]` and `/// # Panics` doc

### Test organization

- Unit tests: `#[cfg(test)]` module in the source file
- For large test suites: split with `#[path = "..."]` includes to stay under 500 lines
- Test modules use `#[allow(clippy::unwrap_used)]`
- Integration tests use fixture files in `tests/fixtures/`
- Fixture files are excluded from self-linting via `.cargo-lint-extra.toml`

## Adding a new rule

1. Create `src/rules/text/my_rule.rs` (or `src/rules/ast/` for AST rules)
2. Implement `TextRule` or `AstRule` trait with a kebab-case `name()`
3. Add config struct to `src/config.rs` with `#[serde(default)]` and a `Default` impl
4. Add the field to `RulesConfig` (kebab-case serde rename is automatic)
5. Wire it up in `src/engine.rs` — instantiate only if `level != Allow`
6. Add the rule name to `set_rule_level()` in `src/main.rs`
7. Add a test fixture in `tests/fixtures/`
8. Write unit tests and integration tests
9. Update README.md with rule documentation

## Self-linting

The project lints itself. Running `cargo run -- lint-extra .` should produce zero findings. The `.cargo-lint-extra.toml` excludes `tests/fixtures/` and any `#[path]`-included test files that contain intentionally flagged content.
