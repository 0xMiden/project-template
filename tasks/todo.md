# project-template Miden v0.15 to v0.16 migration plan

First independent audit status: **BLOCKED**

Second independent audit status: **NEEDS HUMAN — the requested human authorization is now recorded below; its technical findings are incorporated in this revision**

Third independent audit status: **NEEDS HUMAN — it did not receive the direct human approval messages; its four technical findings are incorporated in this revision**

Previous complete-plan audit status: **PASS for the superseded 327-line task revision; it does not attest this newer task revision**

Current independent audit status: **REVISE — the final implementation audit found a contract build-utils rc.6 leak, two asset-guidance errors, a non-fresh plain-Cargo reproducibility gate, and stale commit/drift evidence; the dependency/guidance findings are fixed in signed correction commit `8de62fe27d0873ab560c891c6ab274e35ceab80e`, and this evidence-only reconciliation addresses the remaining record/reproducibility findings**

Current execution status: **FINAL EVIDENCE RECONCILED; PR-READINESS CORRECTION VERIFIED — signed final commit-evidence correction `5ef123fe1332b87f82ff68f750be60461b6db284` is a one-file child of signed evidence commit `03068ada8ea2bf45fd32660810a4d46bef3d1d03`; the signed commit containing this task record corrects the independently re-audited P3/FPI argument-tupling guidance and records its direct `5ef123fe...` parent without claiming its own SHA; the user explicitly authorized pushing this branch directly to `0xMiden/project-template` and opening a draft PR, but no merge is authorized**

Task source: `/Users/philipp/Documents/Work/Miden-Coding/ai-tasks/v16-migration/tasks/TASK-project-template-v16-migration.md` (**current 353-line revision read in full; SHA-256 `c095f2c61ebb8a91eb0689fc77004176dc9de7333ba2bd3782bb1a1a1ff836ca`**)

Required Rust-contract reference map: `/Users/philipp/Documents/Work/Miden-Coding/ai-tasks/v16-migration/resources/V16-RUST-CONTRACT-REFERENCE-MAP.md` (**read in full on 2026-08-25; 222/222 lines**)

Target repository: `/Users/philipp/Documents/Work/Miden-Coding/project-template`

Implementation branch: `kbg/chore/v16-migration`

Evidence directory: `/Users/philipp/Documents/Work/Miden-Coding/ai-tasks/v16-migration/outputs`

## Objective and invariants

Port the standalone `0xMiden/project-template` repository from Miden v0.15 to the pinned v0.16 RC stack. This is an API migration, not a rewrite.

The implementation must preserve all observable behavior:

- Keep both existing contracts, the single `counter_test`, and the single `increment_count` binary.
- Preserve the test name, test scenario, and final assertion that the stored count is exactly `1`.
- Preserve the binary's zero-argument CLI, execution order, output fields, account roles, note flow, and lack of a post-consumption sync/storage read.
- Apply one narrowly authorized, temporary runtime exception: replace the helper's hard-coded `Endpoint::testnet()` with pinned-client `Endpoint::devnet()` so the real-runtime verification targets `https://rpc.devnet.miden.io`. Do not make the endpoint configurable and do not change any other runtime behavior. Restoring Testnet is a later, separately verified change after Testnet is upgraded.
- Do not refactor, rename for taste, extract helpers, weaken assertions, add features, or copy the compiler scaffold wholesale.
- Keep the contract/compiler dependency line separate from the client/protocol dependency line.
- Do not edit reference repositories, push, open a PR, post to GitHub, or merge anything.
- Stop rather than changing behavior when an API or runtime requirement cannot be satisfied mechanically.

## Recorded human runtime decision

**Local decision record:** `RUNTIME-DEVNET-2026-08-24` (a plan-local correlation label, not an invented platform message ID).

**Approval source and provenance:** the authority is the direct human-authored messages in the originating conversation, not this builder-authored plan, an agent summary, or a generic later approval. On 2026-08-24, after the first audit blocked the Testnet-versus-DevNet contradiction, the user explicitly authorized an **intermediate DevNet endpoint for verification** and stated that the project will be changed back to Testnet after Testnet is upgraded. After the second audit requested a more explicit scope record, the user directly confirmed exactly: “I authorize the temporary hard-coded `Endpoint::devnet()` change, live DevNet account and transaction creation, and application-level insertion of new DevNet keys into the existing keystore.” On 2026-08-25, after the third audit said it had not received those messages, the user directly instructed the builder to “revise everything remaining” and reaffirmed: “you do have my human authiorization for the devnet endpoint.” The 2026-08-24 direct message supplies the complete endpoint, live-side-effect, and existing-keystore scope; the 2026-08-25 direct message reconfirms the endpoint decision.

Any independent re-audit and the eventual implementation agent must inherit the full originating conversation containing that exact human-authored authorization, or receive an independently human-supplied approval record alongside this file. Before any DevNet edit, account/key creation, store replacement, or transaction submission, the executor must confirm that the exact `RUNTIME-DEVNET-2026-08-24` authorization text is visible in its inherited human-message history. This plan must not be presented as independent proof of its own authorization. If an auditor or executor receives only the repository file, an agent summary, or a generic “approve implementation” message, it must mark authorization unverified and stop for direct re-authorization.

Decision-complete interpretation, grounded in `miden-client v0.16.0-rc.2` source:

- The only approved endpoint edit is `integration/src/helpers.rs`: `Endpoint::testnet()` -> `Endpoint::devnet()`.
- At that frozen client tag, `Endpoint::devnet()` is exactly `https://rpc.devnet.miden.io` and maps to `NetworkId::Devnet`.
- The authorized live side effects include public DevNet account creation and transaction submission; any resulting network inclusion cannot be rolled back. Each `increment_count` invocation that successfully completes the application's existing `keystore.add_key` call after `client.add_account` persists one newly generated Falcon-512 sender key. This happens before transaction submission, so a later transaction failure does not roll it back; this exact application-level mutation of the existing keystore is authorized. The `NoAuth` counter adds no auth key.
- The approval does not authorize a CLI flag, environment variable, configuration file, localhost fallback, auth/funding change, alternate transaction flow, extra output, or manual keystore inspection/move/deletion.
- Runtime verification must accept only a status response whose `version` is exactly `0.16.0-rc.1`. The node tag's root `Cargo.toml` must independently prove exact protocol/standards/tx/tx-batch pins `=0.16.0-rc.4`.
- `Endpoint::to_network_id()` and `GrpcClient::get_network_id()` classify the locally configured endpoint; neither is remote network-attestation evidence. The probe may record this as **configured endpoint/network classification** only.
- If the configured endpoint is not exact, DevNet reports another node/block-producer version, or the fee/runtime checks fail, stop. Do not silently use Testnet, a generic v0.16 node, or localhost.

## Recorded human compiler-source decision

After Phase 1 proved that `miden-sdk-build-script-support@0.14.0-rc.1` was not published and that the published/tagged `cargo-miden 0.10.0-rc.1` source lacks the helper's required `--stop-after=dependencies` capability, the user explicitly instructed: “Use the compiler source matching the v16 pipeline, even if it has not been released.”

Decision-complete interpretation:

- Freeze the authorized compiler source to `COMPILER_PIPELINE_COMMIT=2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`, the final functional v0.16 pipeline merge before the previously reviewed snapshot's unrelated repository publish-age configuration. Never follow the moving `origin/next` ref after recording it.
- Install `cargo-miden` and `midenc` from that exact Git revision into the isolated v0.16 tool root. Both must still print `0.10.0-rc.1`, but the evidence must also record the source revision because version output alone cannot distinguish this post-tag pipeline.
- Use exact immutable Git sources at `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a` for both `miden-sdk-build-script-support` and guest `miden`, with guest version requirement `=0.14.0-rc.1`. The build-support registry package is unpublished; the published guest rc.1 payload predates the same-version pipeline's embedded-WIT implementation and reproduces `package 'miden:counter-account@0.1.0' not found` after the obsolete WIT key is removed. A disposable two-contract probe proved that changing only the guest source to the authorized pipeline makes both packages build. Every non-compiler host/runtime pin remains unchanged.
- The compatibility probe must exercise this exact source-aligned binary/helper pair. A moving branch, local path dependency, vendored copy, manually supplied package cache, or nearby compiler commit is not authorized.
- Prove the no-v17 boundary before installation: the selected commit's compiler/SDK versions remain `0.10.0-rc.1`/`0.14.0-rc.1`, its compiler dependencies are protocol `=0.16.0-rc.4` and VM `0.29`, its changelogs explicitly classify the changes as protocol 0.16 migration work, and focused release/config/changelog scans contain no protocol 0.17, SDK 0.15, or compiler 0.11 line. The selected functional trees are byte-identical to the later `62c4318...` research snapshot; the only intervening path is `.cargo/config.toml`, which is not included in the selected commit.

## Authority and resolved source conflicts

Use this order when evidence disagrees:

1. The user request and migration task file.
2. Target repository source and unchanged test/binary behavior.
3. Exact source at the frozen pins.
4. For the Rust-contract surface only, `compiler@2a5ebf830c910aa5f7bf53ee4df398915ab12f7a:sdk/sdk/MIGRATION.md` plus CI-maintained examples, with every post-pin section classified rather than applied automatically.
5. `resources/V16-RUST-CONTRACT-REFERENCE-MAP.md` for source routing and `resources/V16-VERSION-TABLE.md` for dependency versions/MSRV.
6. `resources/v16-migration-guide-full.md` for broader client/protocol/VM guidance.
7. The target's existing v0.15 skills. The compiler's whole-project scaffold is authoritative only for the three user-directed packaging changes at the frozen snapshot above; its source and integration files are not implementation precedent.

Resolved planning facts:

- The task's embedded “Migration Delta” is still an empty placeholder. Replace that missing operational context with a source-verification log at the exact tags; do not edit the task file.
- The version table's compiler verification command names the guest SDK tag incorrectly. Use `sdk/v0.14.0-rc.1`, not nonexistent `v0.14.0-rc.1`. Both `sdk/v0.14.0-rc.1` and compiler `v0.10.0-rc.1` resolve locally to `084877ef5feed979d0d732bb0ecbd9855a5022b8`.
- The user-directed whole-project packaging reference was first reviewed at `62c4318...`; after the explicit no-v17 instruction, implementation freezes the earlier functional merge `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`. The build-support/compiler/template/SDK trees are byte-identical between those commits; only a repository-level `.cargo/config.toml` was added afterward. Copy the three packaging adaptations from `2a5ebf830...`; retain their exact rc.1 versions while sourcing both support and guest SDK from the authorized immutable pipeline because the registry support is absent and the registry guest payload fails the required embedded-WIT build.
- The whole-project scaffold is packaging-migrated only. Its two contract source files and every common `integration/` file are byte-identical to this target's v0.15 code; it has zero `#[account_procedure]` and retains the old client/tool pins and APIs. The scaffold source/integration behavior is expressly excluded from the build-oriented template tests; separate integration/CI coverage checks only the two wrapper identities and presence of the support dependency. Never copy or use its `src/` or `integration/` as v0.16 precedent, and never treat its own green build as migration evidence.
- The reference map's broad “31 of 33 files identical” count describes neither comparison precisely. At `BASELINE_COMMIT`, scaffold and target each have 33 paths, 31 common, and 18 byte-identical common files; target-only paths are the two contract lockfiles and scaffold-only paths are the two build scripts. At audited migration commit `6a0c467308e4b1fc60e1702beb9ae5a6747accd8`, the scaffold has 33 paths and the target has 35 non-task paths, all 33 scaffold paths are common, 15 are byte-identical, and the two contract lockfiles are target-only. The load-bearing narrower baseline claim—both contract sources and all integration files were byte-identical stale copies—is verified; the final comparison must use the final counts rather than relabeling the baseline counts.
- The current task explicitly applies the embedded-component-WIT migration in addition to the three settled packaging edits: remove increment-note's obsolete `[package.metadata.miden.dependencies]`/`wit` entry and keep only the ordinary path dependency. The planning checkout and `origin/main` still contain the obsolete table, so the task's statement that this file already has no WIT entry is a stale current-state description; the implementation must remove it as a required v0.16 API adaptation rather than assume it is absent.
- The same current-task correction applies to exactly three stale claims in the cherry-picked `.claude/skills/rust-sdk-patterns/SKILL.md`: the note-metadata sentence, the “two places” cross-component block, and the validation-checklist item. Use `outputs/compiler-port/rust-sdk-patterns.RECONCILED.md` as a section-level reference, re-verify each replacement against the target and immutable compiler snapshot, and do not copy the whole file. Never introduce the nonexistent `project-kind` key; `[lib].kind` remains the project-kind mechanism.
- `origin/next`'s 859-line `sdk/sdk/MIGRATION.md` was read in full by numbered sections. Its `## Unreleased` contains 11 headings; seven were added after frozen tag `sdk/v0.14.0-rc.1`. Use them as an audit inventory, not blanket edit instructions. The user's three packaging changes are an explicit exception to that pin rule; all other post-pin changes remain out unless frozen source/build independently requires them.
- Compatibility must be proven, not inferred from matching version strings: the required build helper invokes `cargo miden ... --stop-after=dependencies`, while tagged `cargo-miden v0.10.0-rc.1` lacks that checkpoint alias and successful partial-build handling. The explicit human decision therefore selects the immutable post-tag pipeline commit for both the isolated binaries and Git-revision build-support dependency. Phase 1 must run that exact source-aligned pair through a temporary plain-Cargo capability probe before any target edit. The compiler is intentionally outside the host Cargo graph: its exact protocol rc.4 dependency cannot coexist with the host graph's semver-compatible rc.6 package, so integration launches the isolated executable across a process/artifact boundary.
- The repository owns no `.masm` source files. MASM module-tree verification is an explicit empty-inventory result, not an omitted gate.
- `miden-client v0.16.0-rc.2:crates/rust-client/src/rpc/endpoint.rs` defines `Endpoint::devnet()` as `https://rpc.devnet.miden.io` and maps it to `NetworkId::Devnet`; this is the exact approved temporary endpoint mechanism.
- `miden-node v0.16.0-rc.1` resolves to commit `7999131c0c9b459322a5cc1d0a9b2b9976ea6de0`. Its root `Cargo.toml` declares workspace version `0.16.0-rc.1` and exact `=0.16.0-rc.4` pins for `miden-protocol`, `miden-standards`, `miden-tx`, `miden-tx-batch`, and `miden-block-prover`. Its RPC status handler returns `env!("CARGO_PKG_VERSION")`, so exact string equality is a valid runtime acceptance gate.
- A deployed node's self-reported package version is not cryptographic proof of its source commit. The runtime/source link is an explicit inference: the official DevNet endpoint reports exact package version `0.16.0-rc.1`, while the official release tag independently supplies the rc.4 dependency evidence. Report it as an inference, not attestation.

