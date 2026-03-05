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
- **file-length** — warns when a file exceeds 500 lines
- **todo-comments** — flags `TODO`, `FIXME`, `HACK`, and `XXX` comments (allows `TODO(#123)` with issue references)

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

### Exit codes

| Code | Meaning |
|---|---|
| `0` | No findings |
| `1` | One or more findings |
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
max = 400

[rules.todo-comments]
level = "warn"
keywords = ["TODO", "FIXME", "HACK", "XXX"]
allow_with_issue = true

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
| `"warn"` | Reports a warning (triggers exit code 1) |
| `"deny"` | Reports an error (triggers exit code 1) |

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

Warns when a file exceeds a maximum line count.

| Setting | Default | Description |
|---|---|---|
| `max` | `500` | Maximum number of lines |

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

### allow-audit

Flags `#[allow(...)]` attributes that suppress specific lints, helping teams audit where warnings are being silenced. Disabled by default.

| Setting | Default | Description |
|---|---|---|
| `flagged` | `["dead_code", "unused_variables", "unused_imports"]` | Lint names to flag when suppressed |

## CI integration

### GitHub Actions

```yaml
- name: Lint (extras)
  run: cargo lint-extra --format json > lint-extra.json

- name: Check lint results
  run: |
    if [ -s lint-extra.json ] && [ "$(cat lint-extra.json)" != "[]" ]; then
      echo "::error::cargo-lint-extra found issues"
      cat lint-extra.json | jq .
      exit 1
    fi
```

Or simply use the exit code:

```yaml
- name: Lint (extras)
  run: cargo lint-extra
```

## How it works

1. Discovers `.cargo-lint-extra.toml` by searching upward from the target directory
2. Walks the file tree using [ignore](https://crates.io/crates/ignore), respecting `.gitignore`
3. Processes files in parallel with [rayon](https://crates.io/crates/rayon)
4. Runs text-based rules (line-by-line and whole-file checks)
5. Parses files with [syn](https://crates.io/crates/syn) and runs AST-based rules (only when AST rules are enabled)
6. Collects and sorts diagnostics by file and line number

## License

[MIT](LICENSE)
