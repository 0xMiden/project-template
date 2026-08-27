---
name: rust-sdk-source-guide
description: Guide for advanced Miden smart contract development using source repo exploration. Covers AI development practices (Plan Mode, verification-driven development, context engineering, sub-agents) and maps Miden source repositories for discovering advanced patterns. Use when building complex multi-contract applications, novel note flows, or anything beyond basic SDK patterns.
---

# Advanced Miden Development: Source-Guided Context Engineering

## Development Approach

### 1. Plan Mode First

For any non-trivial smart contract application, start in Plan Mode before writing code.

- Explore Miden source repos to understand existing patterns
- Design the account/note architecture and present it to the user before implementing
- Identify which standard components can be reused vs what needs to be custom
- Map out the note flow: which accounts exist, what notes flow between them, what storage each needs

Rule of thumb: if the task involves more than one contract or a pattern not covered by the basic skills, plan first.

### 2. Verification-Driven Development

This is the single highest-leverage practice for AI-assisted Miden development.

**Build loop**: After every contract edit, invoke the verified isolated compiler directly: `"${CARGO_HOME:-$HOME/.cargo}/miden-v16-0.10.0-rc.1/bin/cargo-miden" miden build --manifest-path contracts/<name>/Cargo.toml --release`. The project's build hook derives and verifies that same absolute binary independently of ambient `PATH`. If the build fails:
1. Read the error message
2. Translate obvious SDK/compiler errors first:
   - `.as_u64()` -> `.as_canonical_u64()`
   - `Recipient::compute(...)` -> `note::build_recipient(...)`
   - `Value` -> `StorageValue<T>`
   - `StorageMap` -> `StorageMap<K, V>`
3. Search the source repos for a working example of the pattern that failed
4. Adapt the working pattern to your use case
5. Rebuild

**Test loop**: Write tests alongside contracts. Run `cargo test -p integration --release` (tests compile the contracts via `build_project_in_dir()`, so always build contracts before running them). When tests fail:
1. Check the error — is it a build error, a runtime assertion, or a proof failure?
2. For assertion failures: check felt arithmetic (modular wrapping) and storage slot naming
3. For unexpected behavior: compare your code against the closest working example in source repos

Never submit code that doesn't compile and pass tests. The verification loop is your quality guarantee.

### 3. Context Engineering with Source Repos

The basic skills (rust-sdk-patterns, rust-sdk-testing-patterns, miden-concepts, rust-sdk-pitfalls) cover standard patterns. For anything beyond those patterns, Miden's source repositories are the knowledge base.

**How to use source repos effectively**:
- Don't load entire repos into context. Use sub-agents to explore — they search, read relevant files, and summarize findings without filling the main conversation context.
- Read source files only when you need a specific answer (progressive disclosure)
- Look for working examples first, then adapt. Working code that compiles is more reliable than documentation.
- When you find a useful pattern in source, extract just what you need — the exact API call, the exact data layout, the exact test setup.

**Using sub-agents for exploration**:
- Launch an explore sub-agent with a specific question: "At compiler revision `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`, find how `#[account(...)]` generates and scopes one trait per referenced interface."
- The sub-agent searches, reads the relevant files, and returns a focused summary
- Your main context stays clean for implementation

### 4. Iterative Multi-Stage Development

Break complex applications into stages. Complete each before starting the next:

1. **Design** (Plan Mode) — Architecture, note flows, storage design
2. **Implement accounts** — Component structs, storage, methods
3. **Implement notes** — Note scripts, cross-component calls, input parsing
4. **Implement tx scripts** — Initialization, admin operations
5. **Write tests** — MockChain setup, multi-step execution, state verification
6. **Integrate** — Connect pieces, end-to-end test

When stuck at any stage: search the source repos for a similar working pattern. Adapt it, don't guess.

---

## Miden Source Repository Map

Clone these repos alongside your project for reference. Pin the exact refs before using any file as API evidence.

```bash
# Required: protocol layer — standard note types, account components, and MockChain
git clone --branch v0.16.0-rc.6 https://github.com/0xMiden/protocol.git ../protocol

# Required: client API for deployment and chain interaction
git clone --branch v0.16.0-rc.2 https://github.com/0xMiden/miden-client.git ../miden-client

# Required: guest SDK macros, examples, build support, and compiler pipeline.
git clone https://github.com/0xMiden/compiler.git ../compiler
git -C ../compiler checkout 2a5ebf830c910aa5f7bf53ee4df398915ab12f7a

# Required when inspecting MAST/package/VM APIs.
git clone --branch v0.29.1 https://github.com/0xMiden/miden-vm.git ../miden-vm

# Runtime-only reference for the exact DevNet node package.
git clone --branch v0.16.0-rc.1 https://github.com/0xMiden/miden-node.git ../miden-node
```

