---
name: local-node-validation
description: Validates Miden contracts against a local node. Covers node setup, Rust binary adaptation, state verification, and troubleshooting. Use after MockChain tests pass to verify contracts work against a real node.
---

# Local Node Validation

Validates that contracts working in MockChain also work against a real Miden node. This catches known MockChain/live-node behavior gaps before they become harder to debug in production or the frontend.

## Why This Matters

MockChain simplifies execution in ways that hide real-world failures:

1. **No automatic block production** -- MockChain requires explicit `prove_next_block()`. A live node produces blocks on the sequencer's configured cadence.
2. **No network transport** -- MockChain does not simulate the network transaction builder (ntx-builder) that handles network notes.
3. **No RPC latency or timeouts** -- MockChain executes locally and instantly. Live nodes have gRPC round-trips with configurable timeouts.
4. **No live compatibility enforcement** -- MockChain skips the RPC Accept-header check by which a live node rejects incompatible client requests.
5. **Account update block numbers not tracked** -- MockChain returns chain tip instead of actual update block number.
6. **No mempool or batching** -- MockChain does not simulate transaction queuing, batch formation, or block inclusion delays.
7. **`NoteTag(0)` notes are not delivered by default subscriptions** -- live nodes filter `SyncNotes` responses by the tag list each client has subscribed to. `NoteTag::new(0)` has all-zero routing bits, so the default account-derived subscription that `add_account` registers (`NoteTagRecord::with_account_source(NoteTag::with_account_target(id), id)`) does not match it. Such notes still exist on-chain and remain queryable, but a client only receives them during sync if it explicitly tracks tag 0 via `Client::add_note_tag(...)`. MockChain bypasses sync filtering and surfaces these notes anyway, hiding the gap until live validation. Prefer `NoteTag::with_account_target(account_id)`, a use-case tag constructor, or an explicit `add_note_tag` subscription when notes must reach the recipient via sync.

## Prerequisites

- [ ] MockChain integration tests pass: `cargo test -p integration --release`
- [ ] A Miden node available locally. The node is **not** a single binary -- it is composed of standalone executables (validator, sequencer, ntx-builder, transaction prover). The client's own test infra installs the full set: `miden-validator`, `miden-node`, `miden-ntx-builder`, `miden-remote-prover` (see `scripts/start-test-node.sh` in the `miden-client` repo). Install them from exact source, or follow the 0xMiden/node quickstart for the authoritative install flow. This project pins `miden-client` and its SQLite store to `0.16.0-rc.2` with protocol/standards/testing `0.16.0-rc.6`; its accepted real-node target is exactly `miden-node v0.16.0-rc.1`, whose source manifest pins protocol `0.16.0-rc.4`. Treat that correspondence as the source-derived validation pairing whose runtime status must still be verified before use, not as equality between the two protocol crate versions and not as permission to accept an arbitrary v0.16 node.
- [ ] Working integration binary exists in `integration/src/bin/` (the current `increment_count.rs` targets DevNet; it is the behavior reference when creating a separate localhost validator)

> The local-node launch CLI lives in the 0xMiden/node repo, not in `miden-client`. The commands below are the topology the client's `scripts/start-test-node.sh` drives; confirm exact flags against your installed node's `--help` for the version you run.

## Step 1: Clean State and Start Local Node

**Every node session must start from clean, task-specific state.** Stale store files and keystore directories cause conflicts, deserialization errors, and misleading test results. Choose fresh paths before starting; move prior state to a recoverable private backup when it must be displaced. Node and client artifacts do not round-trip across protocol versions, so a fresh store is required after any version change.

The simplest path is the client's bundled helper script, which installs the node binaries (pinned to your `Cargo.lock`), generates genesis, bootstraps each component, and starts the split topology for you:

```bash
# From a checkout of the miden-client repo pinned to your client version:
./scripts/start-test-node.sh            # foreground, streams logs; Ctrl+C stops
# or
./scripts/start-test-node.sh --background   # returns once RPC is ready (used by CI)
```

This brings up the four-component topology and exposes the RPC on `127.0.0.1:57291` (the client default, `MIDEN_NODE_PORT`).

If you run the node binaries directly instead of via the script, the shape is below. Treat it as a reference skeleton, not a copy-paste recipe: it omits details the script handles for you (generating the genesis config, supplying the validator's threshold storage-key material, and providing the shared network-tx auth header that the sequencer and ntx-builder must agree on or the sequencer rejects the ntx-builder's transactions). Verify every subcommand and flag against `--help` for your node version, or just use the script.

