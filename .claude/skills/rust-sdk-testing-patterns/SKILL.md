---
name: rust-sdk-testing-patterns
description: Guide to testing Miden smart contracts with MockChain (Miden v0.15). Covers test setup, contract building, account/note creation, transaction execution, storage verification, faucet setup, output note verification, block numbering, multi-transaction tests, and asset-bearing notes. Use when writing, editing, or debugging Miden integration tests.
---

# Miden Testing Patterns (MockChain)

These patterns target Miden **v0.15** (`miden-client`/`miden-standards`/`miden-testing` 0.15.x).

The **authoritative working example** in this project is the counter contract: [counter_test.rs](../../../integration/tests/counter_test.rs) is a complete test covering imports, MockChain setup, contract building, account creation with storage, note creation, transaction execution, and storage verification. Mirror it for the patterns below.

## Test File Setup

Tests go in `integration/tests/`. All tests are async and use MockChain for local execution without a network.

The v0.15 imports [counter_test.rs](../../../integration/tests/counter_test.rs) relies on are:

```rust
use std::{path::Path, sync::Arc};

use integration::helpers::{build_project_in_dir, counter_storage_slot, COUNTER_STORAGE_KEY};
use miden_client::{
    account::{component::InitStorageData, AccountBuilder, AccountComponent, AccountType},
    auth::AuthSchemeId,
    crypto::RandomCoin,
    note::NoteScript,
    transaction::RawOutputNote,
    Word,
};
use miden_standards::testing::note::NoteBuilder;
use miden_testing::{AccountState, Auth, MockChain};
```

## Step-by-Step Test Pattern

### 1. Initialize MockChain Builder

Start from `let mut builder = MockChain::builder();` (see [counter_test.rs](../../../integration/tests/counter_test.rs)).

### 2. Create Sender/Wallet Accounts

For a bare wallet use `builder.add_existing_wallet(Auth::BasicAuth { auth_scheme: AuthSchemeId::Falcon512Poseidon2 })` (see [counter_test.rs](../../../integration/tests/counter_test.rs)). For wallets with pre-funded assets, use `builder.add_existing_wallet_with_assets(Auth::BasicAuth { auth_scheme: AuthSchemeId::Falcon512Poseidon2 }, [FungibleAsset::new(faucet.id(), 100)?.into()])`.

> Auth-scheme naming: `miden_client::auth` re-exports the same protocol enum under two names — `AuthScheme` (the protocol name) and `AuthSchemeId` (an alias). Both compile; the field is `Auth::BasicAuth { auth_scheme }`, and the variant `Falcon512Poseidon2` is the same on both. This project uses `AuthSchemeId::Falcon512Poseidon2`.

### 3. Set Up Faucets (for fungible assets)
```rust
let faucet = builder.add_existing_basic_faucet(
    Auth::BasicAuth {
        auth_scheme: AuthSchemeId::Falcon512Poseidon2,
    },
    "TOKEN",     // token symbol
    1000,        // max supply
    Some(10),    // token_supply (None defaults to 0)
)?;
```

The 4th argument is `token_supply: Option<u64>` (an explicit `None` is treated as `0`).

### 4. Build Contracts

Build each project from its directory with the `build_project_in_dir` helper, e.g. `let contract_package = Arc::new(build_project_in_dir(Path::new("../contracts/counter-account"), true)?);` (see [counter_test.rs](../../../integration/tests/counter_test.rs) and [integration/src/helpers.rs](../../../integration/src/helpers.rs) `build_project_in_dir`).

### 5. Create Account with Storage

**Storage slot naming convention** (CRITICAL):
```
<package_name>::<interface_segment>::<field_name>
```

The slot name is part of the on-chain storage ABI and is derived by the compiler's `#[component_storage]` macro, **not** from the Rust struct name:
- `<package_name>` is the **bare** package name (`[package].name`), with no `miden:` org prefix.
- `<interface_segment>` is the `[lib].namespace` **interface** segment — the text between the last `/` and the `@` in the namespace — snake_cased. Because it comes from the declared namespace, renaming the Rust struct cannot change the deployed slot name.
- `<field_name>` is the Rust storage field's identifier (not its `description`).

Characters outside `[A-Za-z0-9_]` are replaced with `_` in each segment.

Example: package `counter-account` with `[lib].namespace = "miden:counter-account/counter-contract@0.1.0"` (see [contracts/counter-account/miden-project.toml](../../../contracts/counter-account/miden-project.toml)) and storage struct `CounterContractStorage` (field `count_map`) yields the slot:
- `counter_account::counter_contract::count_map`

