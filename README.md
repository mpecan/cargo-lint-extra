# cargo-lint-extra

Configurable linting rules for Rust projects that fill the gap between rustfmt and Clippy.

`cargo-lint-extra` catches the things your formatter and Clippy don't — overly long lines, stale TODO comments, oversized files, and suppressed warnings that deserve a second look. All rules are configurable via a single TOML file and work out of the box with sensible defaults.

## Installation

```sh
cargo install cargo-lint-extra
```

## Quick start

Run in any Rust project:

```sh
cargo lint-extra
```

That's it. With zero configuration you get:

- **line-length** — warns on lines over 120 chars, errors over 200
- **file-length** — warns on files over 500 lines, errors over 1000
- **todo-comments** — flags `TODO`, `FIXME`, `HACK`, and `XXX` comments (allows `TODO(#123)` with issue references)
- **inline-comments** — flags functions with excessive `//` comments (ratio > 30% or > 3 consecutive)

## Usage

```
cargo lint-extra [OPTIONS] [PATH]
```

| Option | Description |
|---|---|
| `[PATH]` | Root directory to lint (default: `.`) |
| `--format human\|json` | Output format (default: `human`) |
| `--config <FILE>` | Path to config file |
| `--enable <RULES>` | Comma-separated rules to enable |
| `--disable <RULES>` | Comma-separated rules to disable |
| `-W`, `--warnings-as-errors` | Treat warnings as errors (exit 1 if any diagnostics) |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No findings, or warnings only (without `-W`) |
| `1` | One or more errors, or any finding with `-W` |
| `2` | Configuration or path error |

### Examples

```sh
# Lint the current project
cargo lint-extra

# JSON output for CI integration
cargo lint-extra --format json

# Enable the allow-audit rule
cargo lint-extra --enable allow-audit

# Disable specific rules
cargo lint-extra --disable line-length,todo-comments

# Lint a specific directory
cargo lint-extra src/

# Fail on any finding (warnings + errors)
cargo lint-extra -W
```

## Configuration

Create a `.cargo-lint-extra.toml` in your project root (or any parent directory — the tool searches upward):

```toml
[global]
exclude = ["target", "generated"]

[rules.line-length]
level = "warn"
soft_limit = 100
hard_limit = 160
url_exception = true

[rules.file-length]
level = "warn"
soft_limit = 400
hard_limit = 800

[rules.todo-comments]
level = "warn"
keywords = ["TODO", "FIXME", "HACK", "XXX"]
allow_with_issue = true

[rules.inline-comments]
level = "warn"
max_ratio = 0.3
max_consecutive = 3

[rules.file-header]
level = "warn"
required = "// Copyright 2025 My Company"

[rules.allow-audit]
level = "warn"
flagged = ["dead_code", "unused_variables", "unused_imports"]
```

Every field has a default — you only need to specify what you want to change.

### Rule levels

Each rule supports three levels:

| Level | Behaviour |
|---|---|
| `"allow"` | Rule is disabled |
| `"warn"` | Rule is enabled; findings are warnings (exit 0) |
| `"deny"` | Rule is enabled; **all** findings are errors (exit 1) |

For rules with soft/hard limits (`line-length`, `file-length`), hard limit violations are always errors regardless of `level`. Setting `level = "deny"` additionally promotes soft limit violations to errors — useful for strict per-rule enforcement without the global `-W` flag.

## Rules

### line-length

Checks that lines stay within configured limits. Lines exceeding the soft limit produce a warning; lines exceeding the hard limit produce an error.

Lines containing URLs in comments are exempt by default (controlled by `url_exception`).

| Setting | Default | Description |
|---|---|---|
| `soft_limit` | `120` | Character count that triggers a warning |
| `hard_limit` | `200` | Character count that triggers an error |
| `url_exception` | `true` | Exempt comment lines containing URLs |

### file-length

Warns when a file exceeds a soft line-count limit, and errors when it exceeds a hard limit.

| Setting | Default | Description |
|---|---|---|
| `soft_limit` | `500` | Line count that triggers a warning |
| `hard_limit` | `1000` | Line count that triggers an error |

> **Migration note:** The previous `max` field is accepted as a deprecated alias for `soft_limit`. If both are set, `soft_limit` takes precedence. A deprecation warning is printed to stderr.

### todo-comments

Detects leftover `TODO`, `FIXME`, `HACK`, and `XXX` comments. By default, TODOs with issue references like `TODO(#123)` or `TODO(JIRA-456)` are allowed.

| Setting | Default | Description |
|---|---|---|
| `keywords` | `["TODO", "FIXME", "HACK", "XXX"]` | Keywords to flag |
| `allow_with_issue` | `true` | Allow keywords followed by `(#N)` or `(KEY-N)` |

### file-header