### Two version lines and MSRV

Do not conflate the contract-build line with the host/runtime line:

- **Contract build:** guest `miden = "=0.14.0-rc.1"`, `miden-sdk-build-script-support`, `cargo-miden`, and `midenc` come from immutable compiler revision `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`; both executables report `0.10.0-rc.1`. This compiler source resolves protocol rc.4 and VM 0.29 internally.
- **Host integration:** `miden-client`/SQLite store are `0.16.0-rc.2`, protocol/standards/testing are `0.16.0-rc.6`, and `miden-mast-package` is `0.29.1`. The DevNet node package is `0.16.0-rc.1` and its official source pins protocol rc.4.

The highest MSRV controls the checkout: compiler/guest work needs Rust 1.97; protocol and VM need 1.96.1; the client needs 1.96. `wasm32-wasip2` is required for contract compilation. The integration crate must not depend on the `cargo-miden` library: `build_project_in_dir()` launches the verified isolated binary and reads its emitted `.masp` with `miden-mast-package 0.29.1`.

### `compiler/` — The Rust-to-MASM Compiler

Contains the SDK that powers `#[component]`, `#[note]`, and `#[tx_script]` macros.

- **`examples/`** — working examples covering core SDK patterns: account components, note scripts, transaction scripts, authentication components (NoAuth, RPO Falcon512), wallets, and storage. These are the most reliable reference for "how to write X" questions. Note: there is no faucet example here — for faucet reference, use `crates/miden-standards/src/account/faucets/fungible/mod.rs` (the `FungibleFaucet` component) in the protocol repo, or the compiler's `tests/integration/src/sdk/base/faucet.rs` faucet binding test.
- **`sdk/sdk/MIGRATION.md`** — authoritative migration notes for contract macros, including `#[account_procedure]`, one generated trait per `#[account(...)]` interface, the required build-support wrapper, and embedded component WIT.
- **`sdk/build-script-support/`** and **`extra/templates/project/`** — authoritative only at the frozen revision for package-cache plumbing and the three packaging adaptations used by this project.

**WARNING**: Prefer `examples/` for contract API patterns. Read `sdk/sdk/MIGRATION.md`, `sdk/build-script-support/`, or template packaging only for the specific build/migration question they define; do not generalize unrelated compiler internals into contract APIs.

**Explore when**: Writing any new contract type, finding working code examples for patterns not covered by skills.

### `protocol/` — Protocol Layer and Standard Library

The protocol repo (`github.com/0xMiden/protocol`; primary crate `miden-protocol`). Contains the protocol specification, standard components, and standard note types.

- **`crates/miden-standards/`** — Standard note types (P2ID, P2IDE, SWAP, PSWAP, BURN, MINT) and standard account components (BasicWallet, FungibleFaucet, authentication components). Explore to understand note flow patterns and data layouts.
- **`crates/miden-protocol/asm/kernels/transaction/`** — The MASM transaction kernel. Every Rust SDK function (e.g., `native_account::add_asset`, `output_note::create`, `faucet::mint`) maps to a procedure defined here. Start with `api.masm` to find the procedure signature and stack contract, then read the implementation in `lib/` (e.g., `lib/output_note.masm`, `lib/account.masm`, `lib/epilogue.masm`). Useful for understanding exactly what happens under the hood -- for example, whether a function touches the vault, what the conservation check compares, or how note assets are tracked.
- **`crates/miden-tx/`** — Rust execution engine (executor, prover, host). Orchestrates transaction execution but rarely needed for understanding contract behavior. Explore only if debugging execution infrastructure or host-level behavior.
- **`crates/miden-testing/`** — MockChain implementation internals. Explore when you need to understand testing infrastructure beyond what the rust-sdk-testing-patterns skill covers.

**Note**: Protocol standard components are composed from host code. A Rust guest can call a component interface only through a built dependency package with embedded WIT and an `#[account(...)]` wrapper; do not assume a host-side standard component automatically supplies that guest interface. Use the frozen compiler `basic-wallet` example when you need a callable Rust component pattern.

