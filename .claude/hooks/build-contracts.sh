#!/bin/bash
# Post-edit hook: builds the modified contract if a file in contracts/ was edited.
# Reads JSON input from stdin (Claude Code hook protocol).

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.filePath // empty')

# Only trigger for files inside this project's contracts/ directory
if [[ -z "$FILE_PATH" ]] || [[ "$FILE_PATH" != "$CLAUDE_PROJECT_DIR/contracts/"* ]]; then
  exit 0
fi

# Find the contract directory (parent of src/)
CONTRACT_DIR=$(echo "$FILE_PATH" | sed 's|/src/.*||')
CARGO_TOML="$CONTRACT_DIR/Cargo.toml"

if [[ ! -f "$CARGO_TOML" ]]; then
  exit 0
fi

# Resolve the exact v0.16 compiler independently of ambient PATH.
MIDEN_CARGO_HOME="${CARGO_HOME:-${HOME:?HOME must be set}/.cargo}"
MIDEN_V16_TOOL_ROOT="$MIDEN_CARGO_HOME/miden-v16-0.10.0-rc.1"
CARGO_MIDEN_BIN="$MIDEN_V16_TOOL_ROOT/bin/cargo-miden"
EXPECTED_VERSION="cargo-miden 0.10.0-rc.1"
COMPILER_SOURCE_REVISION="2a5ebf830c910aa5f7bf53ee4df398915ab12f7a"
INSTALL_COMMAND="cargo install cargo-miden --git https://github.com/0xMiden/compiler --rev $COMPILER_SOURCE_REVISION --locked --root $MIDEN_V16_TOOL_ROOT"

if [[ ! -x "$CARGO_MIDEN_BIN" ]]; then
  jq -n --arg ctx "Contract build FAILED: required compiler is not executable at $CARGO_MIDEN_BIN. Expected '$EXPECTED_VERSION' from source revision $COMPILER_SOURCE_REVISION. Install with: $INSTALL_COMMAND" \
    '{"hookSpecificOutput": {"additionalContext": $ctx}}'
  exit 2
fi

VERSION_OUTPUT=$("$CARGO_MIDEN_BIN" miden --version 2>&1)
VERSION_EXIT=$?

if [[ $VERSION_EXIT -ne 0 ]] || [[ "$VERSION_OUTPUT" != "$EXPECTED_VERSION" ]]; then
  jq -n --arg ctx "Contract build FAILED: compiler at $CARGO_MIDEN_BIN reported '$VERSION_OUTPUT' (exit $VERSION_EXIT); expected '$EXPECTED_VERSION' from source revision $COMPILER_SOURCE_REVISION. Install with: $INSTALL_COMMAND" \
    '{"hookSpecificOutput": {"additionalContext": $ctx}}'
  exit 2
fi

# Run build once, capturing output
BUILD_OUTPUT=$("$CARGO_MIDEN_BIN" miden build --manifest-path "$CARGO_TOML" --release 2>&1)
BUILD_EXIT=$?

if [[ $BUILD_EXIT -eq 0 ]]; then
  jq -n --arg ctx "Contract build succeeded with $CARGO_MIDEN_BIN ($EXPECTED_VERSION)" \
    '{"hookSpecificOutput": {"additionalContext": $ctx}}'
  exit 0
else
  TAIL_OUTPUT=$(echo "$BUILD_OUTPUT" | tail -20)
  jq -n --arg ctx "Contract build FAILED using $CARGO_MIDEN_BIN ($EXPECTED_VERSION). Fix compilation errors before continuing."$'\n'"$TAIL_OUTPUT" \
    '{"hookSpecificOutput": {"additionalContext": $ctx}}'
  exit 2
fi
