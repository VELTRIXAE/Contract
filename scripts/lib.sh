#!/usr/bin/env bash
# Shared helpers for VELTRIXAE scripts. Source, do not execute.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_DIR="$ROOT_DIR/target/wasm32v1-none/release"
DEPLOYMENTS_DIR="$ROOT_DIR/deployments"
JOURNAL_DIR="$DEPLOYMENTS_DIR/journal"
STELLAR="${STELLAR:-stellar}"

# Ordered deployment steps. The journal records each step as it succeeds so a
# partial run keeps its public provenance even if a later step fails.
JOURNAL_STEPS=(
  "build"
  "upload:oracle"
  "upload:aggregator"
  "deploy:oracle"
  "deploy:aggregator"
  "init:oracle"
  "init:aggregator"
  "wire:aggregator"
)
JOURNAL_SCHEMA_VERSION=1

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

# ---------------------------------------------------------------------------
# Durable deployment journal
#
# The journal lives under deployments/journal/<network>.json and records only
# public data: step names, contract IDs, artifact sha256 digests and structured
# receipt identifiers (tx hash, operation type, contract/code identity). It
# never contains signing material, CLI-keystore contents or secret env data.
# ---------------------------------------------------------------------------

journal_file() {
  local net="${1:-testnet}"
  echo "$JOURNAL_DIR/$net.json"
}

# Atomic JSON write: temp file in the same directory, fsync, then rename.
# Usage: atomic_write_json <dest> <json-string>
atomic_write_json() {
  local dest="$1" payload="$2" dir tmp
  dir="$(dirname "$dest")"
  mkdir -p "$dir"
  tmp="$(mktemp "$dir/.tmp.XXXXXX")" || die "Cannot create temp file in $dir"
  # shellcheck disable=SC2064
  trap "rm -f '$tmp'" RETURN
  printf '%s\n' "$payload" >"$tmp" || die "Cannot write temp file $tmp"
  # Flush to disk before the rename so a crash cannot leave a torn record.
  if command -v sync >/dev/null 2>&1; then sync "$tmp" 2>/dev/null || sync; fi
  mv -f "$tmp" "$dest" || die "Cannot atomically replace $dest"
  trap - RETURN
}