**Explore when**: Understanding note flows, P2ID/SWAP/faucet data layouts, or what SDK functions actually do under the hood (via the kernel MASM).

### `miden-client/` — Client Library

The client repo (`github.com/0xMiden/miden-client`). Contains the Rust API for deploying contracts and interacting with the Miden network.

- Rust client for building transactions, syncing state, managing accounts and notes
- CLI tool source code for reference on client usage patterns

**Explore when**: Deploying contracts to an approved network, submitting transactions, syncing state, and managing notes on-chain.

---

## What to Explore for Each Contract Type

| Building This | Explore These Repos | What to Look For |
|---|---|---|
| Account component with storage | `compiler/` examples, this project's contracts | `StorageMap<K, V>` / `StorageValue<T>` patterns, `#[account_procedure]` declarations |
| Note script | `compiler/` examples, this project's contracts | `#[note_script]` pattern, generated account-interface traits, typed note fields |
| Transaction script | `compiler/` examples | `#[tx_script]` pattern, generated account-interface traits |
| Authentication component | `compiler/` examples | Auth component patterns (NoAuth, RPO Falcon512) |
| Faucet (token minting) | `protocol/` standards (`crates/miden-standards/src/account/faucets/fungible/mod.rs`), `compiler/` faucet binding test (`tests/integration/src/sdk/base/faucet.rs`) | `FungibleFaucet` component, `FungibleFaucet::builder()`, mint/burn pattern |
| P2ID output notes | `compiler/` examples, `protocol/` standards (data layouts) | `note::build_recipient`, script root, `output_note` creation |
| Swap notes | `protocol/` standards (data layouts) | SwapNote data layout, tag construction, payback flow |
| Multi-step tests | `protocol/crates/miden-testing/`, this project's integration test | Build transaction → execute → prove → verify, output note verification |
| Client deployment | `miden-client/` | TransactionRequestBuilder, sync, submit patterns |
| SDK function internals | `protocol/` kernel (`crates/miden-protocol/asm/kernels/transaction/`) | `api.masm` for procedure signatures, `lib/*.masm` for implementations |

---

## Common Advanced Patterns

These patterns go beyond what the basic skills cover. For each, the source repos contain working implementations.

### Multi-Component Accounts
Accounts can include standard components (BasicWallet, authentication) alongside custom logic at account creation time. Host code composes installed components; Rust guest calls additionally require a built dependency package with embedded WIT and an `#[account(...)]` wrapper. The frozen `compiler/` examples show the callable component side.

### Output Note Creation from Contracts
Create output notes (like P2ID) from within contract code. Requires building a recipient with `note::build_recipient(serial_num, script_root, storage)` and then using `output_note::create(...)`. Ground the exact call shape in the frozen compiler examples and the protocol standard note implementation.

### Note Storage Protocol
A `#[note]` struct's fields define the serialized Felt representation. The macro deserializes those fields in declaration order into `self` before `#[note_script]` runs; custom field types must implement the current felt-representation traits. Attached assets remain separate and their creation-time values are read with `active_note::get_initial_assets()`.

### Atomic Swaps
The standard SwapNote in `protocol/` (`crates/miden-standards/src/note/swap.rs`) creates a payback P2ID note automatically when consumed. Explore the SwapNote builder to understand tag construction, storage layout, and the payback mechanism.

### Account Initialization
Use `#[tx_script]` to initialize accounts before they accept operations. Mark the component method with `#[account_procedure]`, expose it through an `#[account(...)]` wrapper, and call it through the generated interface trait.

### Token Creation (Faucets)
Faucet accounts mint and burn tokens. The `protocol/` `FungibleFaucet` standard component (`crates/miden-standards/src/account/faucets/fungible/mod.rs`) shows how to create and manage fungible tokens; construct it via `FungibleFaucet::builder().name(..).symbol(..).decimals(..).max_supply(..).build()?`. There is no faucet example in `compiler/examples/`; for an SDK-level faucet binding reference use the compiler's `tests/integration/src/sdk/base/faucet.rs`.

### P2ID with Expiration (P2IDE)
Send assets with a deadline — the sender can reclaim after the block height passes. The `compiler/` p2ide-note example and `protocol/` P2IDE standard (`crates/miden-standards/src/note/p2ide.rs`) show the timelock pattern.