```bash
# 1. Generate the genesis block, then bootstrap each component from it.
#    The helper script first generates <data>/genesis-config and required account files.
miden-validator genesis --genesis-block-directory <data>/genesis \
  --accounts-directory <data>/accounts --config <data>/genesis-config/genesis.toml
miden-validator   bootstrap --data-directory <data>/validator   --genesis <data>/genesis/genesis.dat
miden-node        bootstrap --data-directory <data>/node        --genesis <data>/genesis/genesis.dat
miden-ntx-builder bootstrap --data-directory <data>/ntx-builder --genesis <data>/genesis/genesis.dat

# 2. Start the components (validator, then sequencer with the RPC, prover, ntx-builder).
#    The sequencer and ntx-builder additionally need a matching network-tx auth header
#    (--rpc.network-tx-auth-header-value / --rpc.auth-header-value in the script); see the script.
miden-validator start --listen 127.0.0.1:50101 --data-directory <data>/validator
miden-node sequencer --rpc.listen 127.0.0.1:57291 --data-directory <data>/node \
  --validator.url http://127.0.0.1:50101 --ntx-builder.url http://127.0.0.1:50301 \
  --block.interval 3s --batch.interval 1s
miden-remote-prover --kind=transaction --port=50051
miden-ntx-builder start --listen 127.0.0.1:50301 --rpc.url http://127.0.0.1:57291 \
  --tx-prover.url http://127.0.0.1:50051 --data-directory <data>/ntx-builder
```

**This clean-start sequence is mandatory every time.** Do not open prior-session state with the new node; use fresh paths or archive the prior state first.

## Step 2: Adapt helpers.rs for Localhost

In `integration/src/helpers.rs`, add a `setup_local_client()` alongside the existing `setup_client()`.

`.sqlite_store(..)` is **not** an inherent `ClientBuilder` method -- it comes from an extension trait in the `miden-client-sqlite-store` crate. It must be in scope or the call fails to compile (method not found). `helpers.rs` already imports it at the top of the file:

```rust
use miden_client_sqlite_store::ClientBuilderSqliteExt; // required for .sqlite_store(..)
```

```rust
pub async fn setup_local_client() -> Result<ClientSetup> {
    let endpoint = Endpoint::localhost();
    let timeout_ms = 10_000;

    let keystore_path = std::path::PathBuf::from("../local-keystore");
    let keystore = Arc::new(FilesystemKeyStore::new(keystore_path)
        .context("Failed to initialize local keystore")?);

    let store_path = std::path::PathBuf::from("../local-store.sqlite3");

    let client = ClientBuilder::new()
        .grpc_client(&endpoint, Some(timeout_ms))
        .sqlite_store(store_path)
        .authenticator(keystore.clone())
        .build()
        .await
        .context("Failed to build local Miden client")?;

    Ok(ClientSetup { client, keystore })
}
```

Use separate paths (`local-keystore/`, `local-store.sqlite3`) to avoid contaminating remote-network state. Client debug mode was removed in v0.16; do not restore the removed builder toggle or the `MIDEN_DEBUG` environment switch.

## Step 3: Create Local Validation Binary

Create `integration/src/bin/validate_local.rs` mirroring the existing DevNet binary (`increment_count.rs`) but using `setup_local_client()`.

The binary must:
1. Call `setup_local_client()` instead of `setup_client()`
2. Sync state: `client.sync_state().await?`
3. Build contracts (same as existing binary)
4. Create accounts, create notes, submit transactions via `client.submit_new_transaction(...)`
5. Sync again after each transaction submission
6. Wait for transaction inclusion (poll `sync_state` until account state updates)
7. Verify final state matches MockChain test expectations
8. Print clear pass/fail for each verification step

The first sync is mandatory in v0.16: transaction inputs are sealed before submission, and the client needs trusted local genesis and chain-tip headers to verify the validator set's shared encryption key. `submit_new_transaction` handles sealing; the paired node must support unsealing and rejects plaintext inputs.

Key differences from the current DevNet binary:
- Localhost endpoint (port 57291)
- Separate keystore and store paths
- Must handle block production timing (sync + wait between submissions)