Verifies that the first non-empty line of each file matches a required string. Disabled by default.

| Setting | Default | Description |
|---|---|---|
| `required` | none | Required header text |

### inline-comments

Flags excessive inline `//` comments inside function bodies. Helps catch AI-generated code that over-explains obvious logic. Doc comments (`///`, `//!`) are excluded. Functions with fewer than 4 meaningful lines are skipped.

Two checks are performed:
- **Ratio** — warns when the comment-to-code ratio exceeds the threshold
- **Consecutive** — warns when too many `//` comment lines appear in a row

| Setting | Default | Description |
|---|---|---|
| `max_ratio` | `0.3` | Maximum ratio of comment lines to total meaningful lines (0.0–1.0) |
| `max_consecutive` | `3` | Maximum number of consecutive `//` comment lines |

### allow-audit

Flags `#[allow(...)]` attributes that suppress specific lints, helping teams audit where warnings are being silenced. Disabled by default.

| Setting | Default | Description |
|---|---|---|
| `flagged` | `["dead_code", "unused_variables", "unused_imports"]` | Lint names to flag when suppressed |

## Test code overrides

You can configure different rule settings for test code using the `[test]` section. This lets you relax rules in tests (e.g., longer lines, allow `#[allow(dead_code)]` in test helpers) while keeping production code strict.

```toml
# Production rules
[rules.line-length]
soft_limit = 100

[rules.allow-audit]
level = "warn"

# Test overrides — only specify what's different
[test]
patterns = ["tests/", "benches/"]  # default
detect_cfg_test = true             # default — detect #[cfg(test)] blocks

[test.rules.line-length]
soft_limit = 150           # relaxed for tests
# hard_limit, url_exception inherited from prod

[test.rules.allow-audit]
level = "allow"            # disable for test files
```

### Override semantics

Per-field merge: specified fields in `[test.rules.*]` override the production value, unspecified fields inherit from the base `[rules.*]` config.

### Test file detection

Two mechanisms determine which code gets test rules:

| Mechanism | Default | Description |
|---|---|---|
| **Path patterns** | `["tests/", "benches/"]` | Prefix match against the relative file path. Patterns starting with `*` use suffix match (e.g., `*_test.rs`). Entire file uses test rules. |
| **`#[cfg(test)]` detection** | `true` | Within any file, `#[cfg(test)]` attributed items get test rules while the rest uses prod rules. |

### CLI interaction

`--enable`/`--disable` CLI overrides modify the base `[rules]` config. Since test overrides merge on top of the base, CLI overrides flow to test rules unless the `[test]` section explicitly overrides that rule.

## Inline suppression

You can suppress specific rules on individual lines, functions, or blocks using comment directives — no config changes needed.

### Syntax

```rust
// Suppress on the same line (inline)
let x = very_long_expression; // cargo-lint-extra:allow(line-length)

// Suppress the next line
// cargo-lint-extra:allow(todo-comments)
// TODO: this is fine for now

// Suppress an entire block (fn, mod, impl, struct, enum, trait)
// cargo-lint-extra:allow(inline-comments)
fn my_function() {
    // all inline-comments diagnostics suppressed in this function
}

// Suppress multiple rules
code // cargo-lint-extra:allow(line-length, todo-comments)

// Suppress all rules
code // cargo-lint-extra:allow
code // cargo-lint-extra:allow()
```

### Scopes

| Scope | Syntax | Effect |
|---|---|---|
| Inline | Code before `//` on the same line | Suppresses that line only |
| Next-line | Standalone `//` comment | Suppresses the next non-blank line |
| Block | Standalone comment before `fn`/`mod`/`impl`/`struct`/`enum`/`trait` | Suppresses the entire block (up to the matching `}`) |

### Limitations

- File-level diagnostics (like `file-length`) have no line number and cannot be suppressed via comments — use the config file instead.
- Only `//` line comments are supported for suppression (not `/* */`).

## CI integration

### GitHub Actions

Block on errors only (hard limit violations):

```yaml
- name: Lint (extras)
  run: cargo lint-extra
```

Block on any finding (strict mode):

```yaml
- name: Lint (extras)
  run: cargo lint-extra -W
```

## How it works

1. Discovers `.cargo-lint-extra.toml` by searching upward from the target directory
2. Walks the file tree using [ignore](https://crates.io/crates/ignore), respecting `.gitignore`
3. Processes files in parallel with [rayon](https://crates.io/crates/rayon)
4. Runs text-based rules (line-by-line and whole-file checks)
5. Parses files with [syn](https://crates.io/crates/syn) and runs AST-based rules (only when AST rules are enabled)
6. Filters out diagnostics suppressed by `// cargo-lint-extra:allow(...)` comments
7. Collects and sorts diagnostics by file and line number

## License

[MIT](LICENSE)
