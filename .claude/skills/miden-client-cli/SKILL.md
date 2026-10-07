---
name: miden-client-cli
description: Map to the official Miden client CLI for the current project-template v17 line. Use midenup's managed `miden client ...` component when it matches the intended workflow; use direct `miden-client-cli` installation when exact parity with this repository's resolved `Cargo.lock` client is required. Covers install, init, network selection, and canonical command/configuration references. Use when an agent needs to create accounts, query state, mint, transfer, or consume notes against a running Miden node; pair with the local-node-validation skill for localhost workflows.
---

# Miden Client CLI

The Miden client CLI is the command-line wrapper around the `miden-client` library. It creates accounts, syncs state, mints assets, submits transactions, and consumes notes against a running Miden node.

This skill maps agents to the managed midenup invocation and the exact client version currently resolved by this repository's lockfile. Do not mix a store or command set from another client release into the v0.17 workflow.

When to reach for this skill: the user wants to interact with a node from the shell. For Rust-library work against a localhost node from a binary, see the `local-node-validation` skill. For test-side note construction, see `rust-sdk-testing-patterns`.

## Installation and Version Check

The repository README uses midenup for the managed Miden toolchain. After installing and initializing midenup, the `miden` wrapper delegates `miden client <args>` to the toolchain's `client` component, whose installed executable is `miden-client`. Always check the installed component before using an existing store:

```sh
miden client --version
```

Midenup components are independent of this repository's `Cargo.lock`, so use the managed component only after the version check matches the workflow you are validating. When exact parity with the current lockfile is required, install and verify the resolved client directly:

```sh
cargo install miden-client-cli --version 0.17.2 --locked
test "$(miden-client --version)" = "miden-client 0.17.2"
```

`integration/Cargo.toml` lower-bounds `miden-client` and `miden-client-sqlite-store` at `0.17.0`; `Cargo.lock` currently resolves both to `0.17.2`, backed by protocol/standards/testing `0.17.1`.

References:
- midenup install, init, toolchain delegation, and component docs: [github.com/0xMiden/midenup](https://github.com/0xMiden/midenup).
- Exact direct CLI install and setup: [miden-client v0.17.2 `bin/miden-cli/README.md`](https://github.com/0xMiden/miden-client/blob/v0.17.2/bin/miden-cli/README.md).

## First-Time Initialization

`init` is optional because the CLI can self-initialize. Explicit initialization is safer when selecting a network and a fresh store. By default it creates global configuration at `~/.miden/miden-client.toml`; pass `--local` to create `./.miden/miden-client.toml`. A local config takes precedence over the global config.

For this repository's Testnet runtime, use a fresh v0.17 local configuration and store. With direct install:

```sh
miden-client init --local --network testnet --store-path store.sqlite3
```

With midenup, the equivalent shape is `miden client init --local --network testnet --store-path store.sqlite3`. Omitting `--network` selects Testnet. For local validation, use `localhost` or a custom HTTP(S) URL serving a stable v0.17 node. A custom RPC URL also needs a matching `--note-transport-endpoint`. Do not open a v0.16, v0.17 release-candidate, or different-network SQLite store with this client; preserve the old state and initialize a fresh path. Old account/note exports do not import into v0.17, and old bundled `.miden/packages` must be rebuilt. For localhost workflows, pair this skill with `local-node-validation`.

The CLI uses `transfer` in place of the removed `send` subcommand, and `account --inspect <ID> --verbose` in place of the removed `account --show --with-code`. Use `miden-client <command> --help` for the exact installed command surface. The old client/CLI debug-mode toggle and `--debug` flag are removed. In v0.17, `exec` takes `--package` instead of `--script-path`; supply a rebuilt `.masp` package. Native fee funding and any node invitation/registration requirements apply before submission.

## Canonical Command Reference

Follow the canonical references at the exact `v0.17.2` tag, which matches the client currently resolved in project-template's `Cargo.lock`.

- CLI Reference: [`docs/external/src/rust-client/cli/index.md`](https://github.com/0xMiden/miden-client/blob/v0.17.2/docs/external/src/rust-client/cli/index.md)
- CLI Configuration: [`docs/external/src/rust-client/cli/cli-config.md`](https://github.com/0xMiden/miden-client/blob/v0.17.2/docs/external/src/rust-client/cli/cli-config.md)
- Release behavior: [`CHANGELOG.md`](https://github.com/0xMiden/miden-client/blob/v0.17.2/CHANGELOG.md).

For the live command list and flags, run `miden-client --help` and drill into a command with `miden-client <command> --help`. If an exact-version midenup component is deliberately selected, the equivalent invocation is `miden client ...`.

## Cross-References

- `local-node-validation`: Rust-binary path against a localhost node, plus node bootstrap and a clean-keystore recipe.
- `rust-sdk-testing-patterns` ("Note Construction"): building notes from compiled `.masp` packages in tests.
- `rust-sdk-patterns` ("Cross-Component Note Pattern"): note scripts that read inputs and call account-component methods, the source of many `consume-notes` flows.