Note the middle segment is `counter_contract` (the interface segment from the namespace), **not** `counter_contract_storage` (the struct) and **not** `counter_account`, and there is no `miden_` org prefix. This is exactly the string [integration/src/helpers.rs](../../../integration/src/helpers.rs) passes to `StorageSlotName::new(...)` in `counter_storage_slot()`.

The component's storage is declared with the v0.15 three-part component macro (`#[component_storage]` struct + `#[component]` trait + `#[component]` impl); the storage struct, not the trait, carries the `#[storage]` fields the slot names derive from. See the `rust-sdk-patterns` skill for the contract side.

**Authoritative pattern** (from [counter_test.rs](../../../integration/tests/counter_test.rs)): build the `StorageSlotName`, seed the component's initial storage into `InitStorageData`, build the `AccountComponent` from the compiled package, then register the account with `builder.add_account_from_builder(...)`:

```rust
let counter_storage_slot = counter_storage_slot()?;
let mut init_storage_data = InitStorageData::default();
// The counter's `count_map` is a `StorageMap<Word, Felt>`; seed its fixed key with 0
// so the increment note finds an existing entry. `insert_map_entry(slot_name, key, value)`
// takes three args: `slot_name: impl TryInto<StorageSlotName>`, `key`, `value`.
init_storage_data.insert_map_entry(counter_storage_slot.clone(), COUNTER_STORAGE_KEY, 0_u64)?;

let counter_component = AccountComponent::from_package(&contract_package, &init_storage_data)?;
let counter_account = builder.add_account_from_builder(
    Auth::BasicAuth {
        auth_scheme: AuthSchemeId::Falcon512Poseidon2,
    },
    AccountBuilder::new([3_u8; 32])
        .account_type(AccountType::Public)
        .with_component(counter_component),
    AccountState::Exists,
)?;
```

> Account model:
> - `AccountType` is the visibility enum `{ Private, Public }`.
> - Set account visibility via `.account_type(AccountType::Public | ::Private)` — there is no separate `.storage_mode(...)` / `AccountStorageMode` on the builder.
> - Faucet-ness is determined by the installed components.

For a **single-value** contract slot (a `StorageValue<Word>` field on-chain) instead of a map, seed it with `insert_value` — a value slot that has no schema default otherwise makes `AccountComponent::from_package` error with `InitValueNotProvided`:

```rust
let value_slot = StorageSlotName::new("my_account::my_component::initialized")?;
let mut init_storage_data = InitStorageData::default();
init_storage_data.insert_value(
    StorageValueName::from_slot_name(&value_slot),
    Word::default(), // zero Word (an uninitialized flag), NOT a bare integer
)?;
```

> Storage-seeding footgun: `InitStorageData::insert_value(name, value)` takes `value: impl Into<WordValue>`. The numeric `From` impls (`u8`/`u16`/`u32`/`u64`) produce a `WordValue::Atomic(string)` that the slot's schema parses — **not** a felt-positioned `Word`. Only `From<Felt>` yields `[felt, 0, 0, 0]`, and `From<Word>`/`From<[Felt; 4]>`/`From<[u32; 4]>` are fully-typed words. For a `StorageValue<Word>` slot whose contract reads index `[0]`, seed a `Word` (`Word::default()` for zero). A map slot (like the counter's `count_map`) is seeded per-entry with `insert_map_entry(...)` instead.

### 6. Create Notes

Build notes with `NoteBuilder`, seeding the `RandomCoin` from the note-script root (see [counter_test.rs](../../../integration/tests/counter_test.rs)):

```rust
let mut note_rng = RandomCoin::new(Word::from(
    NoteScript::from_package(note_package.as_ref())?.root(),
));
let counter_note = NoteBuilder::new(sender.id(), &mut note_rng)
    .package((*note_package).clone())
    .build()?;
```

For a note that also carries assets and inputs, configure the extra builder steps:

```rust
use miden_client::{asset::FungibleAsset, crypto::RandomCoin, note::NoteScript, Felt, Word};
use miden_standards::testing::note::NoteBuilder;

let note_script = NoteScript::from_package(note_package.as_ref())?;
let mut note_rng = RandomCoin::new(Word::from(note_script.root()));
let note = NoteBuilder::new(sender.id(), &mut note_rng)
    .package((*note_package).clone())
    .add_assets([FungibleAsset::new(faucet.id(), 50)?.into()])
    .note_storage([Felt::from(42_u32), Felt::from(0_u32)])?
    .build()?;
```

