#!/usr/bin/env bash
# Ensure sibling Buzz checkout exists for path dependency:
#   buzz-sdk = { path = "../buzz/crates/buzz-sdk" }
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUZZ_DIR="$(cd "$ROOT/.." && pwd)/buzz"
# Pin to a known-good Buzz monorepo revision (block/buzz).
BUZZ_GIT_URL="${BUZZ_GIT_URL:-https://github.com/block/buzz.git}"
BUZZ_GIT_REV="${BUZZ_GIT_REV:-7e34bee62cacaa9d8a96c14d5892a471b59a1983}"

if [[ -f "$BUZZ_DIR/crates/buzz-sdk/Cargo.toml" ]]; then
  echo "bootstrap-deps: found buzz-sdk at $BUZZ_DIR/crates/buzz-sdk"
  exit 0
fi

echo "bootstrap-deps: cloning $BUZZ_GIT_URL @ $BUZZ_GIT_REV → $BUZZ_DIR"
mkdir -p "$(dirname "$BUZZ_DIR")"
if [[ -d "$BUZZ_DIR/.git" ]]; then
  git -C "$BUZZ_DIR" fetch --depth 1 origin "$BUZZ_GIT_REV"
  git -C "$BUZZ_DIR" checkout --force "$BUZZ_GIT_REV"
else
  rm -rf "$BUZZ_DIR"
  git clone --filter=blob:none --no-checkout "$BUZZ_GIT_URL" "$BUZZ_DIR"
  git -C "$BUZZ_DIR" checkout --force "$BUZZ_GIT_REV"
fi

if [[ ! -f "$BUZZ_DIR/crates/buzz-sdk/Cargo.toml" ]]; then
  echo "bootstrap-deps: ERROR — buzz-sdk still missing after clone" >&2
  exit 1
fi

echo "bootstrap-deps: ok"
