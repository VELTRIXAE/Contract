#!/usr/bin/env bash
# Shared helpers for VELTRIXAE scripts. Source, do not execute.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_DIR="$ROOT_DIR/target/wasm32v1-none/release"
DEPLOYMENTS_DIR="$ROOT_DIR/deployments"
STELLAR="${STELLAR:-stellar}"

# Only these networks are allowed. Mainnet is refused unconditionally.
ALLOWED_NETWORKS=("testnet" "local")

log()  { printf '\033[1;34m[veltrixae]\033[0m %s\n' "$*" >&2; }
warn() { printf '\033[1;33m[warn]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[error]\033[0m %s\n' "$*" >&2; exit 1; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "'$1' not found. $2"
}

require_stellar() {
  require_cmd "$STELLAR" "Install the Stellar CLI: https://developers.stellar.org/docs/tools/cli"
}

require_jq() {
  require_cmd jq "Install jq (https://jqlang.org)."
}

# Refuse anything that is not explicitly allowed, and refuse mainnet loudly.
check_network() {
  local net="$1"
  case "$net" in
    mainnet|public|pubnet|*mainnet*|*public*)
      die "Refusing to use network '$net'. VELTRIXAE scripts never touch mainnet." ;;
  esac
  local ok=0
  for n in "${ALLOWED_NETWORKS[@]}"; do [[ "$n" == "$net" ]] && ok=1; done
  [[ $ok -eq 1 ]] || die "Network '$net' is not allowed. Allowed: ${ALLOWED_NETWORKS[*]}"
}

# Identities are names in the Stellar CLI keystore (`stellar keys generate`).
# Secret keys are never read from, or written to, this repository.
check_identity() {
  local id="$1"
  [[ "$id" =~ ^S[A-Z2-7]{55}$ ]] && die "Pass a key *name* from 'stellar keys', never a raw secret key."
  "$STELLAR" keys address "$id" >/dev/null 2>&1 \
    || die "Unknown identity '$id'. Create one with: stellar keys generate $id --network testnet --fund"
}

deployment_file() { echo "$DEPLOYMENTS_DIR/$1.json"; }

# Extract the last 64-char hex string (a tx hash) from CLI log output.
extract_tx_hash() {
  grep -Eo '\b[0-9a-f]{64}\b' "$1" | tail -n1 || true
}

# Public Testnet connection, independent of mutable CLI aliases.
TESTNET_RPC_URL="https://soroban-testnet.stellar.org"
TESTNET_PASSPHRASE="Test SDF Network ; September 2015"
TESTNET_ARGS=(--rpc-url "$TESTNET_RPC_URL" --network-passphrase "$TESTNET_PASSPHRASE")
require_public_testnet() {
  [[ "$1" == testnet ]] || die "This operation supports public Testnet only."
}