> `NoteScript::root()` returns a `NoteScriptRoot` newtype. `RandomCoin::new` needs a `Word`, so convert the root explicitly with `Word::from(...root())` (equivalently `...root().into()` or `...root().as_word()`).

> `Felt::new(u64)` is **fallible** — it returns `Result<Felt, FeltFromIntError>`. `note_storage` takes `impl IntoIterator<Item = Felt>`, so build each felt with the infallible `Felt::from(42_u32)` for in-range literals (`From<u8>/From<u16>/From<u32>` are infallible); for a `u64` use `Felt::new(n)?` or `Felt::new_unchecked(n)`.

### 7. Add to MockChain and Build

Register accounts (`add_account_from_builder(...)` already registered the counter account in Step 5) and seed notes with `builder.add_output_note(RawOutputNote::Full(counter_note.clone()))`, then `let mut mock_chain = builder.build()?;` (see [counter_test.rs](../../../integration/tests/counter_test.rs)).

### 8. Execute Transaction

The full execution flow is `build_tx_context` -> `execute()` -> `add_pending_executed_transaction()` -> `prove_next_block()` (see [counter_test.rs](../../../integration/tests/counter_test.rs)). The single-transaction counter test does not call `apply_delta()` because `counter_account` is not reused after the build; final state is read from `mock_chain.committed_account(...)` after the block is proven. Multi-transaction tests that keep using the in-memory `Account` variable across steps should call `account.apply_delta(&executed.account_delta())?` after each `execute()` (see "Multi-Transaction Test Pattern" below).

### 9. Execute with Transaction Script

A compiler project with `kind = "tx-script"` compiles to a `TransactionScript`-kind package, **not** an `Executable`. Because of that, `TransactionScript::from_package` and `Package::unwrap_program` do **not** apply to it: `from_package` calls `package.try_into_program()`, which returns `Err` for a non-executable package, and `unwrap_program` asserts the kind is `Executable` and **panics**. Build the script from the package's MAST forest plus its entry export instead:

```rust
use miden_client::transaction::TransactionScript;

let tx_script_package = Arc::new(build_project_in_dir(
    Path::new("../contracts/my-tx-script"),
    true,
)?);

// Locate the entry export ("main"/"run", or the sole export) and build from parts, e.g. a
// small helper that finds the entry procedure root in the MAST forest and calls
// `TransactionScript::from_parts(package.mast.mast_forest().clone(), entrypoint)`.
let tx_script = build_tx_script_from_package(tx_script_package.as_ref())?;

let executed = mock_chain
    .build_tx_context(account.id(), &[], &[])?
    .tx_script(tx_script)
    .build()?
    .execute()
    .await?;

mock_chain.add_pending_executed_transaction(&executed)?;
mock_chain.prove_next_block()?;

let updated_account = mock_chain.committed_account(account.id())?;
```

> Reserve `TransactionScript::from_package(&package)?` (and the `#[doc(hidden)]` `unwrap_program()`) for packages that are genuinely `Executable`. For `kind = "tx-script"` compiler packages, use `from_parts` / a `build_tx_script_from_package`-style helper as above — `from_package` returns an error and `unwrap_program()` panics on them.

### 10. Verify Storage State

Read state with `account.storage().get_item(&slot)` / `.get_map_item(&slot, key)` on an in-memory `Account` you keep `apply_delta`-current, or re-fetch the committed account with `mock_chain.committed_account(account.id())?` after `prove_next_block()` and assert on its storage. Map values come back as scalar words in `[value, 0, 0, 0]` layout, so read index `[0]` (see [counter_test.rs](../../../integration/tests/counter_test.rs)):

```rust
let count = mock_chain
    .committed_account(counter_account.id())?
    .storage()
    .get_map_item(&counter_storage_slot, COUNTER_STORAGE_KEY)
    .expect("Failed to get counter value from storage slot");
assert_eq!(count[0].as_canonical_u64(), 1);
```

### 11. Verify Output Notes

**Important**: `add_output_note()` is only available on `MockChainBuilder` (before `build()`) — use it to seed the chain with existing notes. To verify output notes from a transaction, use `extend_expected_output_notes()` on `TxContextBuilder`:

