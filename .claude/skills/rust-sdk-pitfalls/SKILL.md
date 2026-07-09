---
name: rust-sdk-pitfalls
description: Critical pitfalls and safety rules for Miden Rust SDK development. Covers felt arithmetic security, comparison operators, argument limits, storage naming, no-std setup, asset layout, P2ID roots, NoteType construction, note-to-component call boundaries, and note input immutability. Use when reviewing, debugging, or writing Miden contract code.
---

# Miden SDK Pitfalls

## P1: Felt Arithmetic is Modular (SECURITY CRITICAL)

**Severity**: Critical — can cause loss of funds

Felt subtraction wraps around the prime field modulus (p = 2^64 - 2^32 + 1) instead of panicking. Subtracting more than available silently produces a huge positive number.

```rust
// DANGEROUS — no check before subtraction
let new_balance = current_balance - withdraw_amount;
// If withdraw_amount > current_balance, new_balance ≈ 2^64 (wraps!)

// SAFE — always validate first
assert!(
    current_balance.as_canonical_u64() >= withdraw_amount.as_canonical_u64(),
    "Insufficient balance"
);
let new_balance = current_balance - withdraw_amount;
```

**Rule**: ALWAYS check `.as_canonical_u64()` values before any Felt subtraction.

**Max Felt value**: The maximum valid Felt is `p - 1 = 18446744069414584320`, not `u64::MAX` (`18446744073709551615`). Using `u64::MAX` as a sentinel or boundary value causes silent wraparound.

## P2: Felt Comparison Operators Are Misleading for Quantity Logic

**Severity**: High — silently produces incorrect results

`<`, `>`, `<=`, `>=` on Felt values compare field elements, which differs from natural number ordering. In protocol-level code working with field elements, these comparisons may be intentional. For business logic (balances, amounts, counts), the results are misleading.

```rust
// MISLEADING for business logic — compares field elements
if balance > threshold { ... }

// CORRECT for business logic — compare as integers
if balance.as_canonical_u64() > threshold.as_canonical_u64() { ... }
```

**Rule**: For quantity/business logic, ALWAYS convert to `.as_canonical_u64()` before using comparison operators.

## P3: Direct Call Boundary Passes At Most 16 Stack Felts (4 Words)

**Severity**: High — exceeding the 16-felt call boundary is a compile error

A direct cross-context / export / FPI call passes its parameters on the MASM operand stack, whose addressable window is 16 felts (4 Words, counting the canonical-ABI result pointer when present). Passing more than 16 flat felts across that boundary is a **compilation error**: after expanding 64-bit values and any result pointer, the flattened parameters must fit in 16 operand-stack felts. (Indirection for larger payloads via the advice provider is planned but not yet implemented, so today the limit is hard.)

```rust
// COMPILE ERROR — flattens past 16 felts
fn process(a: Word, b: Word, c: Word, d: Word, e: Word) { ... }

// OK — keep signatures small, or pass aggregates by reference so each lowers to a pointer
fn process(a: &Word, b: &Word, c: &Word, d: &Word, e: &Word) { ... }
```

## P4: Storage API Is Typed

**Severity**: Medium — the wrong component shape does not compile

Account storage uses typed slots:

- `StorageValue<T>` for a single typed slot
- `StorageMap<K, V>` for typed maps
- `get()` / `set()` methods
- `K: WordKey`, `T: WordValue`, `V: WordValue`

An account component is written in **three parts**: annotate the storage struct with `#[component_storage]`, the API `trait` with `#[component]`, and the `impl Trait for Storage` block with `#[component]`. See the working example in `contracts/counter-account/src/lib.rs`:

```rust
// 1. Storage struct — annotated #[component_storage], NOT #[component].
//    Applying #[component] to a struct is a hard compile error.
#[component_storage]
struct CounterContractStorage {
    #[storage(description = "counter contract storage map")]
    count_map: StorageMap<Word, Felt>,
}

// 2. API trait — defines the exported interface.
#[component]
trait CounterContract {
    fn get_count(&self) -> Felt;
    fn increment_count(&mut self) -> Felt;
}

// 3. Implementation — the behavior, wired to the storage struct.
#[component]
impl CounterContract for CounterContractStorage {
    fn get_count(&self) -> Felt {
        let key = Word::new([felt!(0), felt!(0), felt!(0), felt!(1)]);
        self.count_map.get(key)
    }
    fn increment_count(&mut self) -> Felt {
        let key = Word::new([felt!(0), felt!(0), felt!(0), felt!(1)]);
        let new_value = self.count_map.get(key) + felt!(1);
        self.count_map.set(key, new_value);
        new_value
    }
}
```

