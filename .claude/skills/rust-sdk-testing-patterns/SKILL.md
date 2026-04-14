---
name: rust-sdk-testing-patterns
description: Guide to testing Miden smart contracts with MockChain. Covers test setup, contract building, account/note creation, transaction execution, storage verification, faucet setup, output note verification, block numbering, multi-transaction tests, and asset-bearing notes. Use when writing, editing, or debugging Miden integration tests.
---

# Miden Testing Patterns (MockChain)

## Test File Setup

Tests go in `integration/tests/`. All tests are async and use MockChain for local execution without a network.

See [counter_test.rs](../../../integration/tests/counter_test.rs) for a complete working test covering imports, MockChain setup, contract building, account creation with storage, note creation, transaction execution, and storage verification.

## Step-by-Step Test Pattern

### 1. Initialize MockChain Builder

See [counter_test.rs](../../../integration/tests/counter_test.rs) line 17 for the pattern: `let mut builder = MockChain::builder();`

### 2. Create Sender/Wallet Accounts

See [counter_test.rs](../../../integration/tests/counter_test.rs) line 20 for the basic wallet pattern. For wallets with pre-funded assets, use `builder.add_existing_wallet_with_assets(Auth::BasicAuth, [FungibleAsset::new(faucet.id(), 100)?.into()])`.

### 3. Set Up Faucets (for fungible assets)
```rust
let faucet = builder.add_existing_basic_faucet(
    Auth::BasicAuth,
    "TOKEN",     // token symbol
    1000,        // max supply
    Some(10),    // total_issuance (None for 0)
)?;
```

### 4. Build Contracts

See [counter_test.rs](../../../integration/tests/counter_test.rs) lines 23-30 for the pattern using `build_project_in_dir`.

### 5. Create Account with Storage

**Storage slot naming convention** (CRITICAL):
```
miden::component::[snake_case(package.metadata.component.package)]::[field_name]
```

Examples:
- Package `miden:counter-account`, field `count_map` -> `miden::component::miden_counter_account::count_map`
- Package `miden:bank-account`, field `balances` -> `miden::component::miden_bank_account::balances`

Rule: Replace colons and hyphens with underscores in the package name.

See [counter_test.rs](../../../integration/tests/counter_test.rs) lines 33-51 for a complete StorageMap example with `StorageSlotName`, `StorageSlot::with_map`, `AccountCreationConfig`, and `create_testing_account_from_package`.

For a Value slot (single Word) instead of a StorageMap:
```rust
let value_slot_name = StorageSlotName::new("miden::component::miden_bank_account::initialized").unwrap();
let storage_slots = vec![StorageSlot::with_value(
    value_slot_name.clone(),
    Word::default(),
)];
```

### 6. Create Notes

See [counter_test.rs](../../../integration/tests/counter_test.rs) lines 54-58 for basic note creation with `create_testing_note_from_package`.

For notes with assets and inputs:
```rust
use miden_client::note::NoteAssets;
use miden_client::asset::FungibleAsset;

let note_assets = NoteAssets::new(vec![FungibleAsset::new(faucet.id(), 50)?.into()])?;
let note = create_testing_note_from_package(
    note_package.clone(),
    sender.id(),
    NoteCreationConfig {
        assets: note_assets,
        inputs: vec![Felt::new(42), Felt::new(0)],
        ..Default::default()
    },
)?;
```

### 7. Add to MockChain and Build

See [counter_test.rs](../../../integration/tests/counter_test.rs) lines 61-65 for adding accounts, notes, and building the mock chain.

### 8. Execute Transaction

See [counter_test.rs](../../../integration/tests/counter_test.rs) lines 67-79 for the full execution flow: `build_tx_context` -> `execute()` -> `apply_delta()` -> `add_pending_executed_transaction()` -> `prove_next_block()`.

