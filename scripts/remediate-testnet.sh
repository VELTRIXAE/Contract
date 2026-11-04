#!/usr/bin/env bash
# Resume initialization of EXISTING recorded contracts. Never deploys or funds.
# Usage: scripts/remediate-testnet.sh [--apply --source KEYSTORE_NAME]
source "$(dirname "$0")/lib.sh"
require_stellar; require_jq
APPLY=0; SOURCE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply) APPLY=1; shift ;;
    --source) SOURCE="${2:?Missing identity}"; shift 2 ;;
    *) die "Usage: $0 [--apply --source KEYSTORE_NAME]" ;;
  esac
done
OUT="$(deployment_file testnet)"
[[ -f "$OUT" ]] || die "No Testnet record. Recover existing IDs and receipts before remediation; do not redeploy for evidence."
jq -e '.network == "testnet"' "$OUT" >/dev/null || die "Wrong manifest network."
TMPD="$(mktemp -d)"; trap 'rm -rf "$TMPD"' EXIT
view() { "$STELLAR" contract invoke "${TESTNET_ARGS[@]}" --send=no "$@"; }
FACTORY="$(jq -er .contracts.factory.id "$OUT")"
ROUTER="$(jq -er .contracts.router.id "$OUT")"
SETTER="$(jq -er .fee_to_setter "$OUT")"
HASH="$(jq -er .contracts.pair.wasm_hash "$OUT")"
# Verify code before simulating any initialization. No automatic replacement.
for c in factory router; do
  id="$(jq -er ".contracts.$c.id" "$OUT")"
  want="$(jq -er ".contracts.$c.wasm_sha256" "$OUT")"
  "$STELLAR" contract fetch "${TESTNET_ARGS[@]}" --id "$id" --out-file "$TMPD/$c.wasm"
  got="$(sha256sum "$TMPD/$c.wasm" | cut -d' ' -f1)"
  [[ "$got" == "$want" ]] || die "$c code mismatch; manual investigation required."
done
NEED_FACTORY=0; NEED_ROUTER=0
if got="$(view --id "$FACTORY" -- fee_to_setter)"; then
  [[ "$got" == "\"$SETTER\"" ]] || die "Factory initialized to unexpected setter."
else
  [[ "$(jq -r '.deployment_model // "legacy"' "$OUT")" != atomic-constructor-v1 ]] || die "Constructor deployment has unreadable state; investigate, do not initialize."
  # Unknown errors (archival, transport, wrong code) must not imply uninitialized.
  view --id "$FACTORY" -- initialize --setter "$SETTER" --pair_wasm_hash "$HASH" >/dev/null
  NEED_FACTORY=1
fi
if got="$(view --id "$ROUTER" -- get_factory)"; then
  [[ "$got" == "\"$FACTORY\"" ]] || die "Router initialized to unexpected factory."
else
  [[ "$(jq -r '.deployment_model // "legacy"' "$OUT")" != atomic-constructor-v1 ]] || die "Constructor deployment has unreadable state; investigate, do not initialize."
  view --id "$ROUTER" -- initialize --factory "$FACTORY" >/dev/null
  NEED_ROUTER=1
fi
log "Plan: initialize factory=$NEED_FACTORY router=$NEED_ROUTER (0 means already correct)."
[[ $APPLY -eq 1 ]] || { log "Simulation only. Human approval is required before --apply."; exit 0; }
[[ -n "$SOURCE" ]] || die "--source required."
check_identity "$SOURCE"
[[ -t 0 ]] || die "Interactive human approval required."
read -r -p "Type 'INITIALIZE EXISTING TESTNET CONTRACTS' to submit: " answer
[[ "$answer" == 'INITIALIZE EXISTING TESTNET CONTRACTS' ]] || die "Not approved."
# Re-runs are safe: initialized instances are checked above and skipped.
if [[ $NEED_FACTORY -eq 1 ]]; then
  "$STELLAR" contract invoke "${TESTNET_ARGS[@]}" --source "$SOURCE" --send=yes --id "$FACTORY" -- initialize --setter "$SETTER" --pair_wasm_hash "$HASH"
fi
if [[ $NEED_ROUTER -eq 1 ]]; then
  "$STELLAR" contract invoke "${TESTNET_ARGS[@]}" --source "$SOURCE" --send=yes --id "$ROUTER" -- initialize --factory "$FACTORY"
fi
"$ROOT_DIR/scripts/verify-deployment.sh"