If you need custom keys or values, implement `WordKey` / `WordValue` by converting to and from a single `Word`.

## P5: Storage Slot Naming Convention

**Severity**: Medium — causes silent default-value reads in tests

Storage slot names follow a strict pattern. Getting it wrong often returns the default value silently.

**Pattern**: `[package_name]::[namespace_interface_segment]::[field_name]`

**Where the segments come from**: The `#[component_storage]` macro (NOT `#[component]`) processes the `#[storage]` fields and derives slot names. It loads `miden-project.toml` (next to your `Cargo.toml`, NOT `Cargo.toml` itself):

- **First segment** = `[package] name`.
- **Middle segment** = the *interface segment* of the `[lib] namespace` value. The namespace is a fully-qualified component id `namespace:package/interface@version`; the interface segment sits between the last `/` and the `@`. This is deliberately decoupled from the Rust storage-struct name, so renaming the private struct cannot change deployed slot names. The struct name (`CounterContractStorage`, …) does NOT appear in the slot name.
- **Last segment** = the `#[storage]` field name.

**Conversion rule**: Each segment is sanitized — any `@version` suffix is stripped, the interface segment is passed through `snake_case`, and characters outside `[A-Za-z0-9_]` are replaced with `_` (an empty or leading-`_` segment is prefixed with `x`). Project package names are conventionally kebab-case (e.g. `counter-account`), so the first segment is that name with hyphens replaced by `_` — it does NOT equal the package name verbatim (`counter-account` → `counter_account`).

| `[package] name` | `[lib] namespace` | Field | Storage Slot Name |
|------------------|-------------------|-------|-------------------|
| `counter-account` | `miden:counter-account/counter-contract@0.1.0` | `count_map` | `counter_account::counter_contract::count_map` |

The integration code depends on this exact name. In `integration/src/helpers.rs`, `counter_storage_slot()` builds it via `StorageSlotName::new("counter_account::counter_contract::count_map")`; a mismatch there reads the default value instead of the seeded one.

**Caveat (toolchain-version dependent)**: This naming is a property of the Rust SDK contract macros, which live in the `miden-base-macros` crate (0.13.0, part of the Rust SDK family alongside `miden` and `miden-base-sys`, all 0.13.0; the separate compiler / `cargo-miden` workspace is versioned 0.9.0). Do not conflate these with the protocol/network version (v0.15). The slot-naming algorithm — `package_name::snake_case(interface_segment)::field`, with non-`[A-Za-z0-9_]` mapped to `_` and `@version` stripped — is stable, but verify against your installed toolchain rather than assuming a protocol version.

## P6: No-std Environment

**Severity**: Medium -- causes compilation errors

All contract code must be `#![no_std]`. Forgetting this or using std types causes build failures.

**Required at the top of every contract file:** See any contract in [contracts/](../../../contracts/) for the correct pattern (`#![no_std]` + `#![feature(alloc_error_handler)]`).

**For heap allocation (Vec, String, Box):**
```rust
extern crate alloc;
use alloc::vec::Vec;
```

## P7: Rust SDK `Asset` Is Two Words (Key + Value)

**Severity**: Medium — reconstructing an asset from raw `asset.inner[...]` offsets is wrong

In the Rust SDK (`miden::Asset` / `miden_base_sys::bindings::Asset`), an `Asset` is encoded as two words:

```rust
pub struct Asset {
    pub key: Word,
    pub value: Word,
}
```

```rust
// Reading the amount from a fungible asset
let amount = asset.value[0];

// Persisting or comparing the asset class
let asset_key = asset.key;
```

Use `asset.key` and `asset.value` (or protocol helpers) rather than reconstructing an asset from raw `asset.inner[...]` offsets.

**SDK vs protocol `Asset`**: the two-word `{key, value}` form is the Rust SDK ABI type. At the protocol layer, `Asset` is an enum `{ Fungible, NonFungible }`, and the vault words are obtained via `to_key_word()` / `to_value_word()`. Reading the fungible amount from `value[0]` is correct on both sides.

## P8: Build Recipients with `note::build_recipient`

**Severity**: Medium — calling a nonexistent `Recipient::compute` fails to compile

Build recipients through the note binding:

```rust
extern crate alloc;
use alloc::vec;

let recipient = note::build_recipient(
    serial_num,
    script_root,
    vec![recipient_id.suffix, recipient_id.prefix],
);
```