### 9. Execute with Transaction Script
```rust
use miden_client::transaction::TransactionScript;

let tx_script_package = Arc::new(build_project_in_dir(
    Path::new("../contracts/my-tx-script"),
    true,
)?);
let program = tx_script_package.unwrap_program();
let tx_script = TransactionScript::new((*program).clone());

let tx_context = mock_chain
    .build_tx_context(account.id(), &[], &[])?
    .tx_script(tx_script)
    .build()?;

let executed = tx_context.execute().await?;
account.apply_delta(executed.account_delta())?;
mock_chain.add_pending_executed_transaction(&executed)?;
mock_chain.prove_next_block()?;
```

### 10. Verify Storage State

See [counter_test.rs](../../../integration/tests/counter_test.rs) lines 82-92 for reading a StorageMap value and asserting on the result.

### 11. Verify Output Notes

**Important**: `add_output_note()` is only available on `MockChainBuilder` (before `build()`) — use it to seed the chain with existing notes. To verify output notes from a transaction, use `extend_expected_output_notes()` on `TxContextBuilder`:

```rust
use miden_client::note::{Note, NoteAssets, NoteMetadata, NoteRecipient};

let expected_note = Note::new(expected_assets, expected_metadata, expected_recipient);

let tx_context = mock_chain
    .build_tx_context(account.id(), &[note.id()], &[])?
    .extend_expected_output_notes(vec![OutputNote::Full(expected_note)])
    .build()?;

// execute() will verify output notes match
let executed = tx_context.execute().await?;
```

## Multi-Transaction Test Pattern

For contracts requiring initialization before use, each step needs its own execute → `apply_delta()` → `add_pending_executed_transaction()` → `prove_next_block()` cycle.

See [miden-bank withdraw_test.rs](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/integration/tests/withdraw_test.rs) for a complete multi-transaction test demonstrating: initialize bank → deposit assets → withdraw assets (3 sequential transactions with state verification between each step).

See [miden-bank deposit_test.rs](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/integration/tests/deposit_test.rs) for asset-bearing note construction using `NoteAssets::new()` with `FungibleAsset`.

## MockChain Block Numbering

Genesis is block 0. Each `prove_next_block()` advances the block number by 1. In contract code, `tx::get_block_number()` returns the **reference block** — the last proven block at the time the transaction started, not the block the transaction will be included in.

## Note Construction

Always use `create_testing_note_from_package` (or mirror its logic with `.masp` package files) for creating notes in tests. Manually constructed notes may fail with a "private notes cannot be converted" error. See [counter_test.rs](../../../integration/tests/counter_test.rs) for the working pattern.

## Asset-Bearing Note Example

To create a note that carries fungible assets in tests:

1. Create a `FungibleAsset` from a faucet ID and amount.
2. Wrap it in `NoteAssets::new(vec![Asset::Fungible(fungible_asset)])`.
3. Pass the `NoteAssets` into `NoteCreationConfig { assets: note_assets, ..Default::default() }`.
4. Use `create_testing_note_from_package` as usual.

The faucet must be set up first (see Step 3) and the sender wallet must hold sufficient assets (see Step 2).

See [miden-bank deposit_test.rs](https://github.com/0xMiden/tutorials/blob/main/examples/miden-bank/integration/tests/deposit_test.rs) lines 56-70 for the complete working pattern, including `FungibleAsset::new()`, `NoteAssets::new()`, and `NoteCreationConfig` usage.

## Key Dependencies

See [integration/Cargo.toml](../../../integration/Cargo.toml) for the current dependency versions used in this project.

## Validation Checklist

- [ ] Test function is `async` and uses `#[tokio::test]`
- [ ] Storage slot names follow `miden::component::package_name::field_name` pattern
- [ ] All contracts built before account/note creation
- [ ] `apply_delta()` called after each `execute()`
- [ ] `prove_next_block()` called after `add_pending_executed_transaction()`
- [ ] Notes added to `MockChainBuilder` via `add_output_note(OutputNote::Full(...))` (before `build()`)
- [ ] Faucet set up before creating assets