```rust
use miden_client::{
    note::{Note, NoteType, PartialNoteMetadata},
    transaction::RawOutputNote,
};

// Note::new takes a PartialNoteMetadata (sender + note_type + tag).
// Build it with PartialNoteMetadata::new(sender, note_type),
// then optionally `.with_tag(tag)` (the tag defaults to NoteTag::default()).
let partial_metadata = PartialNoteMetadata::new(sender, NoteType::Public).with_tag(tag);
let expected_note = Note::new(expected_assets, partial_metadata, expected_recipient);

let tx_context = mock_chain
    .build_tx_context(account.id(), &[note.id()], &[])?
    .extend_expected_output_notes(vec![RawOutputNote::Full(expected_note)])
    .build()?;

// execute() will verify output notes match
let executed = tx_context.execute().await?;
```

> Note metadata:
> - `Note::new(assets, partial_metadata, recipient)` takes a `PartialNoteMetadata` (sender/type/tag only); there is no `Into` conversion on the parameter.
> - For attachment-bearing notes use `Note::with_attachments(assets, partial_metadata, recipient, attachments)` (attachments are `NoteAttachments`).

## MockChain Note Interaction

Notes flow through MockChain in four steps:

1. **Build** the note from a compiled `.masp` package via `NoteBuilder` (see "Note Construction" below).
2. **Seed** with `MockChainBuilder::add_output_note(RawOutputNote::Full(note.clone()))` BEFORE `builder.build()`. This places the note on the chain so a later transaction can consume it. `add_output_note(...)` is only available on the builder; once `builder.build()` returns the `MockChain`, output notes can only appear as the result of executing a transaction. `RawOutputNote` is re-exported from `miden_client::transaction`.
3. **Consume** by passing the note ID to `mock_chain.build_tx_context(account, &[note.id()], &[])`. The transaction's note-script execution reads the consumed note's storage and assets.
4. **Verify** expected output notes with `.extend_expected_output_notes(vec![RawOutputNote::Full(expected.clone())])` on the `TxContextBuilder`. `tx_context.execute().await?` will assert the produced output notes match.

After `execute()` and before `add_pending_executed_transaction(...) + prove_next_block()`: if a later step will keep using the in-memory `Account` variable (for example, to build another `tx_context` or assert account state directly), call `account.apply_delta(&executed.account_delta())?` to keep the variable in sync with the chain. Post-block reads should use `mock_chain.committed_account(account.id())?` (see Step 8 above and "Multi-Transaction Test Pattern" below). For block advancement and reference-block semantics, see "MockChain Block Numbering" below.

## Multi-Transaction Test Pattern

For contracts requiring initialization before use, each step usually needs its own `execute()` → `add_pending_executed_transaction()` → `prove_next_block()` cycle. Fetch the committed account or note state from `mock_chain` between steps before building the next context.

`apply_delta()` is needed whenever you keep reading from / reusing the **same in-memory `Account`** across transactions — whether they land in the same block or in separate blocks. Call `account.apply_delta(&executed.account_delta())?` after each `execute()` (each followed by `add_pending_executed_transaction` + `prove_next_block`) so later local reads like `account.storage().get_map_item(...)` see the latest state. If you instead re-fetch via `mock_chain.committed_account(...)` after `prove_next_block()`, you can skip `apply_delta()` — that is the single-transaction case shown in [counter_test.rs](../../../integration/tests/counter_test.rs), which reads final state only after the last `prove_next_block()` and never reuses the in-memory variable.

## MockChain Block Numbering

Genesis is block 0. Each `prove_next_block()` advances the block number by 1. In contract code, `tx::get_block_number()` returns the **reference block** — the last proven block at the time the transaction started, not the block the transaction will be included in.

## Note Construction

Prefer `NoteBuilder` for creating notes in tests. Start from `NoteBuilder::new(sender.id(), &mut note_rng)`, then configure `.package(...)`, optional `.note_type(...)`, optional `.tag(...)`, optional `.add_assets(...)`, optional `.note_storage(...)?`, optional `.serial_number(...)`, and finally `.build()?`. Seed the `RandomCoin` from `Word::from(NoteScript::from_package(note_package.as_ref())?.root())` (see Step 6 and [counter_test.rs](../../../integration/tests/counter_test.rs)).

The serial number is what makes a note unique, and the RNG source differs between the deterministic test path and the real-client path:

- **Deterministic test path**: seed the `RandomCoin` from the note-script root and omit `.serial_number(...)`, letting `NoteBuilder` derive the serial deterministically from `RandomCoin::new(Word::from(note_script.root()))`. Used when seeding `MockChainBuilder` with a freshly-built note. See [counter_test.rs](../../../integration/tests/counter_test.rs).
- **Real-client path**: pass the client's RNG directly (`NoteBuilder::new(sender.id(), client.rng())`) so each note gets a fresh serial, then publish it with a real `TransactionRequestBuilder`. See [integration/src/bin/increment_count.rs](../../../integration/src/bin/increment_count.rs), which builds the note with `client.rng()` + `.tag(0)`, publishes it via `TransactionRequestBuilder::new().own_output_notes(vec![note.clone()]).build()?`, and consumes it via `.input_notes([(note.clone(), None)]).build()?`. For the surrounding client setup (CLI side), see the `miden-client-cli` skill.