`note::build_recipient` is the Rust SDK alias for the host function `miden::protocol::note::compute_and_store_recipient`, which computes and stores the recipient in one step. You can call either name.

## P9: P2ID Note Root — Prefer `script_root()`, Do Not Hardcode

**Severity**: Low-Medium — breaks after miden-standards updates

Creating P2ID output notes requires the MAST root of the P2ID script. The root changes whenever the P2ID script or the assembler/hashing changes, so a hardcoded literal is fragile and unverifiable.

**Source of truth**: Use `P2idNote::script_root()` from `miden-standards` (returns a `NoteScriptRoot`, a `Word` newtype convertible via `.into()`). Derive the root from the dependency rather than embedding a literal, and re-derive after any dependency bump.

```rust
use miden_standards::note::P2idNote;

// script_root() returns a NoteScriptRoot (a Word newtype); convert to Word when needed.
let p2id_root: Word = P2idNote::script_root().into();
```

**If you must embed a constant** (e.g., inside compiler/contract code that cannot call into miden-standards), regenerate it from the current `miden-standards` version and verify it after every update. The four-limb literal below is ILLUSTRATIVE only — it will not match your build and must not be copied as-is:

```rust
// ILLUSTRATIVE ONLY — will not match your build. Regenerate from
// P2idNote::script_root() for your pinned miden-standards version.
fn p2id_note_root() -> Word {
    Word::try_from([
        13362761878458161062_u64,
        15090726097241769395_u64,
        444910447169617901_u64,
        3558201871398422326_u64,
    ])
    .unwrap()
}
```

**Risk**: If miden-standards updates the P2ID script, any hardcoded digest becomes invalid and withdrawals silently fail.

**NoteType for P2ID**: P2ID output notes created in contract code are constructed with `NoteType::from(felt!(...))` — `felt!(0)` for private, `felt!(1)` for public (see P10). In v0.15 the kernel rejects any note type other than `0` (private) or `1` (public) with `ERR_NOTE_INVALID_TYPE`. A common working pattern reads the note type from an input note's storage and forwards it through `NoteType::from(note_type)`.

## P10: NoteType Variants Unavailable in Compiler SDK

**Severity**: Critical -- wrong values panic at runtime, named variants cause compilation errors

Named enum variants (`NoteType::Private`, `NoteType::Public`) don't exist in contract code — the SDK `NoteType` is an unvalidated transparent `Felt` wrapper. Construct via `NoteType::from()`:

| NoteType | Value |
|----------|-------|
| Private (default) | `NoteType::from(felt!(0))` |
| Public | `NoteType::from(felt!(1))` |

**Note-type encoding**: the note type is 1-bit — `Private = 0` (the protocol default) and `Public = 1`. Only these two values exist; there is no `Encrypted` type. The SDK wrapper does no validation, so an out-of-range value (e.g. `felt!(2)` or `felt!(3)`) is not caught at compile time — the kernel rejects it at execution time with `ERR_NOTE_INVALID_TYPE` (it asserts `note_type <= 1`).

When a note forwards a caller-supplied note type, read it from the note's storage and pass it straight into `NoteType::from(note_type)`.

## P11: Note Scripts Cannot Call Native Account Functions

**Severity**: High -- causes runtime failures

Note scripts cannot call `native_account::add_asset()` or other `native_account::` functions directly. The kernel's `authenticate_account_origin` check rejects these calls from a note context. Instead, note scripts must call an account component method (through the `#[account(...)]` wrapper), which then performs the privileged `native_account::` operation internally.

See `contracts/increment-note/src/lib.rs` for the wrapper pattern: the note declares its consuming account via `#[account(counter_account::CounterContract)] pub struct Wallet;` and, inside `#[note_script] fn run(self, _arg: Word, account: &mut Wallet)`, calls the component methods on that wrapper (`account.get_count()`, `account.increment_count()`) rather than any `native_account::` function directly. Any asset mutation (e.g. `native_account::add_asset()`) must likewise live inside a component method that the note calls through the wrapper, never in the note script itself.

## P12: Note Inputs Are Immutable After Creation

**Severity**: Low -- causes incorrect architecture

Note inputs (the Felt data the `#[note]` macro deserializes into `self`, read at runtime via `active_note::get_storage()`) are baked at note creation time and cannot be modified after creation. Design the typed note struct's field set and field order carefully before deployment; any later change is a breaking change for existing notes.
