---
name: rust-sdk-patterns
description: Complete guide to writing Miden smart contracts with the Rust SDK. Covers the three-part #[component_storage]/#[component] account-component pattern, #[note]/#[note_script] notes, #[tx_script] scripts, the #[account(...)] wrapper, storage patterns, native functions, asset handling, cross-component calls, P2ID note creation, and asset receiving via component methods. Use when writing, editing, or reviewing Miden Rust contract code.
---

# Miden Rust SDK Patterns

## Three Contract Types

### Account Component (three-part pattern)
Defines reusable logic and storage for accounts. Accounts are composed of one or more components.

An account component is written as **three parts** — the storage struct is annotated `#[component_storage]`, and `#[component]` applies to the API trait and the impl block:

1. `#[component_storage]` on the **storage struct** — declares typed `#[storage(...)]` fields and derives slot names.
2. `#[component]` on a **trait** — the component's exported API (this is the source of the generated WIT interface).
3. `#[component]` on the **`impl Trait for Storage`** block — the behavior, wired to the guest bindings.

```rust
#![no_std]
#![feature(alloc_error_handler)]
use miden::{component, component_storage, felt, Felt, StorageMap, Word};

#[component_storage]
struct CounterContractStorage {
    #[storage(description = "counter contract storage map")]
    count_map: StorageMap<Word, Felt>,
}

#[component]
trait CounterContract {
    fn get_count(&self) -> Felt;
    fn increment_count(&mut self) -> Felt;
}

#[component]
impl CounterContract for CounterContractStorage {
    fn get_count(&self) -> Felt {
        let key = Word::new([felt!(0), felt!(0), felt!(0), felt!(1)]);
        self.count_map.get(key)
    }

    fn increment_count(&mut self) -> Felt {
        let key = Word::new([felt!(0), felt!(0), felt!(0), felt!(1)]);
        let current_value: Felt = self.count_map.get(key);
        let new_value = current_value + felt!(1);
        self.count_map.set(key, new_value);
        new_value
    }
}
```

Only the trait's methods are exported to WIT. Inherent (`impl CounterContractStorage`) methods stay private to the contract — use them for helpers like key derivation.

See [counter-account/src/lib.rs](../../../contracts/counter-account/src/lib.rs) for the complete working example demonstrating the three-part pattern, typed `StorageMap<Word, Felt>`, `get()`/`set()`, and felt arithmetic.

**Project metadata for accounts:** See [counter-account/miden-project.toml](../../../contracts/counter-account/miden-project.toml) for `[lib] kind = "account-component"`, the `namespace` (`miden:counter-account/counter-contract@0.1.0`), and `supported-types` under `[package.metadata.miden]`. The `Cargo.toml` (see [counter-account/Cargo.toml](../../../contracts/counter-account/Cargo.toml)) only needs `crate-type = ["cdylib"]` and the `miden` dependency.

### Note Script (`#[note]` / `#[note_script]`)
Executes when a note is consumed by an account. Can call component methods on the consuming account.

A note is two parts: a `#[note]` struct (the note inputs type) and a `#[note]` `impl` block containing exactly one `#[note_script]` entrypoint. The entrypoint takes `self` **by value**, exactly one `Word` argument, and optionally a single reference to an `#[account(...)]` wrapper (`&MyAccount` or `&mut MyAccount`). The consuming account is declared separately with `#[account(...)]`.

```rust
#![no_std]
#![feature(alloc_error_handler)]
use miden::*;

// The native (active) account this note runs against: exposes the
// counter-account `CounterContract` component's methods on the wrapper.
#[account(counter_account::CounterContract)]
pub struct Wallet;

#[note]
struct IncrementNote;

#[note]
impl IncrementNote {
    #[note_script]
    fn run(self, _arg: Word, account: &mut Wallet) {
        let initial_value = account.get_count();
        account.increment_count();
        let expected_value = initial_value + Felt::from_u32(1);
        let final_value = account.get_count();
        assert_eq(final_value, expected_value);
    }
}
```

See [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) for the working example demonstrating `#[note]`, `#[note_script]`, the `#[account(...)]` wrapper, and a cross-component call.