To drive a **cross-component note** (see the `rust-sdk-patterns` "Cross-Component Note Pattern"), populate the note's `note_storage(...)` with the serialized Felt representation of the typed `#[note]` struct's fields in declaration order; the `#[note]` macro deserializes that slice into `self` before the script runs. The increment note carries no such storage — its `#[note_script] fn run(self, _arg: Word, account: &mut Wallet)` simply calls `account.get_count()` / `account.increment_count()`.

## Asset-Bearing Note Example

To create a note that carries fungible assets in tests:

1. Create a `FungibleAsset` from a faucet ID and amount, e.g. `FungibleAsset::new(faucet.id(), 50)?`, and wrap into `NoteAssets::new(vec![Asset::Fungible(asset)])?` (or pass via `NoteBuilder::add_assets`).
2. Seed a `RandomCoin` from `Word::from(NoteScript::from_package(note_package.as_ref())?.root())` (the conversion turns the `NoteScriptRoot` into the `Word` that `RandomCoin::new` expects).
3. Pass the asset into `NoteBuilder::add_assets(...)` and any note inputs into `note_storage(...)?`. `note_storage` wants `Item = Felt`; build each input with the infallible `Felt::from(_u32)` for in-range literals (not `Felt::new(u64)`, which is fallible), or `Felt::new_unchecked(n)` for u64 inputs (see Step 6).
4. Finish with `.package((*note_package).clone()).build()?`.

The faucet must be set up first (see Step 3) and the sender wallet must hold sufficient assets (see Step 2).

## Key Dependencies

See [integration/Cargo.toml](../../../integration/Cargo.toml) for the exact versions. The integration crate depends on `cargo-miden = "0.9"` (its `build_project_in_dir` helper calls `cargo_miden::run`) alongside the 0.15 line — `miden-client`, `miden-standards`, `miden-testing`, and `miden-client-sqlite-store` at `0.15`, plus `miden-mast-package = "0.23"` — with no git-rev/branch pins. The contracts it builds depend on the guest SDK `miden = "0.13"` and compile with the released compiler v0.9.0.

## Validation Checklist

- [ ] Test function is `async` and uses `#[tokio::test]`
- [ ] Auth uses `AuthSchemeId::Falcon512Poseidon2` (or the equivalent `AuthScheme::Falcon512Poseidon2` — both name the same protocol enum)
- [ ] `AccountBuilder` uses `.account_type(AccountType::Public | ::Private)` and no `.storage_mode(...)` / no `AccountStorageMode`
- [ ] Storage slot names follow `<package_name>::<interface_segment>::<field_name>` (bare package name, `[lib].namespace` interface segment, e.g. `counter_account::counter_contract::count_map`)
- [ ] Map slots seeded per-entry via `InitStorageData::insert_map_entry(slot, key, value)`; value slots without a schema default seeded via `InitStorageData::insert_value(StorageValueName::from_slot_name(&slot), ..)` with a `Word` (e.g. `Word::default()`), not a bare integer (numeric `Into<WordValue>` yields an atomic string, not a felt-positioned word)
- [ ] All contracts built before account/note creation
- [ ] `NoteScript::root()` converted with `Word::from(...)` before seeding `RandomCoin`
- [ ] Note-storage felts built with infallible `Felt::from(_u32)` or `Felt::new_unchecked(_u64)` (`Felt::new(u64)` returns `Result`, so a bare `[Felt::new(..)]` array does not satisfy `Item = Felt`)
- [ ] `Note::new(...)` is passed a `PartialNoteMetadata` (not `NoteMetadata`)
- [ ] `kind = "tx-script"` packages built with `from_parts` / a `build_tx_script_from_package`-style helper (not `from_package`/`unwrap_program`, which error/panic on them)
- [ ] `prove_next_block()` called after `add_pending_executed_transaction()`
- [ ] Post-block assertions read state from `mock_chain.committed_account(...)` (or `account.apply_delta(...)` is called when reusing an in-memory `Account` across transactions)
- [ ] Notes added to `MockChainBuilder` via `add_output_note(RawOutputNote::Full(...))` before `build()`
- [ ] Faucet set up before creating assets
