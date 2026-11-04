#!/usr/bin/env bash
# Export machine-readable interface metadata (functions, argument and return
# types, structs, error enums) for each contract to metadata/<contract>.json,
# plus metadata/index.json with WASM hashes and event topics.
source "$(dirname "$0")/lib.sh"
cd "$ROOT_DIR"

[[ "${1:-}" == "--no-build" ]] || cargo build --locked --target wasm32v1-none --release
cargo build --locked --quiet --manifest-path tools/spec-tool/Cargo.toml
TOOL="tools/spec-tool/target/debug/veltrixae-spec-tool"
mkdir -p metadata

for c in factory pair router; do
  log "Exporting metadata/$c.json"
  "$TOOL" json "$WASM_DIR/veltrixae_$c.wasm" > "metadata/$c.json"
done

require_jq
jq -n \
  --slurpfile f metadata/factory.json --slurpfile p metadata/pair.json --slurpfile r metadata/router.json \
  --arg sdk "$(cargo metadata --format-version 1 --locked 2>/dev/null | jq -r '.packages[] | select(.name=="soroban-sdk") | .version' | head -n1)" \
  '{
    project: "VELTRIXAE",
    soroban_sdk: $sdk,
    note: "Event topics keep the upstream Soroswap prefixes for indexer compatibility.",
    contracts: {
      factory: { wasm: "veltrixae_factory.wasm", wasm_sha256: $f[0].wasm_sha256, spec: "factory.json",
        event_topics: [["SoroswapFactory","init"],["SoroswapFactory","new_pair"],["SoroswapFactory","fee_to"],["SoroswapFactory","setter"],["SoroswapFactory","fees"]] },
      pair:    { wasm: "veltrixae_pair.wasm", wasm_sha256: $p[0].wasm_sha256, spec: "pair.json",
        event_topics: [["SoroswapPair","deposit"],["SoroswapPair","swap"],["SoroswapPair","withdraw"],["SoroswapPair","sync"],["SoroswapPair","skim"]] },
      router:  { wasm: "veltrixae_router.wasm", wasm_sha256: $r[0].wasm_sha256, spec: "router.json",
        event_topics: [["SoroswapRouter","init"],["SoroswapRouter","add"],["SoroswapRouter","remove"],["SoroswapRouter","swap"]] }
    }
  }' > metadata/index.json
log "Wrote metadata/index.json"