**Project metadata for notes:** See [increment-note/miden-project.toml](../../../contracts/increment-note/miden-project.toml) for `[lib] kind = "note"`, the `namespace` (`miden:increment-note/miden-increment-note@0.1.0`), the path dependency on the called component (`counter-account = { path = "../counter-account" }`), and the cross-component `[package.metadata.miden.dependencies]` WIT entry.

### Transaction Script (`#[tx_script]`)
One-off logic executed in the context of an account. Used for initialization, admin operations, etc.

`#[tx_script]` annotates a free `fn run`. Its signature is `fn run(arg: Word)` or `fn run(arg: Word, account: &mut MyAccount)` where `MyAccount` is an `#[account(...)]` wrapper. You declare the account wrapper yourself with `#[account(...)]`, and the macro instantiates it as the active account.

```rust
#![no_std]
#![feature(alloc_error_handler)]
use miden::*;

// The account this tx-script runs against: the counter-account `CounterContract` component.
#[account(counter_account::CounterContract)]
pub struct Wallet;

#[tx_script]
fn run(_arg: Word, account: &mut Wallet) {
    account.increment_count();
}
```

**Project metadata for tx scripts:** Like a note, but `[lib] kind = "tx-script"` and `namespace = "miden:base/transaction-script@1.0.0"`.

## Storage Slot Naming

Storage slot names are part of the on-chain storage ABI and are derived as:

```
<package_name_snake>::<interface_segment_snake>::<field_name>
```

The **middle segment is the interface segment of the `[lib].namespace`** in `miden-project.toml` (the part between the last `/` and `@`), snake-cased — **not** the snake-cased struct name. This deliberately decouples slot names from private Rust renames.

Example: package `counter-account` + `namespace = "miden:counter-account/counter-contract@0.1.0"` + field `count_map` derives slot `counter_account::counter_contract::count_map` (see [integration/src/helpers.rs](../../../integration/src/helpers.rs) `counter_storage_slot()` and [counter_test.rs](../../../integration/tests/counter_test.rs)). The version suffix (`@0.1.0`) is ignored so the slot name stays stable. Slots are derived from the slot name; there is no `slot(...)` attribute. See the rust-sdk-pitfalls skill (P5) for more on slot naming.

## Storage Types

| Type | Usage | Read | Write |
|------|-------|------|-------|
| `StorageValue<T>` | Single typed slot (flags, counters, IDs) | `.get() -> T` | `.set(T) -> T` |
| `StorageMap<K, V>` | Typed key-value mapping (balances, records) | `.get(K) -> V` | `.set(K, V) -> V` |

## Native Function Modules

| Module | Key Functions | Purpose |
|--------|--------------|---------|
| `native_account::` | `add_asset(Asset) -> Word`, `remove_asset(Asset) -> Word`, `incr_nonce() -> Felt`, `get_id() -> AccountId` | Modify current account vault/nonce |
| `active_account::` | `get_id() -> AccountId`, `get_balance(Word) -> Felt` | Query current account (`get_balance` takes the asset key word, not an AccountId) |
| `active_note::` | `get_storage() -> Vec<Felt>`, `get_assets() -> Vec<Asset>`, `get_sender() -> AccountId` | Query note being consumed |
| `note::` | `build_recipient(Word, Word, Vec<Felt>) -> Recipient` | Build note recipients from serial number, script root, and note storage |
| `output_note::` | `create(Tag, NoteType, Recipient) -> NoteIdx`, `add_asset(Asset, NoteIdx)` | Create output notes |
| `faucet::` | `create_fungible_asset(Felt) -> Asset`, `mint(Asset)`, `burn(Asset)` | Asset minting |
| `tx::` | `get_block_number() -> Felt`, `get_block_timestamp() -> Felt` | Transaction context |
| Intrinsics | `assert(Felt)`, `assertz(Felt)`, `assert_eq(Felt, Felt)` | Validation (`assert` fails unless the felt equals 1; `assertz` fails unless it equals 0) |

## Asset Handling

`Asset` is a two-word value (`key` + `value`):

**Constructor**: `Asset::new(key, value)` builds an Asset from its vault key word and value word (the arguments are `impl Into<Word>`, so e.g. `Asset::new(key_word, value_word)` or from `[Felt; 4]`).

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

// Add asset to account vault (only from component methods, not note scripts — see pitfall P11)
native_account::add_asset(asset);

