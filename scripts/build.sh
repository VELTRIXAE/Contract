#!/usr/bin/env bash
# Compile all contracts to WASM and, when the Stellar CLI is available,
# write size-optimized copies next to them.
#
#   ./scripts/build.sh            # build + optimize (if possible)
#   ./scripts/build.sh --no-opt   # build only
source "$(dirname "$0")/lib.sh"

cd "$ROOT_DIR"
log "Building contracts (wasm32v1-none, release)"
cargo build --target wasm32v1-none --release

if [[ "${1:-}" == "--no-opt" ]]; then exit 0; fi

if command -v "$STELLAR" >/dev/null 2>&1; then
  for c in veltrixae_factory veltrixae_pair veltrixae_router; do
    log "Optimizing $c"
    "$STELLAR" contract optimize --wasm "$WASM_DIR/$c.wasm" --wasm-out "$WASM_DIR/$c.optimized.wasm"
  done
else
  warn "Stellar CLI not found; skipping optimization."
fi

log "Artifacts:"
ls -l "$WASM_DIR"/veltrixae_*.wasm >&2
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$WASM_DIR"/veltrixae_*.wasm >&2
fi
