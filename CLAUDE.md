# Miden Project

This is a Miden smart contract project using the Rust SDK and compiler.

## Project Structure

- `contracts/` — Smart contracts (each is a separate crate, excluded from workspace)
  - Account components (`#[component]`)
  - Note scripts (`#[note]`)
  - Transaction scripts (`#[tx_script]`)
- `integration/` — Integration tests and deployment scripts (workspace member)

## Agent Rules

### Git Commits
- Never amend commits. Create fixup commits or new commits instead.
- Use commit messages exactly as specified by the user, verbatim.
- Never add Co-Authored-By or "Generated with Claude Code" to commits, PRs, or any content.
- Never push without explicit request.

### Workflow
- Enter plan mode for any non-trivial task (3+ steps or architectural decisions). If something goes wrong, stop and re-plan.
- Use subagents for research, exploration, and parallel analysis. One task per subagent.
- After any correction from the user, update `tasks/lessons.md` with the pattern.
- Never mark a task complete without proving it works.
- For non-trivial changes, ask "is there a more elegant way?" Skip this for simple fixes.
- When given a bug report, fix it autonomously.

### Task Management
1. Write plan to `tasks/todo.md` with checkable items
2. Check in with the user before starting implementation
3. Mark items complete as you go
4. Summarize changes at each step
5. Document results in `tasks/todo.md`
6. Capture lessons in `tasks/lessons.md` after corrections

### Core Principles
- **Simplicity first**: Make every change as simple as possible. Minimal code impact.
- **No laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimal impact**: Only touch what's necessary.

## Build & Test

Contracts are built individually with cargo-miden (not `cargo build`):
```
cargo miden build --manifest-path contracts/<name>/Cargo.toml --release
```

Tests run via the workspace:
```
cargo test -p integration --release
```

Always build contracts before running tests — tests compile contracts via `build_project_in_dir()`.

## SDK Quick Reference

See the working examples in this project:
- `contracts/counter-account/src/lib.rs` — Account component with StorageMap
- `contracts/increment-note/src/lib.rs` — Note script with cross-component call
- `integration/tests/counter_test.rs` — MockChain integration test

## Critical Pitfalls

**Felt arithmetic is modular (SECURITY CRITICAL)**: Subtraction wraps around the field modulus instead of panicking. ALWAYS validate before subtraction:
```rust
assert!(current.as_u64() >= amount.as_u64(), "Insufficient balance");
let result = current - amount;
```

**Felt comparisons are misleading for quantity logic**: `<`, `>`, `<=`, `>=` on Felt compare field elements, which differs from natural number ordering. For business logic (balances, amounts, counts), ALWAYS convert first: `a.as_u64() < b.as_u64()`

**No-std required**: All contracts must use `#![no_std]` and `#![feature(alloc_error_handler)]`. For heap allocation, use `extern crate alloc;` and `BumpAlloc`.

## Advanced Development

For complex applications beyond basic patterns (multi-contract apps, novel note flows, custom asset handling):

1. Clone Miden source repos alongside this project (see `rust-sdk-source-guide` skill for repo list and clone commands)
2. Use Plan Mode first — explore source repos to design the architecture before writing code
3. Use sub-agents to explore repos efficiently without filling main context

## Verification Workflow

After modifying contract code, always:
1. Write tests alongside contracts — tests are the primary verification, builds are the secondary check
2. Build the contract: `cargo miden build --manifest-path contracts/<name>/Cargo.toml --release`
3. Run tests: `cargo test -p integration --release`

## Post-Project Feedback

After completing a project (all tasks done, verification passed), generate a `feedback.md` file in the project root covering:
- What worked well with the agentic tooling (skills, hooks, CLAUDE.md guidance)
- What was missing, confusing, or incorrect
- Suggested improvements to skills, hooks, or documentation
- Patterns that should be captured as new skills or lessons
