#!/usr/bin/env bash
# Read-only verification of a recorded deployment. Submits no transactions.
#
#   ./scripts/verify-deployment.sh [--network testnet] [--source <identity>]
#
# Checks: router.get_factory == factory, factory.fee_to_setter matches the
# record, the on-chain WASM of factory/router matches the recorded sha256, and
# every recorded pool is the factory's registered pair for its tokens.
source "$(dirname "$0")/lib.sh"

NETWORK="testnet"; SOURCE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    --source) SOURCE="$2"; shift 2 ;;
    -h|--help) sed -n '2,8p' "$0"; exit 0 ;;
    *) die "Unknown argument: $1" ;;
  esac
done
require_public_testnet "$NETWORK"; require_stellar; require_jq; require_cmd curl "Install curl."
OUT="$(deployment_file "$NETWORK")"
[[ -f "$OUT" ]] || die "$OUT not found."
jq -e '.network == "testnet"' "$OUT" >/dev/null || die "Manifest network is not testnet."
network="$(curl --fail --silent --show-error "$TESTNET_RPC_URL" -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}')"
[[ "$(jq -er .result.passphrase <<<"$network")" == "$TESTNET_PASSPHRASE" ]] || die "RPC network passphrase mismatch."
SRC_ARGS=(); [[ -n "$SOURCE" ]] && SRC_ARGS=(--source "$SOURCE")

FACTORY_ID="$(jq -r .contracts.factory.id "$OUT")"
ROUTER_ID="$(jq -r .contracts.router.id "$OUT")"
FAIL=0
pass() { printf '  \033[32mPASS\033[0m %s\n' "$*"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$*"; FAIL=1; }
view() { "$STELLAR" contract invoke "${SRC_ARGS[@]}" "${TESTNET_ARGS[@]}" --send=no "$@" 2>/dev/null | tr -d '"'; }

log "Verifying $NETWORK deployment from $OUT"

[[ "$(view --id "$ROUTER_ID" -- get_factory)" == "$FACTORY_ID" ]] \
  && pass "router.get_factory == $FACTORY_ID" || fail "router.get_factory mismatch"

expected_setter="$(jq -r .fee_to_setter "$OUT")"
[[ "$(view --id "$FACTORY_ID" -- fee_to_setter)" == "$expected_setter" ]] \
  && pass "factory.fee_to_setter == $expected_setter" || fail "factory.fee_to_setter mismatch"

echo "  INFO all_pairs_length = $(view --id "$FACTORY_ID" -- all_pairs_length)"

tmpd="$(mktemp -d)"; trap 'rm -rf "$tmpd"' EXIT
for c in factory router; do
  id="$(jq -r ".contracts.$c.id" "$OUT")"; want="$(jq -r ".contracts.$c.wasm_sha256" "$OUT")"
  if "$STELLAR" contract fetch --id "$id" "${TESTNET_ARGS[@]}" --out-file "$tmpd/$c.wasm" 2>/dev/null; then
    got="$(sha256sum "$tmpd/$c.wasm" | cut -d' ' -f1)"
    [[ "$got" == "$want" ]] && pass "$c on-chain WASM sha256 matches" || fail "$c WASM sha256 $got != $want"
  else
    fail "could not fetch $c WASM"
  fi
done

while read -r pool; do
  [[ -z "$pool" ]] && continue
  pair="$(jq -r .pair <<<"$pool")"; a="$(jq -r .token_a <<<"$pool")"; b="$(jq -r .token_b <<<"$pool")"
  [[ "$(view --id "$FACTORY_ID" -- get_pair --token_a "$a" --token_b "$b")" == "$pair" ]] \
    && pass "pool $pair registered for its tokens" || fail "pool $pair not registered"
  if "$STELLAR" contract fetch --id "$pair" "${TESTNET_ARGS[@]}" --out-file "$tmpd/pair.wasm"; then
    want="$(jq -r .contracts.pair.wasm_hash "$OUT")"
    got="$(sha256sum "$tmpd/pair.wasm" | cut -d' ' -f1)"
    [[ "$got" == "$want" ]] && pass "pool $pair pair code hash matches" || fail "pool $pair code hash mismatch"
  else
    fail "could not fetch pool $pair code"
  fi
  t0="$(view --id "$pair" -- token_0)"; t1="$(view --id "$pair" -- token_1)"
  if [[ "$t0" == "$a" && "$t1" == "$b" || "$t0" == "$b" && "$t1" == "$a" ]]; then
    pass "pool $pair token identities match"
  else
    fail "pool $pair token identity mismatch"
  fi
  if reserves="$(view --id "$pair" -- get_reserves)"; then
    echo "  INFO pool $pair reserves = $reserves"
  else
    fail "pool $pair reserves unavailable"
  fi
done < <(jq -c '.pools[]?' "$OUT")

# Horizon retains more transaction history than the RPC retention window.
while read -r tx; do
  if [[ ! "$tx" =~ ^[0-9a-f]{64}$ ]]; then
    fail "missing or malformed transaction receipt"; continue
  fi
  if receipt="$(curl --fail --silent --show-error "https://horizon-testnet.stellar.org/transactions/$tx")" &&
     jq -e --arg tx "$tx" '.hash == $tx and .successful == true' <<<"$receipt" >/dev/null; then
    pass "successful Testnet receipt $tx (operation/code provenance still requires XDR review)"
  else
    fail "receipt unavailable or unsuccessful: $tx"
  fi
done < <(jq -r '[.contracts.factory.deploy_tx, .contracts.factory.initialize_tx, .contracts.router.deploy_tx, .contracts.router.initialize_tx, .contracts.pair.upload_tx] + [.pools[]?.tx] | .[]' "$OUT")

[[ $FAIL -eq 0 ]] && log "All checks passed." || die "Verification failed."