// Remove asset from account vault (Asset is Copy, no clone needed)
native_account::remove_asset(asset);
```

## P2ID Output Note Creation

To send assets to another account, create a P2ID (Pay-to-ID) output note. The sequence is:

1. Build the recipient with `note::build_recipient(serial_number, script_root, inputs)`.
2. Create the note with `output_note::create(tag, note_type, recipient)`, which returns a `NoteIdx`.
3. Move the asset out of the vault and onto the note with `native_account::remove_asset(asset)` + `output_note::add_asset(asset, note_idx)`.

Because a note script cannot call `native_account::*` (pitfall P11), P2ID creation lives inside an account-component method. See the rust-sdk-pitfalls skill for the exact constants and safety rules: P8 (`note::build_recipient`), P9 (P2ID script root — prefer `script_root()`, do not hardcode), and P10 (constructing `NoteType` via `NoteType::from(felt!(...))`).

## Cross-Component Dependencies

To call another component's methods from a note or tx script, declare the dependency in your `miden-project.toml` in **two places**:

- `[dependencies]` — a normal path (or registry) dependency on the component crate: `counter-account = { path = "../counter-account" }`.
- `[package.metadata.miden.dependencies]` — the generated WIT for the component: `counter-account = { wit = "../counter-account/target/generated-wit/" }`. The WIT is produced by building the dependency component first.

See [increment-note/miden-project.toml](../../../contracts/increment-note/miden-project.toml) for a working example showing both sections.

Then expose the dependency's methods on the consuming account by declaring an `#[account(package::Interface)]` wrapper (`#[account(counter_account::CounterContract)] pub struct Wallet;`) and calling methods on the injected `account` parameter. The package name is the dependency's Rust-style name (`-` replaced with `_`, so `counter-account` → `counter_account`) and `Interface` is its exported WIT interface in UpperCamelCase (`CounterContract`). See [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs).

## Common Type Conversions

```rust
// Felt from integer
let f = felt!(42);                     // preferred for literals in contract code
let f = Felt::new(42).unwrap();        // fallible: Felt::new returns Result<Felt, _>
let f = Felt::new_unchecked(42);       // infallible, non-reducing form
let f = Felt::from_u32(42);            // infallible (u32 always fits)
let f = Felt::from_canonical_checked(42).unwrap(); // returns Option<Felt>

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

## Cross-Component Note Pattern

A note script reads from `active_note::*` and forwards work to a public account-component method through the `#[account(...)]` wrapper. This is the canonical pattern for any note that updates account state, because note scripts cannot call `native_account::*` directly (see `rust-sdk-pitfalls` skill, P11).

The `#[note]` macro deserializes the note's inputs into the typed note struct, so serialized note storage is turned into typed fields before the script runs. The `#[note_script]` method receives the deserialized note as `self` (by value) and never indexes a raw Felt slice manually. Alongside the required `Word` arg, the method may optionally accept an `#[account(...)]` wrapper reference (`&Wallet` or `&mut Wallet`). See [compiler/sdk/base-macros/src/lib.rs](https://github.com/0xMiden/compiler/blob/main/sdk/base-macros/src/lib.rs) for the macro contract and [compiler/sdk/base-macros/src/note.rs](https://github.com/0xMiden/compiler/blob/main/sdk/base-macros/src/note.rs) for the generated deserialization (each named field is read via `<T as miden::felt_repr::FromFeltRepr>::from_felt_repr(...)` and EOF is asserted at the end).

Supported field types include `Felt`, the unsigned integer scalars (`u64`, `u32`, `u8`), `bool`, `Option<T>`, and `Vec<T>` via the `FromFeltRepr` trait (`compiler/sdk/field-repr/repr/src/lib.rs`), plus any user type that opts in with `#[derive(FromFeltRepr)]` (this is how `AccountId` supports the macro — see `compiler/sdk/base-sys/src/bindings/types.rs`). Do **not** use `Asset` or `Word` directly as note struct fields; those types do not currently derive `FromFeltRepr`. If you need asset-shaped data inside the note, flatten it into supported scalar fields and reconstruct inside the script, or keep it on the side as a separate `active_note::get_assets()` read.

For the Cargo.toml / `miden-project.toml` wiring (cross-component dependencies + `#[account(...)]` wrapper), see "Cross-Component Dependencies" above. See [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) for the project-template's local example of the `#[note] struct + #[note] impl` macro form.

