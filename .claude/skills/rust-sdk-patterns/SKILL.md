---
name: rust-sdk-patterns
description: Complete guide to writing Miden smart contracts with the Rust SDK. Covers #[component], #[note], #[tx_script] macros, storage patterns, native functions, asset handling, cross-component calls, P2ID note creation, and asset receiving via component methods. Use when writing, editing, or reviewing Miden Rust contract code.
---

# Miden Rust SDK Patterns

## Three Contract Types

### Account Component (`#[component]`)
Defines reusable logic and storage for accounts. Accounts are composed of one or more components.

See [counter-account/src/lib.rs](../../../contracts/counter-account/src/lib.rs) for a working example demonstrating `#[component]`, `StorageMap`, read/write methods, and felt arithmetic.

**Cargo.toml for accounts:** See [counter-account/Cargo.toml](../../../contracts/counter-account/Cargo.toml) for the required `crate-type`, `miden` dependency, `component` metadata, and `project-kind`.

### Note Script (`#[note]`)
Executes when a note is consumed by an account. Can call component methods on the consuming account.

See [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) for a working example demonstrating `#[note]`, `#[note_script]`, and cross-component calls.

**Cargo.toml for notes:** See [increment-note/Cargo.toml](../../../contracts/increment-note/Cargo.toml) for the required `miden` deps, cross-component dependencies, wit deps, and `project-kind = "note-script"`.

### Transaction Script (`#[tx_script]`)
One-off logic executed in the context of an account. Used for initialization, admin operations, etc.

```rust
#![no_std]
#![feature(alloc_error_handler)]
use miden::*;
use crate::bindings::Account;

#[tx_script]
fn run(_arg: Word, account: &mut Account) {
    account.initialize();
}
```

**Cargo.toml:** Same as account but with `project-kind = "tx-script"`.

## Storage Types

| Type | Usage | Read | Write |
|------|-------|------|-------|
| `Value` | Single Word slot (flags, simple state) | `.read() -> Word` | `.write(Word)` |
| `StorageMap` | Key-value mapping (balances, records) | `.get(&Word) -> Felt` | `.set(Word, Felt)` |

**Storage keys** are always `Word` (4 Felts). Use `Word::from_u64_unchecked(a, b, c, d)` or `Word::from([f0, f1, f2, f3])`.

## Native Function Modules

| Module | Key Functions | Purpose |
|--------|--------------|---------|
| `native_account::` | `add_asset(Asset)`, `remove_asset(Asset)`, `incr_nonce()` | Modify account vault/nonce |
| `active_account::` | `get_id() -> AccountId`, `get_balance(AccountId) -> Felt` | Query current account |
| `active_note::` | `get_storage() -> Vec<Felt>`, `get_assets() -> Vec<Asset>`, `get_sender() -> AccountId` | Query note being consumed |
| `output_note::` | `create(Tag, NoteType, Recipient) -> NoteIdx`, `add_asset(Asset, NoteIdx)` | Create output notes |
| `faucet::` | `create_fungible_asset(Felt) -> Asset`, `mint(Asset)`, `burn(Asset)` | Asset minting |
| `tx::` | `get_block_number() -> Felt`, `get_block_timestamp() -> Felt` | Transaction context |
| Intrinsics | `assert(bool)`, `assertz(Felt)`, `assert_eq(Felt, Felt)` | Validation |

## Asset Handling

Fungible asset Word layout: `[amount, 0, faucet_suffix, faucet_prefix]`

**Constructor**: `Asset::new(word)` creates an Asset from a Word.

See [miden-bank bank-account](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/contracts/bank-account/src/lib.rs) for complete asset handling patterns including deposit, withdrawal, and balance tracking.

```rust
// Access asset amount
let amount = asset.inner[0];

// Add asset to account vault (only from component methods, not note scripts — see pitfall P9)
native_account::add_asset(asset);

// Remove asset from account vault
native_account::remove_asset(asset.clone());
```

## P2ID Output Note Creation

To send assets to another account, create a P2ID (Pay-to-ID) output note. See [miden-bank bank-account](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/contracts/bank-account/src/lib.rs) `create_p2id_note()` for a complete working implementation.

## Cross-Component Dependencies

To call another component's methods from a note or tx script, two Cargo.toml sections are needed. See [increment-note/Cargo.toml](../../../contracts/increment-note/Cargo.toml) for a working example showing both `[package.metadata.miden.dependencies]` and `[package.metadata.component.target.dependencies]`.

Then import the bindings in your Rust code. See [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) line 13 for the import pattern: `use crate::bindings::miden::target_component::target_component;`

## Common Type Conversions

```rust
// Felt from integer
let f = felt!(42);                     // preferred for literals in contract code
let f = Felt::new(42);                 // construct a Felt from a u64
let f = Felt::from_u32(42);
let f = Felt::from_u64_unchecked(42);  // when value is known < field modulus

// Word from Felts
let w = Word::from([f0, f1, f2, f3]);
let w = Word::from_u64_unchecked(0, 0, 0, 1);
let w = Word::new([f0, f1, f2, f3]);

// Felt to u64 (for comparisons and arithmetic safety)
let n: u64 = f.as_u64();
```

## No-std Requirements

Every contract file must start with `#![no_std]` and `#![feature(alloc_error_handler)]`. See any contract in [contracts/](../../../contracts/) for the pattern.

If you need heap allocation (Vec, String, etc.):
```rust
extern crate alloc;
use alloc::vec::Vec;
```

## Asset Receiving via Component Methods

Note scripts cannot call `native_account::add_asset()` directly (see pitfall P9). The canonical pattern is for an account component to expose a public method that wraps `native_account::add_asset()`, and note scripts call that method via cross-component bindings.

See [miden-bank bank-account deposit()](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/contracts/bank-account/src/lib.rs) for the component side: the `deposit()` method validates the deposit, updates storage, and calls `native_account::add_asset()`.

See [miden-bank deposit-note](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/contracts/deposit-note/src/lib.rs) for the note side: the note script calls `bank_account::deposit()` via generated bindings.

## Validation Checklist

- [ ] `#![no_std]` and `#![feature(alloc_error_handler)]` at top of every contract
- [ ] `crate-type = ["cdylib"]` in Cargo.toml
- [ ] Correct `project-kind` in `[package.metadata.miden]`
- [ ] Cross-component deps in both `[package.metadata.miden.dependencies]` and `[package.metadata.component.target.dependencies]`
- [ ] Felt arithmetic validated before subtraction (see rust-sdk-pitfalls skill)
- [ ] Felt comparisons use `.as_u64()` (see rust-sdk-pitfalls skill)