## Current-state inventory

- Git: clean before this planning file, on `chore/sync-skills-v15` at `80394cd`, tracking `origin/chore/sync-skills-v15`.
- Decided implementation base: after a fresh fetch, create `kbg/chore/v16-migration` from `origin/main`, then cherry-pick source commit `80394cd` as its own unsquashed skills commit. Current local evidence matches the task: `origin/main=56380d338950d8ca87c7d1bfbae7969c54684ab3`, merge-base `53be3f148dce715dd1bf03ccfb81246e31eb6f17`; main-only changes touch only three lockfiles, while `80394cd` touches only six skills. Re-prove after fetch and stop if topology/overlap changed.
- Open-PR intent to absorb without interacting with GitHub: #55 becomes direct `miden-protocol = "0.16.0-rc.6"`; #56 becomes the v0.16 client pin in `miden-client-cli`; #58's stale README `config.rs` tree entry stays removed. Report all three as superseded by the migration.
- Contracts: `contracts/counter-account` and `contracts/increment-note`.
- Test: `integration/tests/counter_test.rs::counter_test`.
- Binary: `integration/src/bin/increment_count.rs`; it must be run from `integration/` because it uses `../...` paths.
- Locks: root `Cargo.lock` plus one lockfile per contract.
- Installed baseline tools: `cargo-miden 0.9.0`, `midenc 0.6.0`.
- Rust toolchain: `nightly-2026-04-30` / Rust 1.97 with `wasm32-wasip2`; no toolchain-file change is currently expected.
- Ignored state: `store.sqlite3` is a pre-v0.16 SQLite database (`user_version = 1`); `keystore/` contains an existing key index and key. Never include key material in output logs. The authorized runtime will add a new DevNet key to this existing keystore through the application; no manual keystore operation is allowed.
- The installed `miden` wrapper is not a usable fallback, while the current build hook prefers it merely because the command exists.

## Expected file-level migration

