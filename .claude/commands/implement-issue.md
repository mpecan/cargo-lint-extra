# Implement Issue Workflow

You are orchestrating a Plan → Implement → Review → Remediate → PR cycle for a GitHub issue,
with support for stacked PRs when issues form a linear dependency chain.

## Input

The user will provide a GitHub issue number: $ARGUMENTS

## Phase 1: Load Context & Determine Stacking

1. Run the context loader script to fetch all issue data in one shot:
   ```
   .claude/scripts/load-issue-context.sh <number>
   ```
   This fetches: issue details (title/body/labels/state), all comments, the full milestone issue list, dependency states (from `#N` mentions in the body), open PRs on `feat/` branches, and current git state. It replaces multiple `gh api` calls with a single invocation.
2. Read CLAUDE.md for project conventions (especially the "Adding a new rule" checklist)
3. From the script output, verify all dependencies are complete — either closed/merged to main, or listed in the `=== OPEN FEAT BRANCHES ===` section as part of the current stack

### Stacking Decision

Determine whether this issue should **stack** on a previous branch or **start fresh from main**.

**Algorithm:**
1. Find this issue's position in the milestone's implementation sequence (from the script's `Milestone` section — the current issue is marked `<-- this issue`)
2. Look at the **previous issue** in the sequence (by step number, not by dependency list)
3. If the previous issue has an **unmerged PR branch** that is an ancestor of this issue's dependencies → **stack on that branch**
4. If the previous issue is already **merged to main** → **branch from main**
5. If this issue has **multiple unmerged predecessors on different branches** → this is a **merge point**. STOP and tell the user the predecessor PRs must be merged first.

**In practice:**
The script's `=== OPEN FEATURE BRANCHES ===` section lists all open PRs on `feat/` branches with their head/base. Use this to determine the stacking target:

```
# If stacking:
git checkout <predecessor-branch>
git checkout -b feat/<this-issue>

# If fresh from main (works in worktrees where local main may not exist):
git fetch origin main
git checkout -b feat/<this-issue> origin/main
```

**Stacking stops when:** the next issue in the sequence introduces a new dependency that isn't in the current linear chain. At that point, the accumulated stack should be merged to main before continuing.

Report the stacking decision to the user:
```
Stacking: feat/<this-issue> → feat/<predecessor-issue> → ... → main
PR will target: feat/<predecessor-issue>
```
or:
```
Fresh branch: feat/<this-issue> from main
PR will target: main
```

**IMPORTANT:** Record the stacking decision (base branch and PR target) — this information must be included in the plan so it survives context compression.

## Phase 2: Plan

1. Enter plan mode with `EnterPlanMode`
2. Explore the codebase areas relevant to the issue:
   - Read existing rule implementations for the same rule type (text or AST) as reference patterns
   - Read `src/config.rs` to understand the config struct patterns
   - Read `src/engine.rs` to understand how rules are wired up
   - Read `src/main.rs` to understand `set_rule_level()` wiring
   - Read an existing test fixture and its corresponding integration test for the pattern
3. Design the implementation approach following the "Adding a new rule" checklist from CLAUDE.md:
   1. Create `src/rules/text/<rule>.rs` or `src/rules/ast/<rule>.rs`
   2. Implement `TextRule` or `AstRule` trait with kebab-case `name()`
   3. Add config struct to `src/config.rs` with `#[serde(default)]` and `Default` impl
   4. Add field to `RulesConfig` (kebab-case serde rename)
   5. Wire up in `src/engine.rs` — instantiate only if `level != Allow`
   6. Add rule name to `set_rule_level()` in `src/main.rs`
   7. Add test fixture in `tests/fixtures/`
   8. Write unit tests and integration tests
   9. Update README.md with rule documentation

### Plan Content Requirements

The plan must be **self-contained** — Claude Code's plan mode keeps the plan file fully loaded after context compression, so the plan becomes the primary reference for all subsequent phases.

#### Section 1: Branch & Stacking
```markdown
## Branch Strategy
- **Branch name:** `feat/<number>-<short-description>`
- **Base branch:** `main` | `feat/<predecessor-branch>`
- **PR target:** `main` | `feat/<predecessor-branch>`
- **Create command:** `git fetch origin <base> && git checkout -b feat/<branch-name> origin/<base>` (worktree-safe; use `git checkout <predecessor-branch> && git checkout -b feat/<branch-name>` when stacking on a local-only branch)
```

#### Section 2: Implementation Plan
The actual code changes — files to create/modify, config struct fields, rule detection logic, diagnostic messages.

#### Section 3: Implementation Order
Numbered steps following TDD:
1. Create the branch
2. Add config struct to `src/config.rs`
3. Create the rule implementation file
4. Write test fixture with known violations
5. Write unit tests (expect failures initially)
6. Implement the rule logic
7. Run tests to confirm they pass
8. Wire up in engine and main
9. Write integration tests
10. Update README.md
11. Run quality gate

#### Section 4: Post-Implementation Checklist

```markdown
## Post-Implementation Checklist

### Quality Gate (run all, fix any failures before review)
- [ ] `cargo fmt`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo run -- lint-extra .` (self-lint — must produce zero findings)

### Verify new rule works
- [ ] Run `cargo run -- lint-extra tests/fixtures/<rule_fixture>.rs` and confirm expected diagnostics
- [ ] Verify inline suppression works: `// cargo-lint-extra:allow(<rule-name>)`

### Multi-Agent Review (4 parallel agents)
Launch 4 review agents in parallel using the Agent tool:
1. **Acceptance Criteria** — verify each criterion from the issue with PASS/FAIL
2. **Code Quality** — file length (500 lines hard limit), function length (60 lines), naming, error handling (no unwrap/expect)
3. **Architecture** — follows existing rule patterns, config/engine/main wiring correct, `Send + Sync` traits satisfied
4. **Test Coverage** — unit tests, integration tests, fixture file, edge cases, meaningful assertions

Provide each agent with:
- The issue description and acceptance criteria
- The diff command: `git diff <base-branch>...HEAD`

### Remediation
- Fix all MAJOR findings, re-run quality gate
- Present MINOR findings to user for decision

### PR Creation
- Push: `git push -u origin <branch-name>`
- Create PR: `gh pr create --base <pr-target> --title "..." --body "..."`
- PR body must include: `Closes #<number>`, summary, test plan
- If stacked: include Stack section in PR body
- Wait for CI to pass (`gh pr checks <number> --watch`), fix failures if any
- Report PR URL and next issue in sequence (if any)
```

5. Write the plan using the plan mode tool
6. Exit plan mode and wait for user approval

**STOP: Wait for user to approve the plan before proceeding.**

## Phase 3: Implement

1. **Create the branch** per the Branch Strategy section of the approved plan
2. Create tasks for each implementation step using TaskCreate
3. Follow TDD:
   - Write tests first
   - Run tests to confirm they fail
   - Implement the code
   - Run tests to confirm they pass
4. Follow project standards from CLAUDE.md:
   - Line width: 100 chars (rustfmt)
   - Function length: 60 lines (clippy.toml)
   - Cognitive complexity: 15 (clippy.toml)
   - File length: 500 lines (cargo-lint-extra itself)
   - No `.unwrap()` or `.expect()` in production code
   - `#[allow(clippy::unwrap_used)]` only in test modules
5. Mark tasks complete as you go

### Rule Implementation Standards

- Rule `name()` must return kebab-case (e.g., `"clone-density"`)
- Config struct must derive `Deserialize, Default, Clone` with `#[serde(default)]`
- Config fields use snake_case, config sections use kebab-case
- Default level: `Warn` for broadly useful rules, `Allow` for opinionated ones
- Rule must implement `Send + Sync` for parallel execution
- Fixture file must contain both triggering and non-triggering code with clear comments

## Phase 4: Quality Gate

Run the full quality gate:

```
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- lint-extra .
```

The last command (self-lint) is critical — the project lints itself. If the new rule fires on the project's own code, either fix the code or adjust the rule's defaults/config.

If any failures, fix them before proceeding to the multi-agent review.

## Phase 5: Multi-Agent Review

Spawn **four review agents in parallel** using the Agent tool. Each agent reviews **only this issue's changes** from a different perspective.

**Important for stacked PRs:** The diff must be scoped to only this issue's commits, not the entire stack.
```
# For stacked PRs (base is predecessor branch):
git diff <predecessor-branch>...HEAD

# For fresh branches (base is main):
git diff origin/main...HEAD
```

**Important:** Agents should use `git diff` output as the source of truth, NOT read individual files directly. Reading files can produce stale results if edits happen between agent launch and file read, and a diff against `main` will appear empty if the changes aren't committed yet — make sure to commit (or at least stage) the implementation before spawning review agents.

### Agent 1: Acceptance Criteria Verification
**subagent_type:** `general-purpose`
```
Review the changes against the issue's acceptance criteria (from the issue's implementation checklist).
For each criterion, state PASS or FAIL with evidence (file:line references).
Flag any criteria that are partially met.
```

### Agent 2: Code Quality & Standards
**subagent_type:** `general-purpose`
```
Review the changes for:
- File length (hard limit 500 lines)
- Function length (60 lines max per clippy.toml)
- Cognitive complexity (15 max per clippy.toml)
- No .unwrap()/.expect() in production code
- #[allow(...)] only in test modules
- Naming: rule names kebab-case, config fields snake_case
- Config struct has #[serde(default)] and Default impl
Rate each file: CLEAN, MINOR, or MAJOR.
```

### Agent 3: Architecture & Integration
**subagent_type:** `general-purpose`
```
Review the changes for:
- Follows existing rule implementation patterns (compare with similar rules in src/rules/)
- Config wired correctly in src/config.rs (field in RulesConfig, serde rename)
- Engine wiring correct in src/engine.rs (instantiate only if level != Allow)
- CLI wiring correct in src/main.rs (set_rule_level)
- Rule implements Send + Sync
- Diagnostic messages are clear and follow existing format
Rate: CLEAN, MINOR, or MAJOR.
```

### Agent 4: Test Coverage & Correctness
**subagent_type:** `general-purpose`
```
Review the changes for:
- Unit tests in #[cfg(test)] module within the rule file
- Integration tests in tests/integration_test.rs
- Test fixture in tests/fixtures/ with both triggering and non-triggering code
- Edge cases tested
- Tests use #[allow(clippy::unwrap_used)]
- Fixture file excluded from self-linting via .cargo-lint-extra.toml if needed
Rate: CLEAN, MINOR, or MAJOR.
```

### Synthesis

After all four agents complete, produce a structured report:

```markdown
## Review Summary

### Verdict: PASS / PASS WITH MINOR ITEMS / NEEDS REMEDIATION

### Acceptance Criteria: X/Y passed

### Findings by severity

#### MAJOR (must fix before PR)
- [ ] <finding> — <file:line> (from: <agent>)

#### MINOR (fix or consciously skip)
- [ ] <finding> — <file:line> (from: <agent>)

#### NOTES (informational)
- <observation> (from: <agent>)
```

## Phase 6: Remediate

If any MAJOR findings:
1. Fix each MAJOR item
2. Re-run the quality gate (Phase 4)
3. Re-run only the relevant review agents for changed areas
4. Update the review summary

For MINOR findings, present them to the user and let them decide which to address.

**STOP: Present the review summary to the user. Ask for confirmation before creating PR.**
Use `AskUserQuestion` with options:
- "Create PR as-is" — proceed to Phase 7
- "Fix minor items first" — address selected minor items, then re-review
- "I want to review the changes myself first" — pause, let user inspect

## Phase 7: PR

1. Push the branch: `git push -u origin <branch-name>`
2. Create the PR with the correct base branch:
   ```
   # Stacked PR:
   gh pr create --base feat/<predecessor-issue> --title "..." --body "..."

   # Fresh from main:
   gh pr create --base main --title "..." --body "..."
   ```
3. PR body format:
   - Title: `feat: add <rule-name> rule` (matching conventional commits)
   - Body must include:
     - `Closes #<number>`
     - Summary of the rule, its config fields, and default behavior
     - Test plan
     - If stacked: a "Stack" section listing the chain
4. Wait for CI checks to complete:
   ```
   gh pr checks <pr-number> --watch
   ```
   - If CI fails, investigate, fix, push, and wait again
5. If there is a next issue in the milestone sequence that can be stacked, inform the user:
   ```
   Next in sequence: #<next-issue> — <title>
   This can be stacked on the branch just created. Run: /implement-issue <next-issue>
   ```

## Conventions

- Branch naming: `feat/<issue-number>-<short-description>` (e.g., `feat/6-clone-density`)
- Commit messages: conventional commits (`feat:`, `fix:`, `test:`, `refactor:`), include issue number
- One logical change per commit
- Always reference the issue number in commit messages

## Stacking Reference

### When stacking works (linear chain)
```
main ← feat/10-redundant-comments ← feat/6-clone-density ← feat/9-glob-imports
         PR #A (→main)               PR #B (→feat/10)       PR #C (→feat/6)
```
Each PR shows only its own diff. Merge from the bottom up.

### When stacking stops (fan-out / merge point)
If two issues share a dependency but are on different branches, that's a merge point.
The shared dependency must be merged to main before either can proceed.

### After merging a stacked PR
When a PR at the base of a stack is merged to main:
1. Update the next PR's base: `gh pr edit <number> --base main`
2. Or rebase: `git rebase --onto main feat/<merged-branch> feat/<next-branch>`
