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
check_network "$NETWORK"; require_stellar; require_jq
OUT="$(deployment_file "$NETWORK")"
[[ -f "$OUT" ]] || die "$OUT not found."
SRC_ARGS=(); [[ -n "$SOURCE" ]] && SRC_ARGS=(--source "$SOURCE")

FACTORY_ID="$(jq -r .contracts.factory.id "$OUT")"
ROUTER_ID="$(jq -r .contracts.router.id "$OUT")"
FAIL=0
pass() { printf '  \033[32mPASS\033[0m %s\n' "$*"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$*"; FAIL=1; }
view() { "$STELLAR" contract invoke "${SRC_ARGS[@]}" --network "$NETWORK" --send=no "$@" 2>/dev/null | tr -d '"'; }

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
  if "$STELLAR" contract fetch --id "$id" --network "$NETWORK" --out-file "$tmpd/$c.wasm" 2>/dev/null; then
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
done < <(jq -c '.pools[]?' "$OUT")

[[ $FAIL -eq 0 ]] && log "All checks passed." || die "Verification failed."
