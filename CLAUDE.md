# Miden Project

This is a Miden smart contract project using the Rust SDK and compiler.

## Project Structure

- `contracts/`: Smart contracts (each is a separate crate, excluded from workspace)
  - Account components (`#[component]`)
  - Note scripts (`#[note]`)
  - Transaction scripts (`#[tx_script]`)
- `integration/`: Integration tests and deployment scripts (workspace member)

## Build & Test

This project uses an immutable v0.16 compiler pipeline rather than the ambient `cargo-miden` or
the v0.16 midenup channel. Install `cargo-miden` and `midenc` from compiler revision
`2a5ebf830c910aa5f7bf53ee4df398915ab12f7a` as shown in `README.md`. Derive and verify the isolated
binary in every shell:

```bash
MIDEN_CARGO_HOME="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"
MIDEN_V16_TOOL_ROOT="$MIDEN_CARGO_HOME/miden-v16-0.10.0-rc.1"
CARGO_MIDEN_BIN="$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"
test "$("$CARGO_MIDEN_BIN" miden --version)" = 'cargo-miden 0.10.0-rc.1'
test "$("$MIDEN_V16_TOOL_ROOT/bin/midenc" --version)" = 'midenc 0.10.0-rc.1'
```

Contracts are built individually with that exact binary:

```bash
"$CARGO_MIDEN_BIN" miden build \
  --manifest-path contracts/<name>/Cargo.toml --release
```

Tests run via the workspace:

```bash
cargo test -p integration --release
```

Always build contracts before running tests; tests compile contracts via `build_project_in_dir()`.
That helper independently derives and version-checks the same isolated compiler.

The post-edit hook also derives
`${CARGO_HOME:-$HOME/.cargo}/miden-v16-0.10.0-rc.1/bin/cargo-miden` on every invocation and rejects a
missing or mismatched compiler. It never selects `miden`, `cargo miden`, or `cargo-miden` from
ambient `PATH`.

### Per-contract build support

Every contract manifest must use the pinned guest SDK and build-support source:

```toml
[dependencies]
miden = { version = "=0.14.0-rc.1", git = "https://github.com/0xMiden/compiler", rev = "2a5ebf830c910aa5f7bf53ee4df398915ab12f7a" }

[build-dependencies]
miden-sdk-build-script-support = { git = "https://github.com/0xMiden/compiler", rev = "2a5ebf830c910aa5f7bf53ee4df398915ab12f7a" }
```

Each contract's `build.rs` is exactly:

```rust
fn main() {
    miden_sdk_build_script_support::prepare_package_cache();
}
```

For plain `cargo check` or IDE analysis, run Cargo from the contract directory so its
`.cargo/config.toml` is discovered, set `CARGO_MIDEN` to the verified absolute binary, and use a
checkout-private target directory:

```bash
PROJECT_ROOT="$PWD"
PLAIN_CARGO_TARGET="$PROJECT_ROOT/target/plain-cargo-v16"
(
  cd contracts/counter-account
  env -u MIDENC_PACKAGE_CACHE \
    CARGO_MIDEN="$CARGO_MIDEN_BIN" \
    CARGO_TARGET_DIR="$PLAIN_CARGO_TARGET" \
    cargo check --release
)
```

Do not set `MIDENC_PACKAGE_CACHE` manually. `prepare_package_cache()` stages dependency packages
under the contract's build output and exports the selected cache to macro expansion.

## SDK Quick Reference

See the working examples in this project:
- `contracts/counter-account/src/lib.rs`: Account component with typed `StorageMap<Word, Felt>`
- `contracts/increment-note/src/lib.rs`: Note script with cross-component call
- `integration/tests/counter_test.rs`: MockChain integration test

Common cargo commands:
- `cargo build -p integration --bin increment_count --release`: build the project's single binary
- `cargo clean -p integration`: run after editing shared library code (e.g. `helpers.rs`) before re-running tests, to avoid stale compiled binaries

`increment_count` currently uses the helper's temporary hard-coded DevNet endpoint
`https://rpc.devnet.miden.io`. Run it from `integration/` because its contract and store paths are
relative to that directory. It creates public DevNet accounts, adds a sender key to the existing
keystore, and submits transactions; returned IDs are submission evidence, not proof of finality.

## Critical Pitfalls

**Felt arithmetic is modular (SECURITY CRITICAL)**: Subtraction wraps around the field modulus instead of panicking. ALWAYS validate before subtraction:
```rust
assert!(
    current.as_canonical_u64() >= amount.as_canonical_u64(),
    "Insufficient balance"
);
let result = current - amount;
```

**Felt comparisons are misleading for quantity logic**: `<`, `>`, `<=`, `>=` on Felt compare field elements, which differs from natural number ordering. For business logic (balances, amounts, counts), ALWAYS convert first: `a.as_canonical_u64() < b.as_canonical_u64()`

**No-std required**: All contracts must use `#![no_std]` and `#![feature(alloc_error_handler)]`. For heap allocation, use `extern crate alloc;` and `BumpAlloc`.

## Advanced Development

For complex applications beyond basic patterns (multi-contract apps, novel note flows, custom asset handling):

1. Clone Miden source repos alongside this project (see `rust-sdk-source-guide` skill for repo list and clone commands)
2. Use Plan Mode first; explore source repos to design the architecture before writing code
3. Use sub-agents to explore repos efficiently without filling main context

## Verification Workflow

After modifying contract code, always:
1. Write tests alongside contracts; tests are the primary verification, builds are the secondary check
2. Build the contract: `"$CARGO_MIDEN_BIN" miden build --manifest-path contracts/<name>/Cargo.toml --release`
3. Run tests: `cargo test -p integration --release`