| File | Minimal planned adaptation |
| --- | --- |
| `contracts/counter-account/Cargo.toml` | Set guest `miden` version `=0.14.0-rc.1` and source it from authorized immutable Git revision `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`; add the settled build-support dependency at the same revision. The guest source substitution is required because the published same-version payload lacks the pipeline's working embedded-WIT behavior. |
| `contracts/increment-note/Cargo.toml` | Copy the same exact guest/build-support packaging pins. |
| `contracts/counter-account/build.rs` | Add the exact three-line upstream wrapper calling `miden_sdk_build_script_support::prepare_package_cache();`. |
| `contracts/increment-note/build.rs` | Add the identical exact three-line wrapper. |
| Both contract lockfiles | Resolve the guest/compiler line independently; do not copy the compiler template locks. Match the frozen compiler closure exactly: `miden-protocol` and `miden-protocol-build-utils` `0.16.0-rc.4`, guest/build support from exact Git revision `2a5ebf830...`, and the complete VM family `0.29.1`. |
| Both `miden-project.toml` files | Copy the settled `[lib] path = "src/lib.rs"` line; preserve kind, namespace, ordinary dependencies, and supported account types. In increment-note, also remove the obsolete generated-WIT comment, `[package.metadata.miden.dependencies]` table, and `wit` entry exactly as required by the current task; retain the plain `counter-account` path dependency. Never add `project-kind`. |
| `contracts/counter-account/src/lib.rs` | Add `#[account_procedure]` to `get_count` and `increment_count` on the `#[component]` trait only, modelled on `compiler@2a5ebf830c910aa5f7bf53ee4df398915ab12f7a:examples/counter-contract/src/lib.rs:24,27`. Do not alter either implementation. |
| `contracts/increment-note/src/lib.rs` | No expected source change: `Wallet` does not collide with generated `CounterContract`, both are same-module, and the note has no post-pin `get_entrypoint_root`/felt-representation conflict. Change only if frozen source/compiler output proves otherwise. |
| `integration/Cargo.toml` | Remove the in-process `cargo-miden` library dependency so the compiler line remains outside the host graph; pin direct `miden-protocol 0.16.0-rc.6` (PR #55 intent), client/store `0.16.0-rc.2`, standards/testing `0.16.0-rc.6`, `miden-mast-package 0.29.1`, and compatible direct `rand 0.10`; leave Tokio/anyhow unchanged unless resolution proves necessary. |
| Root `Cargo.lock` | Regenerate only from exact host pins; require no `cargo-miden`/compiler packages, preserve the rc.6 host line, and remove stale `miden-tx-batch-prover`. |
| `integration/src/helpers.rs` | Replace in-process `cargo_miden::run`/`CommandOutput` with an invocation of the exact isolated compiler binary, using the same profile/manifest arguments and parsing the pinned CLI's official `Compiled <path>` report before unchanged package deserialization; use `rand::Rng`; remove client debug mode; use verification-wrapped `.grpc_client(&endpoint, Some(timeout_ms))`; install `NoAuth` and `AuthSingleSig` via `.with_component`; wrap the Falcon-512 commitment in `Approver`; make the explicitly authorized one-line temporary `Endpoint::testnet()` -> `Endpoint::devnet()` change; remove only the stale “In protocol v0.15” qualifier from the still-valid account-type/storage-visibility comment; preserve account type, add-account/keystore order, paths, and runtime output. |
| `integration/tests/counter_test.rs` | Replace removed `build_tx_context` with `build_transaction(...).authenticated_input_notes([id]).build()`; wrap the map lookup key in `StorageMapKey`; preserve every assertion and remaining step. |
| `integration/src/bin/increment_count.rs` | No expected source change. It will use the approved DevNet endpoint through the helper. Preserve note builders and both request flows. Add fee arguments/funding only with separate explicit approval because that changes behavior. |
| `.claude/hooks/build-contracts.sh` | Before any contract-source edit, derive the isolated binary path directly from the same Cargo-home-root convention used in Phase 1, require version `0.10.0-rc.1`, and never consult ambient `PATH` or the unusable `miden` wrapper. Preserve the hook JSON protocol, trigger, release build, captured tail, and nonzero failure propagation. |
| `README.md` | Replace v0.16 contract-toolchain provisioning through midenup with exact isolated-root checks/direct builds, and retain PR #58's removal of the nonexistent `config.rs` tree entry. Avoid unrelated prose cleanup. |
| `CLAUDE.md` | Add only source-verified v0.16 contract/test/build guidance required to keep examples accurate. |
| `.claude/skills/*/SKILL.md` | Port stale v0.15 pins/APIs and current-binary Testnet descriptions, add the relevant v0.16/temporary-DevNet rules, and preserve the richer `80394cd` skill content. In `rust-sdk-patterns`, fix exactly the three obsolete generated-WIT claims identified by the task and preserve all unrelated richer content. Audit exactly seven skills; do not overwrite them from the compiler scaffold or reconciled output. |
| `.claude/settings.json`, `.gitignore`, `rust-toolchain.toml`, `.cursorrules`, CI, root `Cargo.toml` | Verify but do not edit unless a pinned build or runtime failure demonstrates a migration requirement. The hook itself derives the persistent isolated tool path, so no settings/PATH injection is planned. Obsolete ignore entries alone are not migration work. |

Known exact host patterns to verify immediately before editing:

```rust
// Account auth is now an ordinary component.
.with_component(NoAuth)

.with_component(AuthSingleSig::new(Approver::new(
    key_pair.public_key().to_commitment(),
    AuthSchemeId::Falcon512Poseidon2,
)))
```

```rust
// MockChain transaction migration; preserve the same one-note input set.
let tx_context = mock_chain
    .build_transaction(counter_account.clone())
    .authenticated_input_notes([counter_note.id()])
    .build()?;
```

```rust
// Account storage now takes the typed map key.
.get_map_item(&counter_storage_slot, StorageMapKey::new(COUNTER_STORAGE_KEY))
```

## Product-only search contract

Every baseline, documentation, and final stale-reference search must use the exact product roots below. This reaches hidden `.claude/**` and `.github/**` content without traversing planning/evidence files or unrelated repository state:

```sh
PRODUCT_PATHS=(
  Cargo.toml rust-toolchain.toml README.md CLAUDE.md LICENSE
  .gitignore .cursorrules .github .claude contracts integration
)
PRODUCT_EXCLUDES=(
  --glob '!.git/**'
  --glob '!**/Cargo.lock'
  --glob '!**/target/**'
  --glob '!**/store.sqlite3'
  --glob '!**/*.sqlite'
  --glob '!**/*.sqlite3'
  --glob '!**/keystore/**'
  --glob '!**/*.bak'
  --glob '!**/*.backup'
  --glob '!**/*.orig'
  --glob '!**/*~'
  --glob '!**/.tmp/**'
  --glob '!**/tmp/**'
)
OLD_PIN_PATTERN='\bv?0\.9(?:\.[0-9]+)?\b|\bv?0\.23(?:\.[0-9]+)?\b|\bmiden\b.{0,80}0\.13(?:\.0)?|cargo-miden.{0,80}0\.9(?:\.0)?|midenc.{0,80}0\.6(?:\.0)?|miden-client.{0,80}0\.(?:14|15)(?:\.[0-9]+)?|miden-(?:client-sqlite-store|standards|testing).{0,80}0\.15(?:\.[0-9]+)?|miden-mast-package.{0,80}0\.23(?:\.[0-9]+)?|\brand\b.{0,80}0\.9(?:\.[0-9]+)?'
MIGRATION_INVENTORY_PATTERN="v?0\\.15(?:\\.[0-9]+)?|\\bv15(?:[-_/][[:alnum:].-]+)?\\b|${OLD_PIN_PATTERN}|miden-tx-batch-prover|with_auth_component|in_debug_mode|build_tx_context|AssetVaultKey|AssetId|AssetClass|AccountDelta|AccountPatch|\\b(?:account_delta|account_patch|apply_delta|apply_patch)\\s*\\(|\\.masl|\\bLibrary\\b|\\bKernelLibrary\\b|link_[A-Za-z0-9_]*_library|[A-Za-z0-9_]*_from_dir|Endpoint::testnet|https://rpc\\.testnet\\.miden\\.io|\\b[Tt]estnet\\b"

OLD_PIN_SAMPLES=$(printf '%s\n' v0.9.0 0.9.2 v0.23 0.23.1)
OLD_PIN_SELF_CHECK=$(printf '%s\n' "$OLD_PIN_SAMPLES" | rg -x "$OLD_PIN_PATTERN")
test "$OLD_PIN_SELF_CHECK" = "$OLD_PIN_SAMPLES"
if printf '%s\n' 0.10.0-rc.1 0.29.1 | rg -q "$OLD_PIN_PATTERN"; then
  exit 1
fi
```

The exact before/after inventory command is:

```sh
rg --hidden -n "$MIGRATION_INVENTORY_PATTERN" \
  "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
```

Inventory the hook and exactly seven local skills in the same evidence log:

```sh
EXPECTED_SKILLS=$(printf '%s\n' \
  .claude/skills/local-node-validation/SKILL.md \
  .claude/skills/miden-client-cli/SKILL.md \
  .claude/skills/miden-concepts/SKILL.md \
  .claude/skills/rust-sdk-patterns/SKILL.md \
  .claude/skills/rust-sdk-pitfalls/SKILL.md \
  .claude/skills/rust-sdk-source-guide/SKILL.md \
  .claude/skills/rust-sdk-testing-patterns/SKILL.md)
ACTUAL_SKILLS=$(find .claude/skills -mindepth 2 -maxdepth 2 -type f -name SKILL.md | LC_ALL=C sort)
test "$ACTUAL_SKILLS" = "$EXPECTED_SKILLS"
test -f .claude/hooks/build-contracts.sh
CONTROL_PATHS=$(printf '%s\n' .claude/hooks/build-contracts.sh "$ACTUAL_SKILLS")
test "$(printf '%s\n' "$CONTROL_PATHS" | wc -l | tr -d ' ')" -eq 8
printf '%s\n' "$CONTROL_PATHS"
```

Use this exact NUL-safe filename inventory for repository-owned MASM source. It traverses only the declared product roots and prunes the directory equivalents of the product exclusions. Do not use `rg --files` with the mixed `PRODUCT_PATHS` array for this purpose: ripgrep emits explicitly named regular-file arguments even when an extension glob does not match them.

```sh
MASM_CAPTURE_DIR=$(mktemp -d "${TMPDIR:-/tmp}/project-template-masm.XXXXXX")
MASM_STDOUT="$MASM_CAPTURE_DIR/masm.paths0"
MASM_STDERR="$MASM_CAPTURE_DIR/masm.stderr"
set +e
find "${PRODUCT_PATHS[@]}" \
  \( -type d \( \
    -name .git -o -name target -o -name keystore -o \
    -path 'integration/stores' -o -path '*/integration/stores' -o \
    -name .tmp -o -name tmp \
  \) -prune \) -o \
  \( -type f -name '*.masm' -print0 \) \
  >"$MASM_STDOUT" 2>"$MASM_STDERR"
MASM_STATUS=$?
set -e
printf 'masm_status=%s\n' "$MASM_STATUS"
test "$MASM_STATUS" -eq 0
test ! -s "$MASM_STDERR"
test ! -s "$MASM_STDOUT"
```

Preserve `MASM_STATUS`, the raw NUL-delimited `MASM_STDOUT`, and complete `MASM_STDERR` in the phase's evidence before evaluating the three assertions. The expected current and final result is status `0` with stdout exactly zero bytes and stderr empty. Any stderr or nonzero status invalidates the inventory. If stdout is nonempty, the final assertion must fail: parse every path with a NUL-safe reader, record it, classify whether and how the file is reachable through its exact module declarations/package build, and stop for plan/source reconciliation before editing or continuing. Do not suppress or override the failed empty-inventory assertion. If implementation adds any new top-level product path, update `PRODUCT_PATHS` explicitly and rerun every inventory; never broaden a gate to `.`.

Run these exact commands before edits, before Phase 7 documentation work, and after all product edits. Preserve complete output and exit status so the baseline and final inventories can be compared. Never use ignored-file bypass flags or `.` as the search root for migration gates. The product-root list deliberately excludes `.git/**`, `tasks/**`, evidence outputs, root stores/keystores, temporary backups, and generated targets; the globs redundantly enforce the safety boundary for matching nested paths. `Cargo.lock` is excluded only from stale-text gates because accepted transitive version skew can legitimately retain older version numbers; lockfiles remain subject to the separate dependency-tree/lock audit.

## Execution plan

### Phase 1 — Toolchain resolution (hard gate before migration edits)

- [x] Create a toolchain evidence log at `outputs/project-template-toolchain-resolution.txt` in the task evidence directory.
- [x] Resolve exact registry releases with `cargo info <crate>@<version>` for:
  - `cargo-miden@0.10.0-rc.1`
  - `midenc@0.10.0-rc.1`
  - `miden@0.14.0-rc.1`
  - `miden-protocol@0.16.0-rc.6`
  - `miden-client@0.16.0-rc.2`
  - `miden-client-sqlite-store@0.16.0-rc.2`
  - `miden-standards@0.16.0-rc.6`
  - `miden-testing@0.16.0-rc.6`
  - `miden-mast-package@0.29.1`
- [x] Verify local and live remote tags/commits without reading moving branches:
  - protocol `v0.16.0-rc.6`
  - miden-client `v0.16.0-rc.2`
  - miden-vm `v0.29.1`
  - compiler `v0.10.0-rc.1`
  - compiler SDK `sdk/v0.14.0-rc.1`
  - compiler templates `templates/v0.32.0-rc.1`
- [x] Record that `miden-sdk-build-script-support@0.14.0-rc.1` is absent from crates.io, then apply the explicit human compiler-source decision. Set both `COMPILER_PIPELINE_COMMIT` and `COMPILER_PACKAGING_COMMIT` to immutable `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`; prove it is the earliest `next` mainline integration containing the support/helper, embedded-WIT, protocol rc.4, and VM 0.29 work. Prove no v17/future line by exact versions/changelog scans and record that its relevant functional trees are byte-identical to the later research snapshot. Do not use a moving branch at installation or dependency resolution time.
- [x] Resolve `miden-node v0.16.0-rc.1` exactly. Record tag commit `7999131c0c9b459322a5cc1d0a9b2b9976ea6de0`, root workspace version `0.16.0-rc.1`, and the root manifest plus lock evidence that `miden-block-prover`, `miden-protocol`, `miden-standards`, `miden-testing`, `miden-tx`, and `miden-tx-batch` are all exactly `=0.16.0-rc.4`. Also record protocol rc.4 commit `bbe8ec8020d8fdaa6ea703bce1557b1380942444`.
- [x] Record from `miden-node v0.16.0-rc.1:crates/rpc/src/server/api/status.rs` that the RPC status version is `env!("CARGO_PKG_VERSION")`, and from the pinned block producer source that a healthy producer reports its package version and `connected`. Define the runtime acceptance contract now: status version `0.16.0-rc.1`, block-producer version `0.16.0-rc.1`, block-producer status `connected`; no semver range or prefix match.
- [x] If a local tag is absent, fetch tags in that reference clone and retry once. If it still does not resolve, stop; never substitute a nearby release. (No tag was absent; live exact-tag resolution matched every local object.)
- [x] Print and record `rustc --version`, `cargo --version`, the active toolchain path, and installed `wasm32-wasip2` target. Confirm the highest MSRV, Rust 1.97, is met.
- [x] Record the existing v0.15 `cargo-miden` executable path and its `0.9.0` version before installing anything.
- [x] Preserve the v0.15 baseline tool by resolving one deterministic isolated root. Use this exact convention in both installation and the hook; record the expanded absolute path (expected here: `/Users/philipp/.cargo/miden-v16-0.10.0-rc.1`):

  ```sh
  MIDEN_CARGO_HOME="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"
  MIDEN_V16_TOOL_ROOT="$MIDEN_CARGO_HOME/miden-v16-0.10.0-rc.1"
  COMPILER_PIPELINE_COMMIT=2a5ebf830c910aa5f7bf53ee4df398915ab12f7a
  cargo install cargo-miden --git https://github.com/0xMiden/compiler \
    --rev "$COMPILER_PIPELINE_COMMIT" --locked --root "$MIDEN_V16_TOOL_ROOT"
  cargo install midenc --git https://github.com/0xMiden/compiler \
    --rev "$COMPILER_PIPELINE_COMMIT" --locked --root "$MIDEN_V16_TOOL_ROOT"
  ```

- [x] Print and verify the isolated binaries directly: `"$MIDEN_V16_TOOL_ROOT/bin/cargo-miden" miden --version` must equal `cargo-miden 0.10.0-rc.1`, and `"$MIDEN_V16_TOOL_ROOT/bin/midenc" --version` must report `0.10.0-rc.1`. Prove the preserved ambient `/Users/philipp/.cargo/bin/cargo-miden` remains `0.9.0`; do not replace it.
- [x] Do not use an unpinned registry install, moving Git branch, local path install, later `next` commit, or `midenup install 0.16`.
- [x] Treat the build-support compatibility question as a **hard pre-edit capability gate**, because a version-string match cannot distinguish the tagged compiler bits from later `origin/next` bits that retain the same version. Record from exact source that the resolved support helper invokes `--stop-after=dependencies`, tagged `v0.10.0-rc.1` lacks that checkpoint alias, and tagged `BuildCommand::exec` does not accept deliberate `CompilerStopped` as success.
- [x] Exercise the actually installed isolated binary and exact Git-revision support crate without changing the target repository:
  1. Set `COMPILER_PACKAGING_COMMIT` to the exact immutable value above; derive `CARGO_MIDEN_BIN="$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"`; require it to be an absolute executable with exact version output.
  2. Create a private `mktemp -d` probe root outside both repositories. Materialize only `extra/templates/project/contracts/counter-account` from `COMPILER_PACKAGING_COMMIT` into that root using a read-only `git archive` of the compiler object; never edit/run Cargo in the compiler checkout.
  3. In the private probe only, replace the unpublished registry build-dependency string with `{ git = "https://github.com/0xMiden/compiler", rev = "2a5ebf830c910aa5f7bf53ee4df398915ab12f7a" }`. Leave `MIDENC_PACKAGE_CACHE` unset, set `CARGO_MIDEN` to the absolute isolated binary, and set `CARGO_TARGET_DIR` to a target directory inside the private probe root. Run `cargo metadata --no-deps --format-version 1` and `cargo check --manifest-path <probe>/Cargo.toml --release -vv`, preserving stdout, stderr, status, resolved lock commit/source, nested command line, and cache paths.
  4. Required result: status `0`; the helper launches the exact isolated binary; no ambient `0.9.x` tool is used; its nested dependency-staging build succeeds; a content-addressed `OUT_DIR/**/miden-packages` generation is published; and no `.staging-*` remains. A parse error for `dependencies`, `CompilerStopped`/unreachable error, missing support crate, stale-cache fallback, or nonzero exit blocks the migration before any hook/product edit.
  5. Do not set `MIDENC_PACKAGE_CACHE` manually, change the revision, or use a branch name to bypass this test. If the exact pair fails, report the incompatibility and stop; do not select a later compiler commit.
- [x] Record the final selected lines: the explicitly authorized unreleased v16 compiler pipeline and guest SDK use exact source `2a5ebf830...`, guest version `0.14.0-rc.1`, protocol and protocol build-utils rc.4, and the compiler lock's VM `0.29.1` family; host integration uses protocol rc.6 and MAST package 0.29.1; exact DevNet node `0.16.0-rc.1` uses protocol rc.4. Do not describe or accept this as merely a generic “v0.16 line.”

Exit only when every executable version is installed and printed, every registry/tag pin plus exact Git source revision and node/protocol pair is proven, the v0.15 tool remains callable for the baseline, and the temporary plain-Cargo capability probe passes with the exact source-aligned isolated `cargo-miden 0.10.0-rc.1`. A green version check without source provenance and a green capability probe is insufficient.

At the start of every later phase or new shell, rederive `MIDEN_CARGO_HOME` and `MIDEN_V16_TOOL_ROOT` with the exact Phase 1 assignments, require `test -x "$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"`, and recheck exact version output before any v0.16 contract build. An unset/stale variable or bare `cargo miden` is not an accepted v0.16 tool selection.

### Phase 2 — Git preflight and branch selection

- [x] Record `git status --short --branch`, `git log --oneline -20`, `git branch -a`, remotes, and `git diff --stat` before any implementation edit.
- [x] Treat `tasks/todo.md` plus the AGENTS-required correction log `tasks/lessons.md` as the only expected planning artifacts. No unrelated worktree change was present.
- [x] After Phase 1 is completely green, run exactly `git fetch --all --prune --tags`. The command was run; Git safely refused to clobber the pre-existing local `v0.9`, `v0.10`, and `v0.11` tags. No tag was deleted or force-updated. A direct live `ls-remote origin refs/heads/main` then proved local `origin/main` exactly matches the live remote at `56380d338950d8ca87c7d1bfbae7969c54684ab3`.
- [x] Re-prove the decided topology after fetch before mutating Git:
  - `80394cd` exists and is not already reachable from `origin/main`;
  - the merge-base and main-only/skills-only path sets remain disjoint;
  - main-only paths are the three lockfiles and skills-only paths are exactly the six named skill files;
  - local `kbg/chore/v16-migration` does not exist, so the required `checkout -b` cannot overwrite prior work;
  - commit signing is configured and usable before the cherry-pick creates a new commit. Do not change signing configuration or create an unsigned commit merely to advance.
- [x] If any topology/path/signing assertion differs, or the cherry-pick would be empty, stop for a new base decision. All assertions matched; no prohibited base/history action was taken.
- [x] With the assertions green, execute the task's exact base sequence:

  ```sh
  git checkout -b kbg/chore/v16-migration origin/main
  git cherry-pick 80394cd
  git log --oneline -5
  ```

  A conflict is a hard stop. Never amend, squash, or fold the cherry-picked skills change into the migration; preserve its original author and separate intent.
- [x] Set `MAIN_BASE_COMMIT=56380d338950d8ca87c7d1bfbae7969c54684ab3`, `SKILLS_BASE_COMMIT=1d47c165550ffe3757fd935b49256bf0370f3ad1`, and `BASELINE_COMMIT=$SKILLS_BASE_COMMIT`. The new signed commit's diff against `MAIN_BASE_COMMIT` is exactly the six skills and its patch/source is `80394cd`; all migration comparisons start at `BASELINE_COMMIT`.

No manifest, source, lockfile, doc, hook, store, or reference-repository changes occur in this phase.

### Phase 3 — Untouched v0.15 baseline capture

- [x] Create these external evidence files without changing repository source:
  - `outputs/project-template-baseline-inventory.txt`
  - `outputs/project-template-baseline-build-test.log`
  - `outputs/project-template-baseline-binary.log`
- [x] Record all manifests, contracts, tests, binaries, source-owned `.masm` files, tool versions, and the baseline commit.
- [x] Record the packaging/metadata baseline explicitly: neither contract has `build.rs`; both contract manifests have guest `miden = "0.13"` and no build-support dependency; both project manifests lack `[lib].path`; increment-note still has the obsolete generated-WIT comment/table/key that the current task requires removing. Record the exact three stale generated-WIT claims in the cherry-picked `rust-sdk-patterns` skill as the before-side of their targeted correction.
- [x] Run and preserve the exact focused WIT/project-kind/stale-skill scans later specified in Phase 9 Gate 8. Baseline results were the required obsolete table/key, exactly three skill claims, and `project-kind` status `1` with empty output.
- [x] Run the exact `OLD_PIN_PATTERN` self-check, `MIGRATION_INVENTORY_PATTERN` command, hook/seven-skill inventory, and MASM filename inventory from the product-only search contract. The union matched the expected stale baseline; control inventory was exactly eight paths; MASM status/stdout/stderr were `0`/empty/empty.
- [x] Set `BASELINE_CARGO_MIDEN_BIN` to the exact preserved absolute path recorded in Phase 1, recheck `"$BASELINE_CARGO_MIDEN_BIN" miden --version` equals `cargo-miden 0.9.0`, then build in this order and capture complete output/exit codes:

  ```sh
  cargo build -p integration --release
  "$BASELINE_CARGO_MIDEN_BIN" miden build --manifest-path contracts/counter-account/Cargo.toml --release
  "$BASELINE_CARGO_MIDEN_BIN" miden build --manifest-path contracts/increment-note/Cargo.toml --release
  cargo test -p integration --release -- --list
  cargo test -p integration --release -- --nocapture
  ```

- [x] Record the exact test name and result: `counter_test` passed; one passed, zero failed, zero ignored.
- [x] Inventory every binary with `find`/`rg`; `increment_count` is the only one and takes no arguments.
- [x] Query Testnet read-only with exact `miden-client 0.15.3`: RPC and block producer both reported `0.15.0`, producer `connected`. The first repository-root run failed before account creation because the existing ignored store had mismatched migration hashes; the untouched binary then passed from disposable clean v0.15 store/keystore state, returning both transaction IDs with exit `0`.
- [x] The no-compatible-node fallback was not needed because exact status verification and the disposable-state live baseline succeeded.
- [x] Record that the binary does not verify final storage or sync after consumption; the evidence is explicitly submission-only and this behavior will not be “improved.”

### Phase 4 — Pin-specific source-verification map

- [x] Write `outputs/project-template-source-verifications.md` before editing code.
- [x] For every non-trivial replacement, cite exact tag/immutable commit, file, symbol/signature, and minimal usage. Read release APIs with `git show <tag>:<path>`; never use a moving branch or current reference worktree. The sole `next` exception is the user-directed Rust-contract audit/packaging snapshot already frozen as `COMPILER_PACKAGING_COMMIT`; refer to it by full commit after the one required `git -C compiler show origin/next:<path>` read.
- [x] Record that `V16-RUST-CONTRACT-REFERENCE-MAP.md` was read completely (222 lines), and that a focused sub-agent read all 859 numbered lines of `COMPILER_PACKAGING_COMMIT:sdk/sdk/MIGRATION.md` in ranges 1-180, 181-360, 361-540, 541-720, and 721-859. Preserve the full heading inventory and frozen-tag/post-pin boundary in the source log; do not summarize “Unreleased” as one undifferentiated change set.
- [x] Record immutable compiler-reference evidence before any edit:
  - packaging: both upstream contract manifests have exact guest/build-support rc.1 pins, both `build.rs` files contain only `prepare_package_cache()`, and both project manifests add `[lib].path`;
  - source: `examples/counter-contract/src/lib.rs:24,27` marks exactly the two callable trait methods; `examples/basic-wallet/src/lib.rs` is the CI-maintained account/asset-operation reference;
  - trap: scaffold contract sources and all integration common files hash equal to this v0.15 target, the scaffold contains zero account-procedure markers, and `tests/templates/tests/templates.rs:1-6` expressly excludes the default project scaffold.
- [x] Classify every `## Unreleased` heading as follows, with an exact target symbol scan and frozen-source citation for each row:

  | `MIGRATION.md` section | Project-template disposition |
  | --- | --- |
  | Transaction summaries are six words | Audited N/A: no guest custom authentication component; host `AuthSingleSig` is not this surface. Do not add guest auth code. |
  | Attachment setter aliases are removed | Audited N/A: no attachment setter calls. |
  | `#[note]` reserves `get_entrypoint_root` | Audited N/A: `IncrementNote` declares no conflicting item. |
  | Contract crates gain a `build.rs` | **Applicable user-directed packaging exception:** add the exact support dependency/wrapper only after Phase 1 compatibility passes. |
  | Kernel scalars are typed instead of `Felt` | Frozen-RC-relevant but audited N/A: no listed kernel count/height/nonce/attachment APIs; stored counter remains intentionally `Felt`. |
  | Typed transaction-script arguments | Audited N/A: no tx-script crate or `#[tx_script]` entrypoint. |
  | Mark account procedures | **Applicable source edit:** mark exactly `get_count` and `increment_count` on the component trait. |
  | `#[account(...)]` generates one trait per component | Applicable semantic audit, no extra edit expected: `Wallet` differs from `CounterContract`, the generated trait is same-module/in scope, and both selected methods become callable. |
  | Tx-kernel bindings beta.1 | Frozen-RC-relevant but audited N/A: no renamed/removed guest kernel calls; do not add basic-wallet behavior. |
  | `#[note]` structs implement `ToFeltRepr` | Audited N/A: the fieldless note has no manual impl/custom field. |
  | Component WIT embedded in package | **Applicable by explicit current-task decision:** remove increment-note's obsolete generated-WIT comment/table/key, retain its ordinary path dependency, and correct exactly three stale claims in `rust-sdk-patterns`. Do not generalize this into unrelated metadata cleanup. |

- [x] Also classify the fully read historical `0.13.0 -> 0.13.1` and `0.12.0 -> 0.13.0` sections as already embodied or absent; do not re-migrate the component trait/storage shape, required project manifest, explicit account wrapper, or protocol-v0.15 bindings.
- [x] Run a focused pre-edit source scan over `contracts/**` and `integration/**` for the non-applicable Unreleased surfaces, including `TransactionSummaryConstructionFailed`, attachment setters, `get_entrypoint_root`, typed kernel count/height/nonce/attachment APIs, `#[tx_script]`, renamed/removed tx-kernel asset APIs, and manual note felt-representation impls. Capture zero matches or classify every match; do not turn prose hits in docs/skills into unrelated code churn.
- [x] Carry forward the already proven adaptations:
  - compiler `#[account_procedure]` trait markers and same-module generated interface trait behavior from the CI-maintained counter example;
  - exact user-directed guest/build-support versions, wrapper `build.rs`, and mandatory project target path from the immutable packaging snapshot, with both guest/support sources frozen to the authorized pipeline after the registry guest failed embedded-WIT resolution;
  - embedded component-WIT handling from the same immutable migration snapshot: ordinary path dependency only for this embedded-WIT package, with no leftover `wit` key;
  - basic-wallet account/asset-operation patterns as the guest asset reference, without importing unused behavior into this project;
  - client `.grpc_client`, removed debug mode, and response-verification behavior;
  - protocol account builder/auth/`Approver` APIs;
  - `rand 0.10` trait used by the pinned client RNG;
  - `MockChain::build_transaction` authenticated-note builder;
  - typed `StorageMapKey` lookup;
  - pinned cargo-miden CLI `Compiled <path>` artifact reporting, unchanged package reader, storage initialization, note builder, and transaction request shapes.
- [x] Prove the **authorized interim DevNet** target and fee policy with a read-only, store-free runtime probe before altering either request or opening the v0.16 SQLite store:
  1. Create a temporary Cargo project under a `mktemp -d` directory outside the target repository, with exact `miden-client = { version = "0.16.0-rc.2", features = ["tonic"] }` and Tokio dependencies. Create its files with the normal patch/edit mechanism, not shell redirection.
  2. Compile and run this pinned-source pattern, capturing output in `outputs/project-template-runtime-probe.log`:

     ```rust
     use anyhow::{Context, ensure};
     use miden_client::rpc::{Endpoint, GrpcClient, NodeRpcClient};

     #[tokio::main]
     async fn main() -> anyhow::Result<()> {
         let endpoint = Endpoint::devnet();
         ensure!(endpoint.to_string() == "https://rpc.devnet.miden.io");
         let rpc = GrpcClient::new(&endpoint, 10_000);
         let status = rpc.get_status_unversioned().await?;
         let (latest, _) = rpc.get_block_header_by_number(None, false).await?;
         let fees = latest.fee_parameters();
         let block_producer = status
             .block_producer
             .as_ref()
             .context("status omitted block producer")?;

         ensure!(status.version == "0.16.0-rc.1");
         ensure!(status.genesis_commitment.is_some());
         ensure!(status.chain_tip > 0);
         ensure!(block_producer.version == "0.16.0-rc.1");
         ensure!(block_producer.status == "connected");
         ensure!(fees.verification_base_fee() == 0);

         println!("endpoint={endpoint}");
         println!("configured_network_id={:?}", endpoint.to_network_id());
         println!("node_version={}", status.version);
         println!("node_genesis={:?}", status.genesis_commitment);
         println!("chain_tip={}", status.chain_tip);
         println!("block_producer_version={}", block_producer.version);
         println!("block_producer_status={}", block_producer.status);
         println!("latest_block={}", latest.block_num());
         println!("fee_faucet_id={:?}", fees.fee_faucet_id());
         println!("verification_base_fee={}", fees.verification_base_fee());
         Ok(())
     }
     ```

  3. Add only `anyhow = "1.0"` for executable assertions. Record the temporary probe's resolved dependency tree and exact exit status.
  4. Required runtime result: configured endpoint exactly `https://rpc.devnet.miden.io`; locally configured network classification printed as DevNet; node version exactly `0.16.0-rc.1`; block-producer version exactly `0.16.0-rc.1` with status exactly `connected`; present genesis commitment; nonzero chain tip; retrievable latest header; and `verification_base_fee=0`.
  5. State the evidence boundary precisely: `Endpoint::to_network_id()` and `GrpcClient::get_network_id()` derive identity from the configured URL and are **not** node-reported network identity. Do not claim remote DevNet attestation. The node version is self-reported, and rc.4 is inferred by pairing that exact official package version with the frozen official tag's manifest/lock; the status RPC does not expose protocol version or source commit.
  6. Record the first probe's genesis commitment and require the immediate pre-transaction probe in Phase 8 to return the same value. This is a continuity check between the two observations, not comparison with an independently authoritative DevNet genesis. If remote network identity later becomes a requirement, stop until an authoritative expected genesis commitment is supplied and compared.
  7. The probe is read-only: it creates no client store/account/note/transaction. If any exact result fails, stop before moving the old store, changing the binary, or submitting transactions. Do not accept another RC, a final release, a generic v0.16 status, Testnet, or localhost without a new human decision.
- [x] If another unknown API appears during compilation, stop that edit path and launch a focused read-only pinned-source query. Do not trial-and-error rewrite.

### Phase 5 — Hook guardrail, then exact dependency and metadata migration

#### 5A. Repair the contract-build hook before any `contracts/**/src` edit

- [x] Make `.claude/hooks/build-contracts.sh` the first repository file changed in the migration. Do this before contract manifests/metadata for the safest ordering and, as a hard requirement, before either `contracts/**/src/lib.rs` is edited.
- [x] Remove all `miden`-wrapper and `command -v cargo-miden` detection. The hook must derive exactly one isolated path on every invocation, independent of ambient `PATH`, using `MIDEN_CARGO_HOME="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"`, `MIDEN_V16_TOOL_ROOT="$MIDEN_CARGO_HOME/miden-v16-0.10.0-rc.1"`, and `CARGO_MIDEN_BIN="$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"`. Do not fall back to `/Users/philipp/.cargo/bin/cargo-miden`, `cargo miden`, or `miden`.
- [x] Invoke the resolved absolute binary as `"$CARGO_MIDEN_BIN" miden --version` and require the complete output to equal `cargo-miden 0.10.0-rc.1`. A missing executable, panic, nonzero version command, empty output, `cargo-miden 0.9.0`, or any other version is a hard hook failure.
- [x] Invoke the same already-verified absolute binary as `"$CARGO_MIDEN_BIN" miden build --manifest-path "$CARGO_TOML" --release`; this direct form includes the literal `miden` token required by cargo-miden's CLI and proves the version-checked binary is the build binary.
- [x] For missing/mismatched tools, emit JSON in the existing `hookSpecificOutput.additionalContext` protocol naming the derived absolute binary path, expected version and source revision, plus the exact immutable-Git install command from Phase 1 with the expanded root shown to the pioneer; then exit `2`. Never silently skip a contract edit because the wrong tool is installed.
- [x] Retain stdin JSON parsing, `FILE_PATH` compatibility, project/contract path filtering, release profile, complete build-output capture, last-20-lines failure context, success JSON, and propagation of build failures with exit `2`.
- [x] Exercise the exact command from `.claude/settings.json` under the captured **default environment**, without a `PATH` prefix or shell-local tool-root export; supply only the settings-required `CLAUDE_PROJECT_DIR="$PWD"`. Feed representative non-contract and contract JSON input directly to the hook command. Record that ambient `command -v cargo-miden` still resolves the preserved `0.9.0` binary while the hook logs/uses the derived isolated `.../miden-v16-0.10.0-rc.1/bin/cargo-miden`. Required results: non-contract input exits `0` without building; contract input verifies `0.10.0-rc.1`, attempts the affected contract build, and propagates its result.
- [x] Do not require the pre-migration contract-input build to succeed with the v0.16 compiler against untouched v0.15 manifests/source. At this point a source-incompatibility exit `2` is acceptable only when the evidence proves exact-tool preflight, a real attempted build, and correct exit/output propagation. The Phase 5A guardrail itself is then ready; migrated builds must become green in Phase 5B/6.
- [x] On the first actual `contracts/**/src` edit in Phase 6, capture the automatic PostToolUse invocation from the default hook environment and prove it again used the isolated absolute binary. The active Codex patch runtime does not dispatch Claude Code's `.claude/settings.json` hooks, so the executor invoked the exact settings command immediately after the edit with the exact edited path and no tool-path environment override; it succeeded with the derived isolated binary. `.claude/settings.json` remains unchanged.

Do not begin Phase 5B or any contract-source adaptation unless the hook itself passes these cases.

#### 5B. Exact pins, build-script support, and target metadata

- [x] With the Phase 1 support-helper capability gate green, apply the three packaging adaptations from `COMPILER_PACKAGING_COMMIT` before any contract `src` edit. Copy the changed lines/files verbatim, not the whole scaffold files:
  1. In both contract manifests, retain exact guest version `0.14.0-rc.1` but source it from immutable Git revision `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`, and add exactly one `[build-dependencies]` entry `miden-sdk-build-script-support` at that same Git revision. The upstream registry strings were applied first; the support package was absent, and execution then proved that the published guest payload cannot supply the same-version pipeline's embedded WIT. The existing human decision authorizing the unreleased v16 compiler pipeline therefore applies to both sources; no later commit is allowed.
  2. Add one `build.rs` per contract with exactly:

     ```rust
     fn main() {
         miden_sdk_build_script_support::prepare_package_cache();
     }
     ```

  3. Add exactly `path = "src/lib.rs"` as the first key under each `[lib]` target. Preserve target kind/namespace, ordinary dependencies, and supported types; never add `project-kind`.
- [x] Apply the current task's separate embedded-WIT API adaptation to `contracts/increment-note/miden-project.toml`: delete only the generated-WIT explanatory comment, `[package.metadata.miden.dependencies]` header, and `counter-account = { wit = "../counter-account/target/generated-wit/" }`; preserve the existing `[dependencies]` table and exact `counter-account = { path = "../counter-account" }` entry.
- [x] Verify the packaging and WIT edits structurally before resolving locks: each contract manifest has exactly one matching build-dependency section/key; each project manifest has exactly one `[lib]` and matching path; increment-note has exactly one ordinary counter-account path dependency and no generated-WIT metadata table/key; neither project manifest contains `project-kind`; and each new `build.rs` byte-compares equal to `COMPILER_PACKAGING_COMMIT:extra/templates/project/<same-relative-path>`. Compare only the specifically authorized manifest sections rather than replacing whole files.
- [x] Update `integration/Cargo.toml` to the complete host pin set in the file map. Remove `cargo-miden` from the host graph; it is an exact isolated executable, not a host library. Add direct `miden-protocol = "0.16.0-rc.6"` to absorb PR #55's intent; do not omit it merely because protocol is also transitive. Use complete prerelease strings and leave Tokio/anyhow unchanged unless exact resolution proves a requirement.
- [x] Regenerate each contract lock independently from its manifest.
- [x] After each lock is deliberately resolved, run `cargo metadata --locked --no-deps --format-version 1 --manifest-path <contract>/Cargo.toml`. Require the exact guest/build-support requirements and no unintended manifest change; metadata inspection must not perform an implicit second lock update.
- [x] Regenerate the root lock from the host pins without blanket `cargo update`; use per-package `--precise` updates or normal resolution from edited exact requirements. The pre-resolution attempt with `cargo-miden` in this graph failed exactly as expected: published rc.1 requires protocol `=0.16.0-alpha.4`, the authorized source compiler requires `=0.16.0-rc.4`, and neither can share Cargo's semver-compatible protocol package slot with required host rc.6. The explicit process/artifact boundary is the resolution; do not downgrade or patch either line.
- [x] Inspect `cargo tree -p integration -d` and all three locks:
  - the host graph contains only the frozen client/protocol/VM line and no `cargo-miden`/compiler package; the compiler/build-support line remains in the isolated executable and the two independent contract locks;
  - no direct dependency drifted from the frozen pins;
  - integration resolves its direct `miden-protocol` exactly `0.16.0-rc.6` alongside client/store rc.2 and standards/testing rc.6;
  - both contract locks resolve `miden-sdk-build-script-support` version `0.14.0-rc.1` from exact Git source revision `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`, never a branch or later commit;
  - both contract locks resolve guest `miden` and its compiler SDK crates at version `0.14.0-rc.1` from the same exact Git revision, `miden-protocol` and `miden-protocol-build-utils` exactly rc.4, and every compiler-side VM workspace crate exactly `0.29.1` as recorded by `COMPILER_PIPELINE_COMMIT:Cargo.lock`; no later compatible protocol RC or `0.29.x` patch is accepted merely because a manifest range permits it;
  - `miden-tx-batch-prover` is gone in favor of `miden-tx-batch`;
  - no attempt was made to unify the accepted version lines.
- [x] Repeat the plain-Cargo helper gate on the actual target packaging with an absolute launcher and checkout-private target. Leave inherited cache state unset so the helper must prove dependency staging; never rely on the ambient v0.9 executable or a shared/global target:

  ```sh
  CARGO_MIDEN_BIN="$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"
  PLAIN_CARGO_TARGET="$PWD/target/plain-cargo-v16"
  case "$CARGO_MIDEN_BIN" in /*) ;; *) exit 1 ;; esac
  test -x "$CARGO_MIDEN_BIN"
  test "$("$CARGO_MIDEN_BIN" miden --version)" = 'cargo-miden 0.10.0-rc.1'
  (cd contracts/counter-account && \
    env -u MIDENC_PACKAGE_CACHE CARGO_MIDEN="$CARGO_MIDEN_BIN" \
      CARGO_TARGET_DIR="$PLAIN_CARGO_TARGET" \
      cargo check --manifest-path Cargo.toml --release -vv)
  (cd contracts/increment-note && \
    env -u MIDENC_PACKAGE_CACHE CARGO_MIDEN="$CARGO_MIDEN_BIN" \
      CARGO_TARGET_DIR="$PLAIN_CARGO_TARGET" \
      cargo check --manifest-path Cargo.toml --release -vv)
  ```

  Cargo discovers each existing `.cargo/config.toml` from the process working directory, not from a child `--manifest-path`; running these from the repository root is invalid because it drops `wasm32-wasip2` and `cfg(miden)`. The counter check passed pre-source. The increment-note pre-source check correctly proved dependency staging but then failed because the still-unmarked counter package exposed no callable interface; rerun it after Phase 6A. Require both final checks to publish content-addressed package generations beneath this target, leave no `.staging-*`, and use only the exact isolated launcher. A nested `dependencies` checkpoint/`CompilerStopped` failure, stale fallback, or cross-checkout target path is a hard stop; do not bypass the helper by setting `MIDENC_PACKAGE_CACHE` manually.
- [x] Run a workspace build, then build both contracts by invoking the isolated binary directly—never bare `cargo miden`:

  ```sh
  MIDEN_CARGO_HOME="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"
  MIDEN_V16_TOOL_ROOT="$MIDEN_CARGO_HOME/miden-v16-0.10.0-rc.1"
  CARGO_MIDEN_BIN="$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"
  PLAIN_CARGO_TARGET="$PWD/target/plain-cargo-v16"
  test "$("$CARGO_MIDEN_BIN" miden --version)" = 'cargo-miden 0.10.0-rc.1'
  (cd contracts/counter-account && \
    env -u MIDENC_PACKAGE_CACHE CARGO_MIDEN="$CARGO_MIDEN_BIN" \
      CARGO_TARGET_DIR="$PLAIN_CARGO_TARGET" \
      cargo check --manifest-path Cargo.toml --release -vv)
  (cd contracts/increment-note && \
    env -u MIDENC_PACKAGE_CACHE CARGO_MIDEN="$CARGO_MIDEN_BIN" \
      CARGO_TARGET_DIR="$PLAIN_CARGO_TARGET" \
      cargo check --manifest-path Cargo.toml --release -vv)
  cargo build --workspace --release
  "$CARGO_MIDEN_BIN" miden build --manifest-path contracts/counter-account/Cargo.toml --release
  "$CARGO_MIDEN_BIN" miden build --manifest-path contracts/increment-note/Cargo.toml --release
  ```

  Treat compiler errors as the input to the next minimal source adaptation; do not advance to tests while the affected layer is red.

### Phase 6 — Minimal code migration, highest risk first

#### 6A. Counter account

- [x] Using only `COMPILER_PACKAGING_COMMIT:examples/counter-contract/src/lib.rs:20-28` as the source pattern, add `#[account_procedure]` immediately above both trait method declarations and nowhere else. The scaffold copy is forbidden as source evidence because its identical v0.15 trait omits both markers.
- [x] Build `counter-account` in release mode with `"$MIDEN_V16_TOOL_ROOT/bin/cargo-miden" miden build --manifest-path contracts/counter-account/Cargo.toml --release`; never rely on ambient Cargo subcommand resolution.
- [x] Verify the resulting package exposes both account procedures using package metadata if supported; later note execution is the definitive behavior proof. The frozen CLI has no separate package-interface inspection command; the definitive proof passed when the unchanged increment note compiled against both generated calls and `counter_test` executed them to the preserved count-`1` assertion.

#### 6B. Increment note

- [x] Build `increment-note` after the counter WIT/package exists with `"$MIDEN_V16_TOOL_ROOT/bin/cargo-miden" miden build --manifest-path contracts/increment-note/Cargo.toml --release`; never rely on ambient Cargo subcommand resolution.
- [x] Keep `Wallet`, `#[account(counter_account::CounterContract)]`, note signature, method calls, arithmetic, and `assert_eq` unchanged. The full Migration Guide audit establishes that the generated `CounterContract` trait is same-module/in scope and does not collide with `Wallet`; the note also has no reserved `get_entrypoint_root` or felt-representation conflict. No trait import/alias was needed.
- [x] If the note cannot call both marked procedures without a behavior-changing rewrite, stop and report. The note compiled and the behavior test passed, so this conditional stop did not trigger.

#### 6C. Integration helper

- [x] Replace in-process `cargo_miden::run`/`CommandOutput` in `build_project_in_dir` with the frozen compiler's own source-proven external-process pattern. The implementation derives and verifies the exact binary, sets child `CARGO_MIDEN`, captures failures, accepts exactly one official `Compiled ` report, resolves it against the child working directory, requires a regular file, and retains `Package::read_from_bytes`.
- [x] Apply the recorded runtime decision as a one-line, temporary `Endpoint::testnet()` -> `Endpoint::devnet()` replacement. The pinned constructor was already source-verified as exact `https://rpc.devnet.miden.io`; timeout and configuration behavior are unchanged.
- [x] Change direct `rand` to 0.10 and import `rand::Rng` so `client.rng().fill_bytes` uses the same trait version as the pinned client.
- [x] Replace the manual raw gRPC construction with `.grpc_client(&endpoint, Some(timeout_ms))`, retaining the approved DevNet endpoint and existing timeout while preserving v0.16 response verification.
- [x] Remove only `.in_debug_mode(true.into())`; do not confuse it with the helper's cargo `--debug` build profile.
- [x] Replace `.with_auth_component(NoAuth)` with `.with_component(NoAuth)`.
- [x] Replace the two-argument `AuthSingleSig::new` with the source-proven `AuthSingleSig::new(Approver::new(commitment, AuthSchemeId::Falcon512Poseidon2))`, then pass it through `.with_component`.
- [x] Remove only the stale “In protocol v0.15” qualifier from `AccountCreationConfig::account_type`; retain the accurate statement that `AccountType::Public`/`Private` encodes storage visibility.
- [x] Preserve the client/store/existing-keystore paths, public `AccountType`, storage seed, package reader, printed account ID, `client.add_account` followed by `keystore.add_key`, and all error context. No alternate keystore path or inspection was introduced.
- [x] Run `cargo clean -p integration` once after changing shared helper code, then rebuild integration to avoid stale binaries. The clean reported no stale package artifacts and the release workspace build passed.

#### 6D. Integration test

- [x] Import `StorageMapKey` and replace only the removed MockChain transaction-construction API and typed map-key argument.
- [x] Preserve sender auth, counter initial state, note package and ID, transaction execution, pending transaction insertion, block proof, storage slot/key, assertion text, expected value `1`, and test name.
- [x] Run the single test immediately. `counter_test` passed: one passed, zero failed, zero ignored.
- [x] Diff `integration/tests/counter_test.rs` against `BASELINE_COMMIT` and prove that every changed hunk is an API adaptation and lines containing the final assertion are unchanged. The complete diff contains only the import, transaction-builder replacement, and typed lookup key; the assertion block is byte-identical.

#### 6E. Binary

- [x] First build `increment_count` unchanged against the migrated helper/dependencies. The release build passed and the binary source has an empty diff against `BASELINE_COMMIT`.
- [x] Preserve `.tag(0)`, the two requests, sync points, print labels/order, no arguments, and no final state read. The complete binary-source diff is empty; the only runtime change is inherited from the authorized helper endpoint.
- [x] On a zero-fee v0.16 environment, do not add fee conversion info or funding code. The read-only DevNet probe reported zero verification base fee and no fee/funding code was added; the immediate runtime probe must repeat this result.
- [x] On a nonzero-fee environment, stop before editing: the fresh sender and `NoAuth` counter both have empty vaults. The sender would need fee conversion info plus a funded fee asset; `NoAuth` must be funded in the native fee asset and rejects explicit conversion info. Adding faucet/funding flow or changing auth/endpoint/CLI beyond the recorded DevNet exception is a separate material behavior change requiring new user approval. The immediate DevNet probe reported zero verification base fee, so this conditional stop did not trigger and no fee/funding behavior was added.

### Phase 7 — Documentation and skills after the hook migration

- [x] Before documentation edits, rerun the exact `MIGRATION_INVENTORY_PATTERN` command, exact hook/seven-skill inventory, and exact MASM filename inventory from the product-only search contract. Require `ACTUAL_SKILLS` to equal the seven explicitly named `EXPECTED_SKILLS`, `CONTROL_PATHS` to contain exactly eight paths (one hook plus those seven skills), and MASM status/stdout/stderr to remain `0`/empty/empty. After documentation edits, rerun the same three commands and preserve a before/after comparison. No placeholder pattern or unrestricted ignored-file scan is permitted.
- [x] Update `README.md` only where v0.16 provisioning/commands and current runtime target are stale. State that the v0.16 midenup channel does not provision this contract toolchain, show the exact isolated-root direct RC install/version check, describe `increment_count` as temporarily targeting DevNet—not Testnet—and remove the nonexistent `config.rs` tree entry exactly as intended by PR #58.
- [x] Update `CLAUDE.md` examples and pitfalls to match the working migrated code; include the required per-contract build-support dependency/wrapper and preserve the package-cache/tool provenance caveat. Do not add new architectural advice.
- [x] Document the already-repaired hook's Cargo-home-derived isolated path and exact `cargo-miden 0.10.0-rc.1` requirement where setup guidance describes automatic contract builds. Do not tell pioneers that ambient `PATH` selects the hook compiler.
- [x] Where plain `cargo check`/IDE analysis is documented, state that each contract's `build.rs` calls `prepare_package_cache()`, `CARGO_MIDEN` must select the verified absolute v0.16 binary, and a checkout-private Cargo target avoids cross-checkout cache reuse. Do not suggest manually setting `MIDENC_PACKAGE_CACHE` as a bypass.
- [x] Port the seven local skills in place:
  - `local-node-validation`: v0.16 node/client pairing, sealed inputs, fresh SQLite store, fees, removed debug mode, and descriptions of the current `increment_count` binary as DevNet rather than Testnet;
  - `miden-client-cli`: current project client `0.16.0-rc.2` and v0.16 CLI behavior, superseding PR #56's intermediate 0.15 update;
  - `miden-concepts`: qualify the stale “no gas” claim and reflect fee/auth semantics;
  - `rust-sdk-patterns`: account-procedure markers, target paths, build-support wrapper, generated interface traits, and exactly the three current-task embedded-WIT corrections. Replace the note-metadata claim, “two places” dependency block, and checklist claim with ordinary `[dependencies]`-only guidance plus the accurate embedded-WIT/leftover-key rule. Use the reconciled output section-by-section, never as a whole-file replacement, and do not introduce `project-kind`;
  - `rust-sdk-pitfalls`: current SDK/compiler pins, deterministic `CARGO_MIDEN`/package-cache rules, and relevant v0.16 silent semantic traps;
  - `rust-sdk-source-guide`: corrected repositories/tags, MSRV, and two-version-line workspace;
  - `rust-sdk-testing-patterns`: rc.6 dependencies, `build_transaction`, typed map keys, `AccountPatch` terminology, and correct `apply_patch` versus intentionally relative `account_delta` examples.
- [x] Treat each skill example as a claim: verify against exact source or the now-green target code. Delete/weaken no safety guidance.
- [x] Classify every case-insensitive Testnet match from the exact Phase 9 documentation gate. Any statement that calls the current `increment_count` binary or helper Testnet-backed must be changed to DevNet. Accurate generic CLI choices, source-exploration topics, and the explicitly future Testnet-restoration note may remain only with a row explaining why they do not describe current runtime behavior.
- [x] Leave the existing `*.masl` ignore rule untouched unless a pinned build/runtime failure proves it blocks the migration. Do not add generated package artifacts to Git.
- [x] Do not change CI, `.cursorrules`, settings, root workspace structure, or the Rust toolchain merely to modernize them. Edit only if the required local build/quality gate proves a v0.16 incompatibility.

### Phase 8 — Full verification and real runtime gate

- [x] Create final evidence logs:
  - `outputs/project-template-final-build-test.log`
  - `outputs/project-template-final-binary.log`
  - `outputs/project-template-final-report.md`
- [x] Run formatting checks without bulk reformatting unrelated code.
- [x] Run, with the isolated v0.16 tool selected, and record full output/exit codes:

  ```sh
  MIDEN_CARGO_HOME="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"
  MIDEN_V16_TOOL_ROOT="$MIDEN_CARGO_HOME/miden-v16-0.10.0-rc.1"
  test "$("$MIDEN_V16_TOOL_ROOT/bin/cargo-miden" miden --version)" = 'cargo-miden 0.10.0-rc.1'
  cargo build --workspace --release
  "$MIDEN_V16_TOOL_ROOT/bin/cargo-miden" miden build --manifest-path contracts/counter-account/Cargo.toml --release
  "$MIDEN_V16_TOOL_ROOT/bin/cargo-miden" miden build --manifest-path contracts/increment-note/Cargo.toml --release
  cargo build -p integration --bin increment_count --release
  cargo test -p integration --release -- --list
  cargo test -p integration --release -- --nocapture
  ```

  Require the repeated plain-Cargo checks to exercise the wrapper successfully with the exact launcher, checkout-private target, published cache generations, and zero lingering `.staging-*`. Preserve verbose evidence; the final contract builds do not substitute for this packaging capability.

- [x] Compare test names and results line by line with the baseline. Required result: the same `counter_test` passes with its assertion intact; zero deleted/ignored/weakened tests.
- [x] Run the exact product-root MASM filename inventory from the search contract and append its complete stdout, stderr, and status. Required result for this repo: status `0`, empty stdout, and empty stderr, proving zero repository-owned `.masm` files in the declared product roots; module declaration reachability is therefore N/A. If a file appears, classify its exact module-declaration/package reachability and stop for reconciliation rather than claiming the empty result. Do not claim generated Rust-contract procedures are covered by this check; prove them through package inspection and successful transaction submission.
- [x] Immediately before store replacement or transaction submission, rerun the exact store-free `Endpoint::devnet()` probe from Phase 4 and append its output. Require the exact configured endpoint/classification, node and block-producer `0.16.0-rc.1` equality, block-producer `connected` status, the same genesis commitment observed in Phase 4, chain/header checks, and `verification_base_fee=0`; a prior result is insufficient because runtime state is time-dependent. Describe package/source/protocol correspondence as the recorded inference, not remote attestation.
- [x] Before any v0.16 client opens the active store path, record the exact ignored `store.sqlite3` path and move it to a task-specific private temporary backup so removal is recoverable. Do not migrate or commit it. Do not manually inspect, copy, move, delete, or print the existing keystore; the later application's `keystore.add_key` mutation is the sole authorized keystore write.
- [x] If the immediate runtime probe is not green, stop at the runtime gate. Do not fall back to Testnet/localhost, accept another node RC, or change auth, funding, CLI, output, request arguments, or transaction flow. The probe was green, so this conditional stop did not trigger.
- [x] From `integration/`, run every binary (currently only `increment_count`) against the approved DevNet node. Label the result **`SUBMISSION-ONLY PASS`** only if the process exits `0`, the publish `submit_new_transaction` returns a transaction ID, the existing intervening `sync_state()` succeeds, the consume `submit_new_transaction` returns a transaction ID, and the same six output fields are printed in the same order. This proves successful submission calls only. Because the preserved binary performs no post-consumption sync, transaction-status query, note-state query, or counter storage read, do not claim on-chain commitment/finality, confirmed note consumption, or a runtime-observed counter value. Attribute semantic execution and the count-`1` assertion to the unchanged passing `counter_test`, not to the live binary. The submissions intentionally create authorized public DevNet side effects and may result in irreversible network inclusion; the binary does not observe that inclusion.
- [x] If committed-state evidence is later required, stop for a separately source-verified, separately approved out-of-band observer plan. Do not add polling, sync, status queries, or storage reads to `increment_count`, because that would violate the preserved-behavior invariant. No such evidence was required or added.
- [x] Record, without printing or inspecting secret material, that normal account setup completed `client.add_account` and then application-level `keystore.add_key` against the existing keystore. This persistent DevNet-key insertion is expected and explicitly authorized. Any manual keystore mutation or alternate keystore path is out of scope.
- [x] Confirm a fresh v0.16 DevNet SQLite store was created and synced; never reopen the archived v0.15/Testnet store with v0.16. Record a follow-up that the future return to Testnet must use a fresh Testnet store and must re-run the same exact-version/fee/runtime gates for the upgraded Testnet stack.
- [x] Compare binary behavior with the baseline/static fallback. A build-only result is failure. The final binary report must use the exact `SUBMISSION-ONLY PASS` label and repeat the unobserved-commitment/count limitations beside the returned IDs.

### Phase 9 — Stale-reference, drift, and material-change audit

- [x] Keep `tasks/todo.md` in place. It is planning evidence, not product content, and is excluded by the explicit product roots rather than moved or hidden to manipulate results. Source the exact `PRODUCT_PATHS`/`PRODUCT_EXCLUDES` arrays from the product-only search contract for every command below.
- [x] Gate 1 — stale v0.15 product references:

  ```sh
  rg --hidden -n 'v?0\.15(?:\.[0-9]+)?|\bv15(?:[-_/][[:alnum:].-]+)?\b' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Required result: zero matches (`rg` exit `1`). This includes dotted v0.15 versions and compact forms/branch names such as `v15` and `kbg/chore/v15-migration`. There are no intentional v0.15 exceptions in product manifests, source, docs, hooks, skills, or settings. Older transitive versions inside excluded `Cargo.lock` files are assessed through the lock/dependency audit instead.

- [x] Gate 2 — old direct pins/tool versions, including pre-v0.16 lines that do not contain `0.15`:

  ```sh
  OLD_PIN_SELF_CHECK=$(printf '%s\n' "$OLD_PIN_SAMPLES" | rg -x "$OLD_PIN_PATTERN")
  test "$OLD_PIN_SELF_CHECK" = "$OLD_PIN_SAMPLES"
  if printf '%s\n' 0.10.0-rc.1 0.29.1 | rg -q "$OLD_PIN_PATTERN"; then
    exit 1
  fi
  rg --hidden -n "$OLD_PIN_PATTERN" \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Required result: the positive and negative self-checks pass, then the product search finds zero matches (`rg` exit `1`). The shared pattern deliberately catches standalone `v0.9.x`/`0.9.x` compiler references and standalone old VM `v0.23`/`0.23.x` references even when nearby text does not name cargo-miden, midenc, or MAST. No product exception is permitted for these old direct pins or tool/VM versions. The frozen new pins (`miden 0.14.0-rc.1`, isolated cargo-miden/midenc `0.10.0-rc.1`, direct protocol/standards/testing rc.6, client/store rc.2, MAST package 0.29.1, rand 0.10) must be verified separately. Verify compiler executables by absolute path/version/source provenance and verify host pins by manifest/metadata; require `cargo-miden` absent from integration metadata and the root lock.

- [x] Gate 3 — removed crates and APIs:

  ```sh
  rg --hidden -n \
    'miden-tx-batch-prover|with_auth_component|in_debug_mode|build_tx_context|AssetVaultKey|AuthMethod|AuthSingleSigAcl|Asset::vault_key|`(?:Library|KernelLibrary)`|\bKernelLibrary\b|\bmiden_(?:client::assembly|assembly)::Library\b|\bLibrary::[A-Za-z_][A-Za-z0-9_]*|\b(?:Arc|Box|Option|Result|Vec)<Library(?:[,>])|link_[A-Za-z0-9_]*_library|[A-Za-z0-9_]*_from_dir' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Required result: zero stale API matches (`rg` exit `1`). This deliberately targets `Library` as a Rust/API symbol rather than matching every English use of the word.

  Then inventory every bare `Library` occurrence:

  ```sh
  rg --hidden -n '\bLibrary\b' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Classify every result in `outputs/project-template-semantic-symbol-audit.md`. Generic headings/prose such as “Standard Library” and “Client Library” are intentional exceptions and require no edit. Any removed Rust `Library` API context must be remediated and must also make the narrowed stale-API command fail until fixed. This classification prevents unrelated prose churn.

- [x] Gate 4 — obsolete `.masl` spelling, with one deliberate no-cleanup exception:

  ```sh
  rg --hidden -n '\.masl' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Required result: exactly one match, `.gitignore:9:*.masl`. It is intentional because deleting an obsolete ignore entry alone is out of migration scope. Any other `.masl` match fails the gate. If a required build/runtime change legitimately alters `.gitignore`, update this expected-line evidence rather than hiding the match.

- [x] Gate 5 — asset-identity semantic classification:

  ```sh
  rg --hidden -n '\b(AssetVaultKey|AssetId|AssetClass)\b' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  First classify **every** match from the initial run—including any `AssetVaultKey`—in `outputs/project-template-semantic-symbol-audit.md` with path:line, containing symbol, semantic role (`AssetVaultKey` = removed v0.15 vault identity; v0.16 `AssetId` = per-asset vault identity; `AssetClass` = faucet/class identity), action or justified no-action, and pinned-source citation. Use `COMPILER_PACKAGING_COMMIT:examples/basic-wallet/src/lib.rs` as the CI-maintained guest account/asset-operation pattern and exact frozen protocol source for the `AssetId`/`AssetClass` semantic definitions; neither source permits a blind rename. Remediate any stale occurrence, rerun the exact command, and require zero final `AssetVaultKey` matches. Every surviving `AssetId` and `AssetClass` must remain in the final ledger. Zero unclassified initial or final occurrences are allowed; do not infer correctness from compilation.

- [x] Gate 6 — account-update semantic classification:

  ```sh
  rg --hidden -n '\b(AccountDelta|AccountPatch)\b|\b(?:account_delta|account_patch|apply_delta|apply_patch)\s*\(' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Classify **every** match from the initial run and every match from the final rerun in the same semantic audit with path:line, semantic role, action/no-action, and pinned-source citation. `AccountPatch`/account `apply_patch()` is required for absolute account updates; `AccountDelta`/`account_delta()` is intentional only for the relative `TransactionSummary` surface. Account-update `apply_delta()` examples are stale and must be migrated. Any update-path `AccountDelta`/`apply_delta`, any relative-summary `AccountPatch`, or any unclassified initial/final match fails the gate. Zero total matches is acceptable only if the captured command output proves there are no product occurrences.

- [x] Gate 7 — Testnet-specific documentation and source classification:

  ```sh
  rg --hidden -n 'Endpoint::testnet' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Required result: zero matches (`rg` exit `1`); the current product source and examples must not select Testnet while the authorized interim DevNet behavior is active.

  Then classify Testnet prose/URLs separately:

  ```sh
  rg --hidden -ni 'https://rpc\.testnet\.miden\.io|\btestnet\b' \
    "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
  ```

  Add every match to `outputs/project-template-semantic-symbol-audit.md` with path:line and role. Required result: zero docs/skill statements that describe the **current** helper or `increment_count` binary as Testnet-backed. Accurate generic network choices, source-exploration topics, and the explicit future Testnet-restoration follow-up are intentional exceptions only when classified. Do not erase accurate general Testnet documentation merely to force zero matches.

- [x] Gate 8 — exact Rust-contract packaging and source-reference boundary:
  - Require each contract manifest to contain exactly one `[dependencies]` Git entry for `miden` with version `=0.14.0-rc.1`, URL and full revision equal to the authorized pipeline, plus exactly one `[build-dependencies]` Git entry for `miden-sdk-build-script-support` at that same URL/revision; validate section membership, versions, and resolved lock sources with TOML/lock parsing, not loose text counts.
  - Require each project manifest to contain exactly one `[lib] path = "src/lib.rs"`; require increment-note to retain exactly one `counter-account = { path = "../counter-account" }` under `[dependencies]` and contain zero `[package.metadata.miden.dependencies]` tables and zero `wit` keys. Require zero `project-kind` keys in both project manifests.
  - Capture stdout, stderr, and status for these focused final scans. The first two must return status `1` with empty stdout/stderr; any status `2` is a gate error rather than a pass:

    ```sh
    rg -n '^\[package\.metadata\.miden\.dependencies\]$|^[[:space:]]*[^#].*\bwit[[:space:]]*=' \
      contracts/increment-note/miden-project.toml
    rg --hidden -n '\bproject-kind\b' \
      "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"
    ```

    Then use section-aware TOML parsing to prove the remaining counter-account dependency belongs to `[dependencies]`, its value is exactly `{ path = "../counter-account" }`, and `[lib].kind` remains `note`; a loose text match is not sufficient for section membership.
  - Compare `.claude/skills/rust-sdk-patterns/SKILL.md` against the Phase 3 baseline and `rust-sdk-patterns.RECONCILED.md` by the three named sections. Require the old note-metadata WIT-entry claim, “declare ... in two places” block, and “under both ... (wit)” checklist claim to be absent; require their replacements to say that embedded-WIT dependencies use the ordinary `[dependencies]` path, a leftover `wit` key is an error for this package, and the metadata key remains only an escape hatch for packages without embedded WIT. Require all unrelated skill content to remain preserved and zero `project-kind` matches across product roots.
  - Run this exact literal stale-claim scan. At Phase 3 baseline it must return status `0`, empty stderr, and exactly three matching lines; at the final gate it must return status `1` with empty stdout/stderr. Status `2` or any baseline count other than three is an error:

    ```sh
    rg -n -F \
      -e 'cross-component `[package.metadata.miden.dependencies]` WIT entry' \
      -e 'in **two places**' \
      -e 'under both `[dependencies]` (path) and `[package.metadata.miden.dependencies]` (wit)' \
      .claude/skills/rust-sdk-patterns/SKILL.md
    ```

  - Run exact fixed-string positive checks for the following five canonical strings from the three reconciled sections. Require each to occur on exactly one line with status `0` and empty stderr, then inspect the complete three affected sections semantically:

    ```sh
    POSITIVE_WIT_SKILL_PATTERNS=(
      '**Project metadata for notes:** See [increment-note/miden-project.toml](../../../contracts/increment-note/miden-project.toml) for `[lib] kind = "note"`, the `namespace` (`miden:increment-note/miden-increment-note@0.1.0`), and the path dependency on the called component (`counter-account = { path = "../counter-account" }`).'
      'declare the component under `[dependencies]` in `miden-project.toml`: `counter-account = { path = "../counter-account" }`.'
      'WIT is embedded in its compiled package, so no `[package.metadata.miden.dependencies]` entry is needed.'
      'it survives only as an escape hatch for dependency packages that do not embed WIT.'
      '- [ ] Cross-component deps declared under `[dependencies]` in `miden-project.toml` (no `wit` key: WIT is embedded in the compiled package)'
    )
    for expected in "${POSITIVE_WIT_SKILL_PATTERNS[@]}"; do
      test "$(rg -n -F -- "$expected" .claude/skills/rust-sdk-patterns/SKILL.md | wc -l | tr -d ' ')" -eq 1
    done
    ```

  - Byte-compare both target `build.rs` files with their same relative paths at `COMPILER_PACKAGING_COMMIT`. Require exactly two target `#[account_procedure]` markers, both on counter trait declarations, and zero in increment-note.
  - Record that the scaffold itself still has zero procedure markers and is excluded from compiler template tests. Do not compare/copy scaffold `src/` or `integration/` into the target; their verified equality is evidence that they are stale, not a desired final state.
  - Re-run the Phase 5B/8 plain-Cargo checks and require the exact isolated launcher/cache contract. A passing `cargo miden build` alone does not make the build-support packaging gate green.

- [x] Gate 9 — decided Git base and superseded open-PR intent:
  - require branch `kbg/chore/v16-migration`, `MAIN_BASE_COMMIT` equal to the fetched `origin/main`, and one separate signed `SKILLS_BASE_COMMIT` whose diff is exactly the six files from source commit `80394cd`;
  - require integration metadata to expose direct `miden-protocol = "0.16.0-rc.6"` (PR #55 intent), not merely a transitive protocol edge;
  - require `.claude/skills/miden-client-cli/SKILL.md` to state the v0.16 client `0.16.0-rc.2` and contain no intermediate `0.14`/`0.15` pin (PR #56 intent);
  - require the stale README `config.rs` tree entry to be absent (PR #58 intent);
  - record in the final report that #55, #56, and #58 are superseded by these v0.16 results. Do not post, close, merge, or otherwise mutate any PR.

- [x] Rerun the exact union command `rg --hidden -n "$MIGRATION_INVENTORY_PATTERN" "${PRODUCT_EXCLUDES[@]}" "${PRODUCT_PATHS[@]}"`, the exact hook/seven-skill inventory, and the exact MASM filename inventory after Gates 1-9. Compare all three with Phase 3 line by line. Require `.claude/hooks/build-contracts.sh` plus exactly the same seven skill paths to be present, require the MASM result to remain status `0`/empty stdout/empty stderr, and explain every surviving union text match through the gate-specific intentional-exception/semantic ledgers.
- [x] Preserve each command, stdout/stderr, and exit status in the final evidence. Do not add exclusions for product source, docs, hooks, skills, or settings to force green output, and do not substitute an unrestricted scan.

- [x] Run `git diff --check`, review `git status`, `git diff --stat BASELINE_COMMIT`, and the complete diff.
- [x] Justify every hunk as one of: exact pin/lock resolution, required API adaptation, required v0.16 build/runtime configuration, or documentation of the migrated behavior. Revert all unrelated churn.
- [x] Write a material-behavior/side-effect delta ledger. Its only source-behavior exception is `integration/src/helpers.rs` changing `Endpoint::testnet()` to `Endpoint::devnet()`. It must also record the authorized consequences: public DevNet account creation, transaction submission and possible irreversible network inclusion, plus application-level insertion of the new DevNet key into the existing keystore. Distinguish observed submission/returned IDs from unobserved commitment. Every other source hunk must be API adaptation only and preserve behavior. Any additional material behavior or persistent side effect stops the migration for a new human decision.
- [x] Compare read-only against both compiler reference boundaries and report them separately:
  - tagged `compiler@v0.10.0-rc.1` is the frozen release/API source and predates the build-support packaging;
  - `COMPILER_PACKAGING_COMMIT:extra/templates/project` supplies only the three explicitly required packaging adaptations; its contract source and integration common files are byte-identical stale v0.15 copies and contain zero procedure markers. Its source/integration behavior is not built by the template-test job; wrapper identity and support-dependency presence are separately integration-test/CI-covered;
  - label 33 paths each / 31 common / 18 byte-identical as the `BASELINE_COMMIT` comparison. For audited migration commit `6a0c467...`, report scaffold 33 paths, target 35 non-task paths, 33 common, 15 byte-identical, and the two target-only contract lockfiles;
  - the embedded-WIT deletion is a separate current-task-required API adaptation beyond the three verbatim packaging changes; report the stale planning-checkout state and the exact manifest/three-skill-claim removals rather than misclassifying it as one of the packaging copies;
  - standalone skill files retain the target's later v0.15 improvements before being ported, no scaffold source/integration file was copied, and no compiler-repository file was edited.
- [x] Confirm no generated artifacts, SQLite database, keystore files, secrets, or temporary backups are staged. The authorized ignored keystore mutation may exist locally but must never be inspected, logged, or staged.

### Phase 10 — User checkpoint, signed local commit, and final report

- [x] Present the green verification evidence and material-change audit, then obtain the required commit approval before committing. Approval received directly from the user on 2026-08-27: “yeah create a local commit. no push yet.”
- [x] Preserve the Phase 2 cherry-picked skills commit as a separate, signed history entry immediately above `MAIN_BASE_COMMIT`; never squash, amend, re-author, or fold it into migration work.
- [x] Create one cohesive signed migration commit containing only `BASELINE_COMMIT..HEAD` migration changes because the port and its documentation must move together. The exact header-only commit is `6a0c467308e4b1fc60e1702beb9ae5a6747accd8` (`chore: migrate project template to Miden v0.16`).
- [x] Use a header-only conventional commit, sign it, and add no body, co-author, or generated attribution. `git verify-commit 6a0c467308e4b1fc60e1702beb9ae5a6747accd8` and `%G?/%GS` report a valid signature by `philipp.keinberger@gmail.com`.
- [x] Verify the migration history shape `origin/main -> separate signed skills commit -> signed migration commit`: `56380d338950d8ca87c7d1bfbae7969c54684ab3 -> 1d47c165550ffe3757fd935b49256bf0370f3ad1 -> 6a0c467308e4b1fc60e1702beb9ae5a6747accd8`. Never amend; the final-audit correction must be a new signed follow-up commit after a separate checkpoint.
- [x] Do not push or open a PR.
- [x] Return the task's required sections in this exact order:
  1. Toolchain Resolution
  2. Baseline
  3. Migration Summary
  4. Source Verifications
  5. Files Changed
  6. Material-Change Audit
  7. Test Comparison
  8. Binary Run Log
  9. Drift vs compiler's `extra/templates/project`
  10. Blockers or Follow-ups
  11. Local Branch and Commit

  In `Blockers or Follow-ups`, state that PRs #55, #56, and #58 are superseded by the completed v0.16 changes without performing any GitHub action. In `Local Branch and Commit`, report `MAIN_BASE_COMMIT`, source skills commit `80394cd`, resulting `SKILLS_BASE_COMMIT`, each migration commit, and signature status separately.

### Phase 11 — Final independent-audit corrections and signed follow-up

- [x] Reconfirm clean starting branch `kbg/chore/v16-migration` at signed audited migration commit `6a0c467308e4b1fc60e1702beb9ae5a6747accd8`; re-read frozen compiler `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a` and protocol rc.6 asset source without fetching or changing any revision.
- [x] Independently downgrade only `miden-protocol-build-utils` rc.6 to rc.4 in each contract lock with the precise Cargo update. The diff for each lock is limited to that package's version and checksum and now matches the frozen compiler lock.
- [x] Correct only the two audited asset-guidance passages: guest `asset.key` is the asset ID / vault identity word, `AssetClass` distinguishes assets issued by one faucet, protocol `Asset` uses `to_id_word()`/`to_value_word()`, and fungible amount remains `value[0]`. Update the semantic ledger with exact protocol rc.6 citations.
- [x] Correct external final report, source-verification record, semantic audit, and build-test evidence. Distinguish baseline 33/31/18 from final pre-correction scaffold 33 / target 35 non-task / 33 common / 15 byte-identical / two target-only lockfiles. Record signed migration commit `6a0c467...` and remove the superseded `.git/index.lock` blocker claim.
- [x] Run each contract's canonical plain-Cargo check from its contract directory with absolute `CARGO_MIDEN`, inherited `MIDENC_PACKAGE_CACHE` unset, `--locked --offline --release -vv`, and a genuinely new checkout-private target directory. Record the full command/status, content-addressed package generations, and zero `.staging-*` residue without deleting an existing target.
- [x] Rerun formatting, workspace release build, both isolated compiler builds, unchanged binary build, test listing, full release test, lock/source parsing, focused stale asset search, unchanged-source checks, and `git diff --check`. Do not run the live binary or touch stores/keystore material.
- [x] Review the complete correction diff and require only both contract locks, two asset-guidance skills, this task record, and the named external evidence files. No application behavior, endpoint, auth, fee, funding, CLI, output, test, or transaction-flow change is present.
- [x] Present the verified correction diff and obtain a new explicit checkpoint before creating a signed follow-up commit. Approval received directly from the user on 2026-08-27: “yes i authrioize”. The separately signed correction exists at `8de62fe27d0873ab560c891c6ab274e35ceab80e`, has parent `6a0c467308e4b1fc60e1702beb9ae5a6747accd8`, and changes exactly the two contract locks, two asset-guidance skills, and this task record. This supersedes the earlier environment-local failed attempt; no active Git blocker remains. No push or GitHub action occurred.

### Phase 12 — Final evidence reconciliation

- [x] Reconfirm a clean starting worktree on branch `kbg/chore/v16-migration` at signed correction commit `8de62fe27d0873ab560c891c6ab274e35ceab80e`; verify its valid signature, signed parent `6a0c467308e4b1fc60e1702beb9ae5a6747accd8`, and exact five-file correction scope.
- [x] Replace the unrelated old corrected plain-Cargo log with two complete fresh runs from the contract directories. Use new checkout-private targets, absolute `CARGO_MIDEN`, inherited `MIDENC_PACKAGE_CACHE` removed, and `--locked --offline --release -vv`; preserve merged stdout/stderr and the real command statuses.
- [x] Require each fresh run to exit zero, compile its primary contract, emit a completion line, resolve protocol/build-utils rc.4, VM 0.29.1, and Git SDK source `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`, publish exactly one `gen-*` directory, and leave zero `.staging-*` directories.
- [x] Reconcile this task record, the external final report, final build/test evidence, and raw plain-Cargo evidence with the signed correction topology and fresh target paths. Historical failures must not appear as the final state.
- [x] Rerun formatting, workspace and contract builds, unchanged binary build, test listing, the full release test, evidence consistency searches, `git diff --check`, and final Git status. All commands pass; `counter_test` remains the only test and reports one passed, zero failed, zero ignored. The live binary was not run and no store/keystore was accessed.
- [x] Present the exact evidence-only tracked diff and verification results for a new explicit commit checkpoint. Approval received directly from the user on 2026-08-27: “yes authriize without push”. The first environment-local attempt could not create `.git/index.lock`, but that historical failure was superseded by signed evidence commit `03068ada8ea2bf45fd32660810a4d46bef3d1d03` with parent `8de62fe27d0873ab560c891c6ab274e35ceab80e`, exact subject `docs: reconcile v0.16 migration evidence`, and tracked scope only `tasks/todo.md`. No push or GitHub action occurred.

## Stop conditions

Stop and report concisely rather than improvising when any of these occurs:

- The executor cannot see the exact direct human-authored `RUNTIME-DEVNET-2026-08-24` authorization in inherited history or an independently human-supplied approval record.
- The post-fetch Git topology/path sets differ from the decided `origin/main` plus disjoint `80394cd` model, `kbg/chore/v16-migration` already exists, commit signing is not ready, or the exact cherry-pick conflicts/is empty. Do not merge PR #52, base on the skills branch, amend, squash, or improvise a replacement history.
- A frozen registry/tag pin does not resolve after one tag refresh.
- The immutable compiler source does not resolve exactly to `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`, its no-v17 version/dependency proof changes, the private Phase 1 plain-Cargo probe fails, or the helper uses an ambient/moving compiler/cache bypass. Do not copy the required packaging changes until this compatibility gate passes.
- Any proposed compiler/support/template source differs from `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`; do not substitute a later `origin/next` state even if its printed versions are unchanged.
- Increment-note cannot build with the explicitly required generated-WIT metadata removal, or the compiler indicates this dependency package does not embed WIT. Stop with exact immutable-source/build evidence; do not restore the stale `wit` key, invent another metadata form, or broaden the change without a reconciled human decision.
- The explicit compiler-process/MAST-artifact boundary fails, `cargo-miden` re-enters the host Cargo graph, or the direct host rc.6 line cannot resolve. Do not downgrade/patch the compiler rc.4 or host rc.6 line and do not link the compiler library into integration.
- The integration crate cannot retain direct `miden-protocol = "0.16.0-rc.6"` while satisfying the frozen client/standards/testing pins; do not silently drop PR #55's decided intent or rely only on a transitive edge.
- The untouched baseline is red, or a migrated build/test layer remains red.
- The same blocker is encountered three times.
- A required API or behavior cannot be proven from exact source.
- A test cannot be migrated without changing its meaning or assertion.
- The binary needs a different endpoint than the approved `Endpoint::devnet()`, a different node tag, auth model, funding workflow, CLI, output, or transaction flow.
- The configured endpoint/locally derived classification differs from exact `Endpoint::devnet()`, DevNet's unversioned status is not exactly node `0.16.0-rc.1`, its block producer is not exactly `0.16.0-rc.1` and `connected`, the two observed genesis commitments differ, its header checks fail, or its verification base fee is nonzero.
- The default-environment hook cannot derive and execute the isolated `cargo-miden 0.10.0-rc.1` binary independently of ambient `PATH`.
- The selected v0.16 runtime charges fees and the existing empty accounts cannot run unchanged.
- The application cannot preserve `client.add_account` followed by authorized insertion of the new DevNet key into the existing keystore without manual key handling or secret disclosure.
- Safe handling of unrelated worktree changes, the pre-v0.16 store, key material, or commit signing is unclear.
- Any step would require editing a reference repo, publishing, pushing, opening/merging a PR, or other remote-side mutation.

## Planning review

- First independent audit result: **BLOCKED**, not passed. It found an unresolved runtime target, missing exact node grounding, unsafe hook sequencing/tool selection, unrestricted scan commands, underspecified stale/semantic gates, and a false self-audit status.
- Second independent audit result: **NEEDS HUMAN**, not passed. The user has now supplied its requested explicit authorization for the endpoint change, live DevNet account/transaction creation, and application-level DevNet-key insertion into the existing keystore. Its technical findings are also revised below.
- Third independent audit result: **NEEDS HUMAN**, not passed. It did not receive the originating conversation's direct approval messages, so it correctly refused to authenticate the plan's self-transcription. The current re-audit must receive those human messages. Its standalone-version, exact-MASM-inventory, and runtime-evidence findings are incorporated below.
- Post-audit Rust-contract reference update: the user required the full `V16-RUST-CONTRACT-REFERENCE-MAP.md`, immutable compiler packaging/examples, and the complete sectioned `sdk/sdk/MIGRATION.md` review before migration. Those reads are complete and reconciled below; this materially revised plan has not yet received a new full audit.
- Earlier current-task Git-base update: the superseded 327-line task decided `origin/main` plus a separate cherry-pick of `80394cd`, and assigned the superseding intent of PRs #55/#56/#58. Phase 2, dependency/docs work, final gates, and commit reporting implement that still-current decision; no Git mutation has occurred during planning.
- Previous complete-plan audit result: **PASS for the superseded 327-line task revision**. It does not attest the current 353-line task's added embedded-WIT correction.
- Current 353-line task update: the task now explicitly requires fixing the three stale generated-WIT claims carried by `80394cd` and forbids `project-kind`. This revision also reconciles the task's stale assertion that the target manifest already lacks the WIT table: the planning checkout still contains it, so removal is an explicit migration edit with exact before/final gates.
- Current independent audit result: **PASS**. Its initial medium finding identified a broken Markdown-sensitive three-claim regex; the corrected literal scan matches exactly the three baseline claims, all five canonical positive checks match the reconciled reference exactly once, and the re-audit returned no findings.
- Current plan result: **SIGNED MIGRATION AND EVIDENCE CHAIN COMPLETE; PR-READINESS P3/FPI CORRECTION VERIFIED**. Signed migration commit `6a0c467308e4b1fc60e1702beb9ae5a6747accd8`, signed dependency/guidance correction `8de62fe27d0873ab560c891c6ab274e35ceab80e`, signed evidence commit `03068ada8ea2bf45fd32660810a4d46bef3d1d03`, and signed final commit-evidence correction `5ef123fe1332b87f82ff68f750be60461b6db284` form a direct-parent chain. The fresh raw plain-Cargo runs, external evidence, builds, tests, searches, and Git checks are green. The signed commit containing this task record is a direct child of `5ef123fe...` and corrects only the pre-PR P3/FPI guidance plus its task/lesson record; its self SHA is recorded externally after creation. Direct branch push and draft-PR creation are authorized; merge remains forbidden.
- First-audit change map:
  1. Runtime contradiction -> `Recorded human runtime decision`, Phase 4 exact DevNet probe, Phase 6C one-line endpoint edit, and Phase 8 real-runtime gate.
  2. Exact node/runtime pin -> `Authority and resolved source conflicts`, Phase 1 node tag/manifest/lock resolution, and exact Phase 4/8 status acceptance.
  3. Hook-before-source sequencing -> Phase 5A, which precedes all contract source edits and rejects every cargo-miden version except `0.10.0-rc.1`.
  4. Product-only searches -> `Product-only search contract`, Phase 3 baseline scan, Phase 7 hidden documentation scan, and Phase 9 gates; no `rg -u*` remains as an executable instruction.
  5. Concrete stale/semantic gates -> Phase 9 Gates 1-9 with patterns, exclusions, expected results, intentional exceptions, exhaustive semantic classification, exact packaging/source-boundary checks, and decided Git/PR-intent checks.
  6. Audit truthfulness -> document header and this section preserve the three earlier non-passing verdicts, scope the prior PASS to the superseded task revision, and record the current complete-plan re-audit only after its sole finding was fixed and the audit returned PASS.
- Second-audit change map:
  1. Explicit persistent-effect authorization -> `Recorded human runtime decision`, Phase 6C, Phase 8, and the Phase 9 material-side-effect ledger.
  2. Ambient-PATH-independent hook -> Phase 1's deterministic Cargo-home-derived isolated root and Phase 5A's direct absolute-binary resolution plus default/automatic-hook tests.
  3. Runtime evidence boundary -> `Recorded human runtime decision`, Phase 4's configured-classification/self-reported-version wording and genesis continuity check, Phase 8, and stop conditions.
  4. Complete before/final inventory -> exact `MIGRATION_INVENTORY_PATTERN` and hook/seven-skill commands, Phase 3, Phase 7, Phase 9 Gate 6, and the final before/after comparison.
  5. Focused stale gates -> Phase 9 Gate 1 covers compact `v15`; Gate 3 narrows removed `Library` API contexts and separately classifies allowed generic prose.
  6. Runtime docs and keystore effects -> Phase 7's Testnet classification/current-DevNet wording, Phase 8's explicit existing-keystore semantics, and Phase 9 Gate 7/side-effect ledger.
- Third-audit change map:
  1. Authentic human authorization -> `Recorded human runtime decision` assigns local record `RUNTIME-DEVNET-2026-08-24`, identifies the exact direct conversation message as the sole authority, records its scope/exclusions, distinguishes it from this plan's transcription, and requires both re-auditor and executor to receive the full approval context or stop.
  2. Standalone old compiler/VM versions -> shared `OLD_PIN_PATTERN`, its positive/negative self-check, `MIGRATION_INVENTORY_PATTERN`, Phase 3, and Phase 9 Gate 2 cover standalone `v0.9.x`/`0.9.x` and `v0.23`/`0.23.x` with zero final exceptions and no baseline/final regex drift.
  3. Reproducible empty MASM proof -> `Product-only search contract` defines the exact NUL-safe `find -print0` inventory, raw stdout/stderr/status capture, expected `0`/zero-byte/empty result, mixed-root `rg --files` prohibition, and appeared-file reachability stop; Phases 3, 7, 8, and 9 invoke it.
  4. Submission-only runtime evidence -> Phase 8 defines exact `SUBMISSION-ONLY PASS` criteria (two returned IDs, intervening sync, preserved output, exit `0`), attributes semantic/count proof to `counter_test`, and forbids claims about inclusion, confirmed consumption, finality, or runtime-observed counter state.
- Rust-contract reference update map:
  1. Full prerequisite reads -> header, `Authority and resolved source conflicts`, and Phase 4 record the 222-line reference-map read plus sub-agent review of all 859 Migration Guide lines by numbered range.
  2. Settled packaging copy -> `Expected file-level migration`, Phase 1 immutable commit/support compatibility gate, Phase 5B exact two-manifest/two-build-script/two-project-path changes, Phase 8 plain-Cargo verification, and Phase 9 Gate 8.
  3. Scaffold source trap -> resolved facts, Phase 4 immutable source/CI evidence, Phase 6 CI-maintained counter example, and Phase 9 drift report forbid copying scaffold `src/`/`integration/` or treating its build as evidence.
  4. Complete `## Unreleased` treatment -> Phase 4 classifies all 11 headings as applicable edit or audited N/A; the current task explicitly makes embedded component WIT applicable while the plan avoids unrelated forward-port changes.
  5. Tool/source incompatibility -> Phase 1 proves the required post-pin helper with the exact tagged binary in a private probe before product edits; Phase 5B/8 repeat it with absolute `CARGO_MIDEN`, an unset inherited cache, and checkout-private target; stop conditions forbid a moving compiler or manual cache bypass.
  6. Correct drift claim -> resolved facts and Phase 9 distinguish baseline 33/31/18 from the audited migration commit's scaffold 33, target 35 non-task, 33 common, 15 byte-identical, and two target-only lockfiles while preserving the narrower verified baseline conclusion that both contract sources and all integration files were identical stale copies.
- Current-task Git-base/PR update map:
  1. Decided base -> `Current-state inventory` and Phase 2 fetch `origin/main`, re-prove disjoint topology, create `kbg/chore/v16-migration`, and cherry-pick `80394cd` with conflict/empty/signing stop gates.
  2. Separate authorship/history -> Phase 2 records `MAIN_BASE_COMMIT`, the distinct signed `SKILLS_BASE_COMMIT`, and `BASELINE_COMMIT`; Phase 10 forbids squashing/amending and reports skills/migration commits separately.
  3. PR #55 intent -> expected integration manifest, Phase 1 registry proof, Phase 5B direct `miden-protocol 0.16.0-rc.6`, Phase 9 Gate 9, and the direct-edge stop condition.
  4. PR #56/#58 intent -> Phase 7 updates the client skill directly to rc.2 and removes README's nonexistent `config.rs`; Gate 9 verifies both.
  5. Remote boundary -> Phase 9/final report marks #55/#56/#58 superseded but performs no PR close/comment/merge or other GitHub mutation.
- Current-task embedded-WIT update map:
  1. Honest current state -> `Authority and resolved source conflicts` and Phase 3 record that the planning checkout still contains the obsolete manifest table despite the task's stale “already absent” wording.
  2. Manifest adaptation -> `Expected file-level migration`, Phase 4 classification, and Phase 5B remove only the generated-WIT comment/table/key while preserving the ordinary path dependency.
  3. Exactly three skill fixes -> expected file map, Phase 3 before inventory, Phase 7 section-level edits, and Phase 9 Gate 8 name and verify the note-metadata sentence, “two places” block, and checklist item without replacing unrelated skill content.
  4. Metadata shape -> Phase 5B and Gate 8 require `[lib].kind`, forbid `project-kind`, and require no leftover `wit` key for the embedded-WIT dependency.
- [x] User approved implementation on 2026-08-25: “Execute the plan and start building finally....”

Implementation result: _Phases 1-10 produced signed migration commit `6a0c467308e4b1fc60e1702beb9ae5a6747accd8` on 2026-08-27 above signed skills baseline `1d47c165550ffe3757fd935b49256bf0370f3ad1` and main base `56380d338950d8ca87c7d1bfbae7969c54684ab3`. The final independent audit then found four narrow evidence/dependency-guidance defects without identifying an application-behavior change: both contract locks leaked `miden-protocol-build-utils` rc.6 instead of frozen compiler rc.4, two asset-guidance passages conflated the asset ID word with `AssetClass` and named removed `to_key_word()`, the canonical plain-Cargo check reused its target and lacked locked/offline reproducibility, and commit/drift evidence remained stale. Signed correction commit `8de62fe27d0873ab560c891c6ab274e35ceab80e` fixes the dependency/guidance findings without changing application behavior. Signed evidence commits `03068ada8ea2bf45fd32660810a4d46bef3d1d03` and `5ef123fe1332b87f82ff68f750be60461b6db284` record the verified Phase 12 and final commit-evidence reconciliation. A final pre-PR re-audit against compiler tag `sdk/v0.14.0-rc.1` identified and verified the narrow P3/FPI correction carried by the signed commit containing this task record. No product behavior changed._

## Implementation review

- Toolchain: isolated `cargo-miden` and `midenc` `0.10.0-rc.1` resolve from exact source `2a5ebf830c910aa5f7bf53ee4df398915ab12f7a`; the preserved ambient `cargo-miden 0.9.0` is never selected by the hook or contract-build gates.
- Packaging and source: both contracts use exact Git-sourced guest/build-support `0.14.0-rc.1`, exact copied `build.rs` wrappers and `[lib] path`, embedded-WIT metadata removal, and exactly two counter `#[account_procedure]` markers. No compiler repository file or stale scaffold source/integration file was copied or edited.
- Host/API migration: the integration graph resolves the frozen client/store rc.2 and protocol/standards/testing rc.6 line without linking `cargo-miden`; every source edit except the explicitly authorized Testnet-to-DevNet endpoint is an API adaptation.
- Tests/builds: formatting, release workspace build, both contract builds, the unchanged binary build, test listing, and full release test all pass. `counter_test` remains the only test, is not ignored, and passes with its final count-`1` assertion byte-identical.
- Live runtime: exact DevNet node and block-producer `0.16.0-rc.1`, connected status, stable genesis, and zero base fee were observed immediately before execution. The binary returned both transaction IDs and exited zero. This is submission-only evidence; it does not prove commitment, finality, confirmed consumption, or a runtime-observed counter value.
- Persistent effects: the old v0.15 store was recoverably moved to `/private/tmp/project-template-v15-store-20260826.sqlite3`; a fresh v0.16 DevNet store was created/synced; application-level insertion of the new DevNet key into the existing ignored keystore occurred as explicitly authorized. No key material was inspected or logged.
- Final audit: every stale-reference/search gate, semantic symbol ledger, exact zero-MASM inventory, reference drift comparison, `git diff --check`, full-diff classification, and no-staged-secret/artifact check passes. Evidence is in the task output directory named at the top of this file.
- Final-audit correction: both contract closures now use compiler-side build utils rc.4; the two asset passages use `AssetId`/`AssetClass` and `to_id_word()` accurately; both fresh locked/offline plain-Cargo checks publish exactly one content-addressed generation and leave zero staging directories; formatting, builds, test listing, the one-test release suite, source-preservation checks, and the complete correction-diff review all pass.
- Checkpoint: signed migration commit `6a0c467308e4b1fc60e1702beb9ae5a6747accd8`, signed dependency/guidance correction `8de62fe27d0873ab560c891c6ab274e35ceab80e`, signed evidence reconciliation `03068ada8ea2bf45fd32660810a4d46bef3d1d03`, and signed final commit-evidence correction `5ef123fe1332b87f82ff68f750be60461b6db284` all have valid signatures by `philipp.keinberger@gmail.com`. The signed commit containing this task record is a separate direct child of `5ef123fe...` and carries the verified P3/FPI skill correction without amending history. The user authorized a direct push and draft PR on `0xMiden/project-template`; no fork, merge, or unrelated GitHub action is authorized.

Lessons/corrections: _The durable correction rules are now recorded in `tasks/lessons.md`, as required by the repository instructions._
