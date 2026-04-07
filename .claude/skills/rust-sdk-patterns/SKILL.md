---
name: rust-sdk-patterns
description: Complete guide to writing Miden smart contracts with the Rust SDK. Covers #[component], #[note], #[tx_script] macros, storage patterns, native functions, asset handling, cross-component calls, and P2ID note creation. Use when writing, editing, or reviewing Miden Rust contract code.
---

# Miden Rust SDK Patterns

## Three Contract Types

### Account Component (`#[component]`)
Defines reusable logic and storage for accounts. Accounts are composed of one or more components.

See [counter-account/src/lib.rs](../../../contracts/counter-account/src/lib.rs) for a working example demonstrating `#[component]`, typed `StorageMap<Word, Felt>`, `get()`/`set()`, and felt arithmetic.

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
| `StorageValue<T>` | Single typed slot (flags, counters, IDs) | `.get() -> T` | `.set(T) -> T` |
| `StorageMap<K, V>` | Typed key-value mapping (balances, records) | `.get(K) -> V` | `.set(K, V) -> V` |

**Storage keys** are always `Word` (4 Felts). Use `Word::from_u64_unchecked(a, b, c, d)` or `Word::from([f0, f1, f2, f3])`.

## Native Function Modules

| Module | Key Functions | Purpose |
|--------|--------------|---------|
| `native_account::` | `add_asset(Asset)`, `remove_asset(Asset)`, `incr_nonce()` | Modify account vault/nonce |
| `active_account::` | `get_id() -> AccountId`, `get_balance(AccountId) -> Felt` | Query current account |
| `active_note::` | `get_storage() -> Vec<Felt>`, `get_assets() -> Vec<Asset>`, `get_sender() -> AccountId` | Query note being consumed |
| `note::` | `build_recipient(Word, Word, Vec<Felt>) -> Recipient` | Build note recipients from serial number, script root, and note storage |
| `output_note::` | `create(Tag, NoteType, Recipient) -> NoteIdx`, `add_asset(Asset, NoteIdx)` | Create output notes |
| `faucet::` | `create_fungible_asset(Felt) -> Asset`, `mint(Asset)`, `burn(Asset)` | Asset minting |
| `tx::` | `get_block_number() -> Felt`, `get_block_timestamp() -> Felt` | Transaction context |
| Intrinsics | `assert(bool)`, `assertz(Felt)`, `assert_eq(Felt, Felt)` | Validation |

## Asset Handling

`Asset` is now a two-word value:

```rust
pub struct Asset {
    pub key: Word,
    pub value: Word,
}
```

For fungible assets, the amount lives in `asset.value[0]`. The asset class / vault identity lives in `asset.key`.

```rust
// Access fungible amount
let amount = asset.value[0];

// Keep the asset key if you need to persist or compare the asset class
let asset_key = asset.key;

// Add asset to account vault
native_account::add_asset(asset);

// Remove asset from account vault
native_account::remove_asset(asset.clone());
```

## P2ID Output Note Creation

To send assets to another account, create a P2ID (Pay-to-ID) output note:

```rust
fn create_p2id_note(&mut self, serial_num: Word, asset: &Asset,
                     recipient_id: AccountId, tag: Felt, note_type: Felt) {
    let tag = Tag::from(tag);
    let note_type = NoteType::from(note_type);
    let script_root = Self::p2id_note_root(); // Hardcoded P2ID script digest

    // P2ID note storage: [recipient_suffix, recipient_prefix]
    let recipient = note::build_recipient(
        serial_num,
        script_root,
        vec![recipient_id.suffix, recipient_id.prefix],
    );

    let note_idx = output_note::create(tag, note_type, recipient);
    let _remaining_value = native_account::remove_asset(asset);
    output_note::add_asset(asset, note_idx);
}
```

`Recipient::compute(...)` was removed in `miden` 0.11. Use `note::build_recipient(...)` instead.

## Note Storage And Attached Assets

Notes receive explicit storage data as `Vec<Felt>`, accessed with `active_note::get_storage()`. Attached assets are separate and should be read with `active_note::get_assets()`.

```rust
let storage = active_note::get_storage();
let serial_num = Word::from([storage[0], storage[1], storage[2], storage[3]]);
let tag = Tag::from(storage[4]);
let note_type = NoteType::from(storage[5]);

let assets = active_note::get_assets();
let first_asset = assets[0];
```

## Cross-Component Dependencies

To call another component's methods from a note or tx script, two Cargo.toml sections are needed. See [increment-note/Cargo.toml](../../../contracts/increment-note/Cargo.toml) for a working example showing both `[package.metadata.miden.dependencies]` and `[package.metadata.component.target.dependencies]`.

Then import the bindings in your Rust code. See [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) line 13 for the import pattern: `use crate::bindings::miden::target_component::target_component;`

## Common Type Conversions

```rust
// Felt from integer
let f = felt!(42);
let f = Felt::new(42);
let f = Felt::from_u32(42);
let f = Felt::from_canonical_checked(42).unwrap();

// Word from Felts
let w = Word::from([f0, f1, f2, f3]);
let w = Word::new([f0, f1, f2, f3]);
let w = Word::from([0_u32, 0, 0, 1]);
let w = Word::try_from([0_u64, 0, 0, 1]).unwrap();

// Inspect a Word
let limbs: [Felt; 4] = w.into_elements();
let bytes: [u8; 32] = w.as_bytes();
let hex = w.to_hex();

// Felt to u64 (for comparisons and arithmetic safety)
let n: u64 = f.as_canonical_u64();
```

## No-std Requirements

Every contract file must start with `#![no_std]` and `#![feature(alloc_error_handler)]`. See any contract in [contracts/](../../../contracts/) for the pattern.

If you need heap allocation (Vec, String, etc.):
```rust
extern crate alloc;
use alloc::vec::Vec;
```

## Validation Checklist

- [ ] `#![no_std]` and `#![feature(alloc_error_handler)]` at top of every contract
- [ ] `crate-type = ["cdylib"]` in Cargo.toml
- [ ] Correct `project-kind` in `[package.metadata.miden]`
- [ ] Typed storage uses `StorageValue<T>` / `StorageMap<K, V>` with `get()` / `set()`
- [ ] Cross-component deps in both `[package.metadata.miden.dependencies]` and `[package.metadata.component.target.dependencies]`
- [ ] Felt arithmetic validated before subtraction (see rust-sdk-pitfalls skill)
- [ ] Felt comparisons use `.as_canonical_u64()` (see rust-sdk-pitfalls skill)
