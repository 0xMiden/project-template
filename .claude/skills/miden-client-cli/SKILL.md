---
name: miden-client-cli
description: Map to the official Miden client CLI. This project uses the exact `miden-client-cli 0.16.0-rc.2` binary, installed directly and invoked as `miden-client ...`; a midenup-managed `miden client ...` component is acceptable only when its reported version is the same exact RC. Covers install, init, network selection, and canonical command/configuration references. Use when an agent needs to create accounts, query state, mint, transfer, or consume notes against a running Miden node; pair with the local-node-validation skill for localhost workflows.
---

# Miden Client CLI

The Miden client CLI is the command-line wrapper around the `miden-client` library. It creates accounts, syncs state, mints assets, submits transactions, and consumes notes against a running Miden node.

This skill maps agents to the exact project version and its upstream command reference. Do not mix a store or command set from another client release into the v0.16 workflow.

When to reach for this skill: the user wants to interact with a node from the shell. For Rust-library work against a localhost node from a binary, see the `local-node-validation` skill. For test-side note construction, see `rust-sdk-testing-patterns`.

## Exact Project Installation

Install and verify the same client RC used by `integration/Cargo.toml`:

```sh
cargo install miden-client-cli --version 0.16.0-rc.2 --locked
test "$(miden-client --version)" = "miden-client 0.16.0-rc.2"
```

The midenup `0.16.0` channel does not supply this exact client RC. `miden client <args>` remains a valid component-invocation mechanism for a managed toolchain, but use it for this project only after `miden client --version` reports exactly `miden-client 0.16.0-rc.2`. Do not assume a channel name proves component identity.

The exact RC uses `miden-client` and `miden-client-sqlite-store` `0.16.0-rc.2`, backed by protocol/standards/testing `0.16.0-rc.6`.

References:
- midenup install, init, toolchain delegation, and alias docs: [github.com/0xMiden/midenup](https://github.com/0xMiden/midenup).
- Pinned CLI install and setup: [miden-client v0.16.0-rc.2 `bin/miden-cli/README.md`](https://github.com/0xMiden/miden-client/blob/v0.16.0-rc.2/bin/miden-cli/README.md).

## First-Time Initialization

`init` is optional because the CLI can self-initialize. Explicit initialization is safer when selecting a network and a fresh store. By default it creates global configuration at `~/.miden/miden-client.toml`; pass `--local` to create `./.miden/miden-client.toml`. A local config takes precedence over the global config.

For this repository's temporary DevNet runtime, use a fresh v0.16 local configuration and store:

```sh
miden-client init --local --network devnet --store-path store.sqlite3
```

Omitting `--network` selects Testnet. Other accurate choices are `localhost`, `testnet`, or a full custom HTTP(S) RPC URL. Do not open a pre-v0.16 or different-network SQLite store with this client; archive it and initialize a fresh path. For localhost workflows, pair this skill with `local-node-validation`.

Current v0.16 command changes include `transfer` in place of the removed `send` subcommand, and `account --inspect <ID> --verbose` in place of the removed `account --show --with-code`. Use `miden-client <command> --help` for the exact installed command surface. The old client/CLI debug-mode toggle and `--debug` flag are removed.

## Canonical Command Reference

Follow the canonical references at the exact `v0.16.0-rc.2` tag, which matches project-template's pinned client.

- CLI Reference: [`docs/external/src/rust-client/cli/index.md`](https://github.com/0xMiden/miden-client/blob/v0.16.0-rc.2/docs/external/src/rust-client/cli/index.md)
- CLI Configuration: [`docs/external/src/rust-client/cli/cli-config.md`](https://github.com/0xMiden/miden-client/blob/v0.16.0-rc.2/docs/external/src/rust-client/cli/cli-config.md)
- Release behavior: [`CHANGELOG.md`](https://github.com/0xMiden/miden-client/blob/v0.16.0-rc.2/CHANGELOG.md).

For the live command list and flags, run `miden-client --help` and drill into a command with `miden-client <command> --help`. If an exact-version midenup component is deliberately selected, the equivalent invocation is `miden client ...`.

## Cross-References

- `local-node-validation`: Rust-binary path against a localhost node, plus node bootstrap and a clean-keystore recipe.
- `rust-sdk-testing-patterns` ("Note Construction"): building notes from compiled `.masp` packages in tests.
- `rust-sdk-patterns` ("Cross-Component Note Pattern"): note scripts that read inputs and call account-component methods, the source of many `consume-notes` flows.