# Validate a journal document. Rejects malformed or stale records loudly.
# Usage: journal_validate <file>
journal_validate() {
  local file="$1"
  [[ -f "$file" ]] || die "Journal not found: $file"
  jq -e . "$file" >/dev/null 2>&1 || die "Journal is malformed (not valid JSON): $file"
  local schema
  schema="$(jq -r '.schema_version // empty' "$file")"
  [[ -n "$schema" ]] || die "Journal is missing schema_version: $file"
  [[ "$schema" == "$JOURNAL_SCHEMA_VERSION" ]] \
    || die "Journal schema_version '$schema' is stale (expected $JOURNAL_SCHEMA_VERSION): $file"
  jq -e '.network | type == "string"' "$file" >/dev/null 2>&1 \
    || die "Journal is missing a network field: $file"
  jq -e '.steps | type == "object"' "$file" >/dev/null 2>&1 \
    || die "Journal is missing a steps object: $file"
  # Every recorded step must carry its public identity fields.
  local bad
  bad="$(jq -r '
    .steps | to_entries[]
    | select(
        (.value.status != "ok")
        or (.value.recorded_at | type != "string")
        or ((.value.artifact_sha256 // "") | test("^[0-9a-f]{64}$") | not)
      )
    | .key' "$file" 2>/dev/null || true)"
  [[ -z "$bad" ]] || die "Journal has malformed step record(s): $bad ($file)"
}

# Structured receipt validation. Replaces the old "last 64-hex string" guess.
# Verifies the receipt's operation type and contract/code identity match the
# expected step, so unrelated operations cannot be recorded.
# Usage: validate_receipt <receipt-json> <expected-op> <expected-identity>
validate_receipt() {
  local receipt="$1" expected_op="$2" expected_id="$3"
  jq -e . >/dev/null 2>&1 <<<"$receipt" || die "Receipt is not valid JSON."
  local op tx
  op="$(jq -r '.operation_type // empty' <<<"$receipt")"
  tx="$(jq -r '.tx_hash // empty' <<<"$receipt")"
  [[ -n "$op" ]] || die "Receipt is missing operation_type."
  [[ "$op" == "$expected_op" ]] \
    || die "Receipt operation_type '$op' does not match expected '$expected_op'."
  [[ "$tx" =~ ^[0-9a-f]{64}$ ]] || die "Receipt tx_hash is not a 64-char hex string."
  local id
  id="$(jq -r '.contract_id // .code_id // empty' <<<"$receipt")"
  [[ -n "$id" ]] || die "Receipt is missing contract_id/code_id identity."
  [[ "$id" == "$expected_id" ]] \
    || die "Receipt identity '$id' does not match expected '$expected_id'."
}

# Extract a structured receipt from CLI log output. The CLI prints a JSON
# receipt block; we require it rather than scanning for arbitrary hex.
# Usage: extract_receipt <logfile>
extract_receipt() {
  local log="$1"
  [[ -f "$log" ]] || die "Log file not found: $log"
  local receipt
  receipt="$(grep -Eo '\{[^{}]*"operation_type"[^{}]*\}' "$log" | tail -n1 || true)"
  [[ -n "$receipt" ]] || die "No structured receipt found in $log."
  printf '%s\n' "$receipt"
}

# Record a successful step in the journal. Refuses to overwrite an existing
# final manifest and only stores public fields.
# Usage: journal_append_step <network> <step> <artifact_sha256> <receipt-json>
journal_append_step() {
  local net="$1" step="$2" sha="$3" receipt="$4"
  local file manifest
  file="$(journal_file "$net")"
  manifest="$(deployment_file "$net")"
  [[ ! -f "$manifest" ]] \
    || die "Final manifest already exists ($manifest); refusing to append to journal."
  [[ "$sha" =~ ^[0-9a-f]{64}$ ]] || die "artifact_sha256 must be 64-char hex."
  jq -e . >/dev/null 2>&1 <<<"$receipt" || die "Receipt is not valid JSON."

  local doc
  if [[ -f "$file" ]]; then
    journal_validate "$file"
    doc="$(cat "$file")"
  else
    doc="$(jq -n --arg net "$net" --argjson v "$JOURNAL_SCHEMA_VERSION" \
      '{schema_version: $v, network: $net, steps: {}}')"
  fi

  # Keep only whitelisted public fields from the receipt.
  local public_receipt
  public_receipt="$(jq -c '{tx_hash, operation_type, contract_id, code_id}' <<<"$receipt")"

  local updated
  updated="$(jq -c \
    --arg step "$step" \
    --arg sha "$sha" \
    --argjson receipt "$public_receipt" \
    --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '.steps[$step] = {status: "ok", recorded_at: $ts, artifact_sha256: $sha, receipt: $receipt}' \
    <<<"$doc")"
  atomic_write_json "$file" "$updated"
  log "Journaled step '$step' -> $file"
}

# Report completed vs missing steps without submitting any transactions.
# Usage: journal_report <network>
journal_report() {
  local net="$1" file
  file="$(journal_file "$net")"
  if [[ -f "$file" ]]; then
    journal_validate "$file"
  fi
  printf 'Deployment journal: %s\n' "$file"
  local step done
  for step in "${JOURNAL_STEPS[@]}"; do
    done=""
    if [[ -f "$file" ]]; then
      done="$(jq -r --arg s "$step" '.steps[$s].status // empty' "$file")"
    fi
    if [[ "$done" == "ok" ]]; then
      printf '  [x] %s\n' "$step"
    else
      printf '  [ ] %s (missing)\n' "$step"
    fi
  done
}

# Public Testnet connection, independent of mutable CLI aliases.
TESTNET_RPC_URL="https://soroban-testnet.stellar.org"
TESTNET_PASSPHRASE="Test SDF Network ; September 2015"
TESTNET_ARGS=(--rpc-url "$TESTNET_RPC_URL" --network-passphrase "$TESTNET_PASSPHRASE")
require_public_testnet() {
  [[ "$1" == testnet ]] || die "This operation supports public Testnet only."
}