**Account deployment**: `Client::add_account()` only writes the account to the local client store; it does not register the account on-chain. To make a public or network account discoverable by other clients, submit a transaction involving the account (typically the account's first transaction). Until that transaction is included in a block, `get_account_details(id)` from any other client returns "not found".

## Step 4: Run and Verify

Ensure clean client state before running (the node should already be clean from Step 1). A pre-v0.16 or other-network SQLite database is not migratable evidence: move it to a recoverable backup or choose a fresh task-specific store path before the v0.16 client opens it. Keep keystores separate by network and do not inspect or print secret material.

Before submitting, inspect the chain's `verification_base_fee`. The current project flow has unfunded `AuthSingleSig` and `NoAuth` accounts and is valid unchanged only when that value is zero. On a fee-charging chain, signature auth requires committed fee-conversion information and a funded payment asset; `NoAuth` pays in the native fee asset at 1/1, must be funded in that asset, and does not accept explicit conversion information. Stop rather than silently adding funding or changing auth.

```bash
cargo run --bin validate_local --release
```

### Verification Checklist

- [ ] `sync_state()` succeeds (node reachable, no version mismatch)
- [ ] The sync stores trusted genesis and chain-tip headers before the first sealed submission
- [ ] `verification_base_fee` is compatible with the accounts' funding and auth setup (zero for the unchanged project flow)
- [ ] Account creation succeeds (account appears after sync)
- [ ] Note publication succeeds (transaction accepted by node)
- [ ] Note consumption succeeds (state transitions as expected)
- [ ] Final state matches MockChain test expectations
- [ ] No RPC timeout errors
- [ ] Node logs show no errors

## Step 5: Inspect Node Logs

Run the node with verbose logging. The helper script honors `RUST_LOG` and writes a per-component log file per service; if you launch the binaries directly, set it on the process you want to inspect (the sequencer carries the RPC):

```bash
RUST_LOG=info ./scripts/start-test-node.sh
# or, running the sequencer directly:
RUST_LOG=info miden-node sequencer --rpc.listen 127.0.0.1:57291 --data-directory <data>/node \
  --validator.url http://127.0.0.1:50101 --ntx-builder.url http://127.0.0.1:50301 \
  --block.interval 3s --batch.interval 1s
```

Look for:
- Transaction acceptance/rejection messages
- Block production confirmations
- Error or warning lines

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Unavailable` RPC error | Node not running or wrong port | Start node, verify the sequencer's RPC is listening on 57291 |
| Version mismatch error | The node/client stack is not the accepted v0.16 pairing | For this project, require client/store `0.16.0-rc.2`, protocol/standards/testing `0.16.0-rc.6`, and node `0.16.0-rc.1`; verify the node's source/tag and runtime status rather than accepting a generic v0.16 label |
| Submission fails before RPC acceptance | The fresh store has not synced trusted genesis/tip headers, or the node cannot unseal v0.16 inputs | Sync first and use the exact v0.16 node/client pairing; do not downgrade to plaintext submission |
| Authentication aborts while paying a fee | The chain has a nonzero base fee but the account lacks the required fee asset/conversion setup | Fund and configure fees only as an explicitly designed behavior change; the unchanged project requires a zero verification base fee |
| Vite or proxy returns 404 on RPC calls from frontend | Proxy targets the wrong path prefix | gRPC paths are `/rpc.Api/<method>` (e.g. `/rpc.Api/Status`, `/rpc.Api/SyncNotes`, `/rpc.Api/GetAccount`); forward the `/rpc.Api` prefix in the proxy config |
| Transaction rejected | Invalid proof or state | Check contract code, reset node data, try again |
| Account not found after `add_account()` | `add_account()` is local-only; it does not register the account on-chain | Submit a transaction involving the account to deploy it on-chain, then `sync_state()` |
| Store errors or deserialization failures | Stale state from a previous session, or artifacts from an earlier protocol version, which do not round-trip | Re-bootstrap node data from a fresh genesis and give the v0.16 client a fresh SQLite path; archive an existing store instead of opening it with the new client |
| `.sqlite_store(..)` does not compile | Extension trait not in scope | `use miden_client_sqlite_store::ClientBuilderSqliteExt;` |
| Block not produced | Node produces blocks on the sequencer's configured cadence | Submit a transaction; check the sequencer's `--block.interval` (and `--batch.interval`) settings, or consult `miden-node sequencer --help` |

## Cross-References

- `miden-client-cli`: for driving a running node from the shell (create accounts, mint, transfer, consume notes) instead of a Rust binary; pair it with this skill's Step 1 node bootstrap for localhost workflows.