**Storage-free case** (unit struct, calls the account wrapper): declare a unit struct (`#[note] struct IncrementNote;`). The script receives the `#[account(...)]` wrapper and calls component methods on it — the counter's [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) is exactly this shape (`account.get_count()` / `account.increment_count()`). For a note that forwards assets, read `active_note::get_sender()` and iterate `active_note::get_assets()`, calling the component method per asset through the wrapper. The macro still generates the deserialization wrapper; for a unit struct it only asserts the note-input Felt slice is empty.

**Typed-storage case** (note carries scripted data): declare named fields on the note struct. The macro deserializes them in declaration order, and the script accesses them via `self.<field>`. Illustrative shape:

```rust
#[account(counter_account::CounterContract)]
pub struct Wallet;

#[note]
struct TargetedNote {
    target_account_id: AccountId,
}

#[note]
impl TargetedNote {
    #[note_script]
    fn run(self, _arg: Word, account: &mut Wallet) {
        // `self.target_account_id` is deserialized from the note inputs;
        // forward work to component methods on `account`.
    }
}
```

(`use` statements and crate attributes elided; see [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) for a complete file.) For a verified working example with a typed field and an `&mut` account-wrapper parameter, see [compiler/examples/p2id-note/src/lib.rs](https://github.com/0xMiden/compiler/blob/main/examples/p2id-note/src/lib.rs) (`#[note] struct P2idNote { target_account_id: AccountId }`, where the script asserts `account.get_id() == self.target_account_id` and calls `account.receive_asset(asset)` for each attached asset).

**Component side that absorbs the call**: the counter's `CounterContract` component exposes `get_count` / `increment_count` (see [counter-account/src/lib.rs](../../../contracts/counter-account/src/lib.rs)); the note calls those through the wrapper. A component method validates (felt-arithmetic safety, see `rust-sdk-pitfalls` P1), updates storage, and — for a withdraw-style flow — creates a P2ID output note via the P2ID pattern above.

**Test wiring**: tests pass the serialized Felt representation of the note struct's fields via `NoteBuilder::note_storage([...])`, in declaration order. See `rust-sdk-testing-patterns` skill, "Note Construction" section, for building a note from a compiled `.masp` package with `NoteScript::from_package` + `NoteBuilder`.

## Asset Receiving via Component Methods

Note scripts cannot call `native_account::add_asset()` directly (see pitfall P11). The canonical pattern is for an account component to expose a public (trait) method that wraps `native_account::add_asset()`, and the note script calls that method through the `#[account(...)]` wrapper.

Component side: a trait method (e.g. `deposit`) validates the deposit, updates storage, and calls `native_account::add_asset()`. Note side: the note declares `#[account(package::Interface)] pub struct Wallet;` and, inside `#[note_script] fn run(self, _arg: Word, account: &mut Wallet)`, calls `account.deposit(...)` on that wrapper. It is **not** a free `package::deposit()` call — the call goes through the injected `account`, exactly as the counter's [increment-note/src/lib.rs](../../../contracts/increment-note/src/lib.rs) calls `account.increment_count()`.

## Validation Checklist

- [ ] `#![no_std]` and `#![feature(alloc_error_handler)]` at top of every contract
- [ ] Account components use the three-part pattern: `#[component_storage]` struct + `#[component]` trait + `#[component]` impl (never `#[component]` on a struct)
- [ ] `crate-type = ["cdylib"]` in `Cargo.toml`
- [ ] Correct `[lib] kind` in `miden-project.toml` (`account-component` / `note` / `tx-script`) with the matching `namespace`
- [ ] Typed storage uses `StorageValue<T>` / `StorageMap<K, V>` with `get()` / `set()`; slot names derive from `<package>::<namespace-interface>::<field>`
- [ ] Notes/tx-scripts that call a component declare an `#[account(package::Interface)]` wrapper and call methods on the injected `account`
- [ ] Cross-component deps declared in `miden-project.toml` under both `[dependencies]` (path) and `[package.metadata.miden.dependencies]` (wit)
- [ ] Felt arithmetic validated before subtraction (see rust-sdk-pitfalls skill)
- [ ] Felt comparisons use `.as_canonical_u64()` (see rust-sdk-pitfalls skill)
