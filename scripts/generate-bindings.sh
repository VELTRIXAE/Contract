#!/usr/bin/env bash
# Generate TypeScript client bindings for VELTRIXAE Factory, Pair and Router
# into bindings/<contract>/ for veltrixae-frontend and veltrixae-aggregator.
#
# Uses tools/spec-tool (soroban-spec-typescript, the generator behind
# `stellar contract bindings typescript`). Fully offline: no contract IDs or
# network values are baked in; consumers pass them at runtime.
source "$(dirname "$0")/lib.sh"
cd "$ROOT_DIR"

[[ "${1:-}" == "--no-build" ]] || cargo build --target wasm32v1-none --release
cargo build --quiet --manifest-path tools/spec-tool/Cargo.toml
TOOL="tools/spec-tool/target/debug/veltrixae-spec-tool"

for c in factory pair router; do
  out="bindings/$c"
  log "Generating TypeScript bindings: $out"
  "$TOOL" typescript "$WASM_DIR/veltrixae_$c.wasm" "$out" "@veltrixae/$c"
done
log "Done. Build a package with: (cd bindings/router && npm install && npm run build)"
