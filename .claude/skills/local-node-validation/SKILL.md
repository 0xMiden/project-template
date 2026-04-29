---
name: local-node-validation
description: Validates Miden contracts against a local node. Covers node setup, Rust binary adaptation, state verification, and troubleshooting. Use after MockChain tests pass to verify contracts work against a real node.
---

# Local Node Validation

Validates that contracts working in MockChain also work against a real Miden node. This catches known MockChain/live-node behavior gaps before they become harder to debug in production or the frontend.

## Why This Matters

MockChain simplifies execution in ways that hide real-world failures:

1. **No automatic block production** -- MockChain requires explicit `prove_next_block()`. A live node produces blocks on its own schedule.
2. **No network transport** -- MockChain does not simulate the network transaction builder that handles network notes.
3. **No RPC latency or timeouts** -- MockChain executes locally and instantly. Live nodes have gRPC round-trips with configurable timeouts.
4. **No version/genesis validation** -- MockChain skips the `Accept` header version check that live nodes enforce.
5. **Account update block numbers not tracked** -- MockChain returns chain tip instead of actual update block number.
6. **No mempool or batching** -- MockChain does not simulate transaction queuing, batch formation, or block inclusion delays.
7. **Genesis hash is cached in the client store** -- after the first sync, the client persists the network's genesis commitment in `local-store.sqlite3` and ships it on every Accept header. Switching networks (local node to testnet, or vice versa) without wiping the store fails with `accept header validation failed`.
8. **`NoteTag(0)` notes are invisible during live sync** -- live nodes filter notes by client tag subscriptions. `NoteTag::new(0)` produces all-zero routing bits that no subscription matches, so such notes never reach the client. MockChain bypasses sync filtering, so the bug only surfaces on a real node. Use `NoteTag::with_account_target(account_id)` (or a use-case constructor) when targeting a specific account.

## Prerequisites

- [ ] MockChain integration tests pass: `cargo test -p integration --release`
- [ ] `miden-node` installed and version-matched with the client. Check the `miden-client` version in `integration/Cargo.toml`; the node binary must be on the same minor release. `cargo install miden-node --locked` may pin an older published crate; if so, install from source: `cargo install miden-node --locked --git https://github.com/0xMiden/miden-node --tag v<version>`. `midenup` manages matched toolchains.
- [ ] Working integration binary exists in `integration/src/bin/`

## Step 1: Clean State and Start Local Node

**Every node session must start from clean state.** Stale store files and keystore directories cause conflicts, deserialization errors, and misleading test results. Always wipe before starting.

```bash
# 1. Wipe all state from previous runs
rm -rf local-node-data/ local-keystore/ local-store.sqlite3

# 2. Bootstrap fresh node
mkdir -p local-node-data
miden-node bundled bootstrap \
  --data-directory local-node-data \
  --accounts-directory .

# 3. Start node (keep running in separate terminal)
miden-node bundled start \
  --data-directory local-node-data \
  --rpc.url http://0.0.0.0:57291
```

**This clean-start sequence is mandatory every time.** Do not attempt to reuse state from a previous session.

## Step 2: Adapt helpers.rs for Localhost

In `integration/src/helpers.rs`, add a `setup_local_client()` alongside the existing `setup_client()`:

```rust
pub async fn setup_local_client() -> Result<ClientSetup> {
    let endpoint = Endpoint::new("http".into(), "localhost".into(), Some(57291));
    let timeout_ms = 10_000;
    let rpc_client = Arc::new(GrpcClient::new(&endpoint, timeout_ms));

    let keystore_path = std::path::PathBuf::from("../local-keystore");
    let keystore = Arc::new(FilesystemKeyStore::new(keystore_path)
        .context("Failed to initialize local keystore")?);

    let store_path = std::path::PathBuf::from("../local-store.sqlite3");

    let client = ClientBuilder::new()
        .rpc(rpc_client)
        .sqlite_store(store_path)
        .authenticator(keystore.clone())
        .in_debug_mode(true.into())
        .build()
        .await
        .context("Failed to build local Miden client")?;

    Ok(ClientSetup { client, keystore })
}
```

Use separate paths (`local-keystore/`, `local-store.sqlite3`) to avoid contaminating testnet state.

## Step 3: Create Local Validation Binary

Create `integration/src/bin/validate_local.rs` mirroring the existing testnet binary (`increment_count.rs`) but using `setup_local_client()`.

The binary must:
1. Call `setup_local_client()` instead of `setup_client()`
2. Sync state: `client.sync_state().await?`
3. Build contracts (same as existing binary)
4. Create accounts, create notes, submit transactions
5. Sync again after each transaction submission
6. Wait for transaction inclusion (poll `sync_state` until account state updates)
7. Verify final state matches MockChain test expectations
8. Print clear pass/fail for each verification step

Key differences from testnet binary:
- Localhost endpoint (port 57291)
- Separate keystore and store paths
- Must handle block production timing (sync + wait between submissions)

**Account deployment**: `Client::add_account()` only writes the account to the local client store; it does not register the account on-chain. To make a public or network account discoverable by other clients, submit a transaction involving the account (typically the account's first transaction). Until that transaction is included in a block, `get_account_details(id)` from any other client returns "not found".

## Step 4: Run and Verify

Ensure clean client state before running (the node should already be clean from Step 1):
```bash
rm -rf local-keystore/ local-store.sqlite3
cargo run --bin validate_local --release
```

### Verification Checklist

- [ ] `sync_state()` succeeds (node reachable, no version mismatch)
- [ ] Account creation succeeds (account appears after sync)
- [ ] Note publication succeeds (transaction accepted by node)
- [ ] Note consumption succeeds (state transitions as expected)
- [ ] Final state matches MockChain test expectations
- [ ] No RPC timeout errors
- [ ] Node logs show no errors

## Step 5: Inspect Node Logs

Run the node with verbose logging:
```bash
RUST_LOG=info miden-node bundled start \
  --data-directory local-node-data \
  --rpc.url http://0.0.0.0:57291
```

Look for:
- Transaction acceptance/rejection messages
- Block production confirmations
- Error or warning lines

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Unavailable` RPC error | Node not running or wrong port | Start node, verify port 57291 |
| `accept header validation failed` after switching networks | Client store cached the genesis commitment from a different network | Delete `local-store.sqlite3` and re-sync |
| Version mismatch error | Node binary lags client crate (`cargo install miden-node` may pin an older published crate) | Reinstall to match `miden-client` from `integration/Cargo.toml`: `cargo install miden-node --locked --git https://github.com/0xMiden/miden-node --tag v<version>`, or use `midenup` |
| Vite or proxy returns 404 on RPC calls from frontend | Proxy targets the wrong path prefix | gRPC paths are `/rpc.Api/<method>` (e.g. `/rpc.Api/SyncState`, `/rpc.Api/GetAccount`); forward the `/rpc.Api` prefix in the proxy config |
| Transaction rejected | Invalid proof or state | Check contract code, reset node data, try again |
| Account not found after `add_account()` | `add_account()` is local-only; it does not register the account on-chain | Submit a transaction involving the account to deploy it on-chain, then `sync_state()` |
| Store errors or deserialization failures | Stale state from a previous session, or cached genesis from a different network | Wipe everything: `rm -rf local-node-data/ local-keystore/ local-store.sqlite3` and re-bootstrap |
| Block not produced | Node produces blocks when transactions arrive | Submit a transaction; check `--block-producer.block-interval` setting |
