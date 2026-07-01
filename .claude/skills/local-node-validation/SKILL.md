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
4. **No version negotiation** -- MockChain skips the protocol-version check that a live node negotiates at connect (a mismatched node is rejected).
5. **Account update block numbers not tracked** -- MockChain returns chain tip instead of actual update block number.
6. **No mempool or batching** -- MockChain does not simulate transaction queuing, batch formation, or block inclusion delays.
7. **`NoteTag(0)` notes are not delivered by default subscriptions** -- live nodes filter `SyncNotes` responses by the tag list each client has subscribed to. `NoteTag::new(0)` has all-zero routing bits, so the default account-derived subscription that `add_account` registers (`NoteTagRecord::with_account_source(NoteTag::with_account_target(id), id)`) does not match it. Such notes still exist on-chain and remain queryable, but a client only receives them during sync if it explicitly tracks tag 0 via `Client::add_note_tag(...)`. MockChain bypasses sync filtering and surfaces these notes anyway, hiding the gap until live validation. Prefer `NoteTag::with_account_target(account_id)`, a use-case tag constructor, or an explicit `add_note_tag` subscription when notes must reach the recipient via sync.

## Prerequisites

- [ ] MockChain integration tests pass: `cargo test -p integration --release`
- [ ] A Miden node available locally. The node is **not** a single binary -- it is composed of standalone executables (validator, sequencer, ntx-builder, transaction prover). The client's own test infra installs the full set: `miden-validator`, `miden-node`, `miden-ntx-builder`, `miden-remote-prover` (see `scripts/start-test-node.sh` in the `miden-client` repo). Install them from the node source pinned in your client's `Cargo.lock`, or follow the 0xMiden/node quickstart for the authoritative install flow. Match the node to the `miden-client` version in `integration/Cargo.toml` (`miden-client = "0.15"`); `midenup` manages matched toolchains.
- [ ] Working integration binary exists in `integration/src/bin/` (the testnet binary `increment_count.rs` is the starting template for the localhost variant)

> The local-node launch CLI lives in the 0xMiden/node repo, not in `miden-client`. The commands below are the topology the client's `scripts/start-test-node.sh` drives; confirm exact flags against your installed node's `--help` for the version you run.

## Step 1: Clean State and Start Local Node

**Every node session must start from clean state.** Stale store files and keystore directories cause conflicts, deserialization errors, and misleading test results. Always wipe before starting. (Node and client artifacts also do not round-trip across protocol versions, so a fresh store is required after any version change.)

The simplest path is the client's bundled helper script, which installs the node binaries (pinned to your `Cargo.lock`), generates genesis, bootstraps each component, and starts the split topology for you:

```bash
# From a checkout of the miden-client repo pinned to your client version:
./scripts/start-test-node.sh            # foreground, streams logs; Ctrl+C stops
# or
./scripts/start-test-node.sh --background   # returns once RPC is ready (used by CI)
```

This brings up the four-component topology and exposes the RPC on `127.0.0.1:57291` (the client default, `MIDEN_NODE_PORT`).

If you run the node binaries directly instead of via the script, the shape is below. Treat it as a reference skeleton, not a copy-paste recipe: it omits details the script handles for you (it does not show generating the genesis config the validator bootstraps from, and it leaves out the shared network-tx auth header that the sequencer and ntx-builder must agree on or the sequencer rejects the ntx-builder's transactions). Verify every subcommand and flag against `--help` for your node version, or just use the script.

```bash
# 1. Bootstrap each component from a generated genesis block
#    (the genesis config/block must be produced first; the helper script
#     builds it from the client repo before this step)
miden-validator   bootstrap --data-directory <data>/validator \
  --genesis-block-directory <data>/genesis --accounts-directory <data>/accounts \
  --genesis-config-file <data>/genesis-config/genesis.toml
miden-node        bootstrap --data-directory <data>/node        --file <data>/genesis/genesis.dat
miden-ntx-builder bootstrap --data-directory <data>/ntx-builder --file <data>/genesis/genesis.dat

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

**This clean-start sequence is mandatory every time.** Do not attempt to reuse state from a previous session.

## Step 2: Adapt helpers.rs for Localhost

In `integration/src/helpers.rs`, add a `setup_local_client()` alongside the existing `setup_client()`.

`.sqlite_store(..)` is **not** an inherent `ClientBuilder` method -- it comes from an extension trait in the `miden-client-sqlite-store` crate. It must be in scope or the call fails to compile (method not found). `helpers.rs` already imports it at the top of the file:

```rust
use miden_client_sqlite_store::ClientBuilderSqliteExt; // required for .sqlite_store(..)
```

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
4. Create accounts, create notes, submit transactions via `client.submit_new_transaction(...)`
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
| Version mismatch error | Node and client crate versions differ | Run a node built from the node source pinned in your client's `Cargo.lock` (match `miden-client = "0.15"` in `integration/Cargo.toml`); the protocol version is negotiated at connect and a mismatch is rejected |
| Vite or proxy returns 404 on RPC calls from frontend | Proxy targets the wrong path prefix | gRPC paths are `/rpc.Api/<method>` (e.g. `/rpc.Api/Status`, `/rpc.Api/SyncNotes`, `/rpc.Api/GetAccount`); forward the `/rpc.Api` prefix in the proxy config |
| Transaction rejected | Invalid proof or state | Check contract code, reset node data, try again |
| Account not found after `add_account()` | `add_account()` is local-only; it does not register the account on-chain | Submit a transaction involving the account to deploy it on-chain, then `sync_state()` |
| Store errors or deserialization failures | Stale state from a previous session, or artifacts from an earlier protocol version, which do not round-trip | Wipe the node data, keystore, and client store (`rm -rf local-node-data/ local-keystore/ local-store.sqlite3`), then re-bootstrap from a fresh genesis |
| `.sqlite_store(..)` does not compile | Extension trait not in scope | `use miden_client_sqlite_store::ClientBuilderSqliteExt;` |
| Block not produced | Node produces blocks on the sequencer's configured cadence | Submit a transaction; check the sequencer's `--block.interval` (and `--batch.interval`) settings, or consult `miden-node sequencer --help` |

## Cross-References

- `miden-client-cli`: for driving a running node from the shell (create accounts, mint, send, consume notes) instead of a Rust binary; pair it with this skill's Step 1 node bootstrap for localhost workflows.
