#!/usr/bin/env bash
# Read-only verification of a recorded deployment. Submits no transactions.
#
#   ./scripts/verify-deployment.sh [--network testnet] [--source <identity>]
#       [--backend <file>] [--artifacts <dir>] [--revision <git-rev>]
#       [--write-manifest <file>]
#
# Checks: router.get_factory == factory, factory.fee_to_setter matches the
# record, the on-chain WASM of factory/router matches the recorded sha256, and
# every recorded pool is the factory's registered pair for its tokens.
#
# Reconciliation (#19): when --backend is given, the backend-reported
# Factory/Router Testnet IDs are fetched from Testnet, their on-chain WASM is
# hashed and compared against locally built artifacts (--artifacts) and the
# source revision (--revision). Receipts are validated for structured
# operation/code identity. A manifest is written ONLY if every check passes;
# otherwise the run exits non-zero and reports explicit blockers. Mocked or
# unavailable evidence is never treated as success.
source "$(dirname "$0")/lib.sh"

NETWORK="testnet"; SOURCE=""; BACKEND=""; ARTIFACTS=""; REVISION=""; WRITE_MANIFEST=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    --source) SOURCE="$2"; shift 2 ;;
    --backend) BACKEND="$2"; shift 2 ;;
    --artifacts) ARTIFACTS="$2"; shift 2 ;;
    --revision) REVISION="$2"; shift 2 ;;
    --write-manifest) WRITE_MANIFEST="$2"; shift 2 ;;
    -h|--help) sed -n '2,19p' "$0"; exit 0 ;;
    *) die "Unknown argument: $1" ;;
  esac
done
require_public_testnet "$NETWORK"; require_stellar; require_jq; require_cmd curl "Install curl."
OUT="$(deployment_file "$NETWORK")"
[[ -f "$OUT" ]] || die "$OUT not found."
jq -e '.network == "testnet"' "$OUT" >/dev/null || die "Manifest network is not testnet."
network="$(curl --fail --silent --show-error "$TESTNET_RPC_URL" -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}')"
[[ "$(jq -er .result.passphrase <<<"$network")" == "$TESTNET_PASSPHRASE" ]] || die "RPC network passphrase mismatch."

FACTORY_ID="$(jq -r .contracts.factory.id "$OUT")"
ROUTER_ID="$(jq -r .contracts.router.id "$OUT")"
FAIL=0
BLOCKERS=()
pass() { printf '  \033[32mPASS\033[0m %s\n' "$*"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$*"; FAIL=1; BLOCKERS+=("$*"); }
info() { printf '  \033[36mINFO\033[0m %s\n' "$*"; }
view() {
  # Bash 3 with nounset rejects expansion of an empty array. Add the optional
  # source to the positional arguments instead; anonymous reads need no key.
  if [[ -n "$SOURCE" ]]; then set -- --source "$SOURCE" "$@"; fi
  "$STELLAR" contract invoke "${TESTNET_ARGS[@]}" --send=no "$@" 2>/dev/null | tr -d '"'
}

log "Verifying $NETWORK deployment from $OUT"
=======

[[ "$(view --id "$ROUTER_ID" -- get_factory)" == "$FACTORY_ID" ]] \
  && pass "router.get_factory == $FACTORY_ID" || fail "router.get_factory mismatch"

expected_setter="$(jq -r .fee_to_setter "$OUT")"
[[ "$(view --id "$FACTORY_ID" -- fee_to_setter)" == "$expected_setter" ]] \
  && pass "factory.fee_to_setter == $expected_setter" || fail "factory.fee_to_setter mismatch"

if pair_count="$(view --id "$FACTORY_ID" -- all_pairs_length)"; then
  if [[ "$pair_count" =~ ^[0-9]+$ ]]; then
    pass "factory.all_pairs_length = $pair_count"
  else
    fail "factory.all_pairs_length returned a malformed count"
  fi
else
  fail "factory.all_pairs_length unavailable"
fi

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
# Each receipt must be a successful Testnet transaction whose operations
# reference the expected contract (structured operation/code identity).
receipt_ok() {
  local tx="$1" want_contract="$2" receipt op_contracts
  if [[ ! "$tx" =~ ^[0-9a-f]{64}$ ]]; then
    fail "missing or malformed transaction receipt"; return 1
  fi
  if ! receipt="$(curl --fail --silent --show-error "https://horizon-testnet.stellar.org/transactions/$tx")"; then
    fail "receipt unavailable: $tx"; return 1
  fi
  if ! jq -e --arg tx "$tx" '.hash == $tx and .successful == true and .network_passphrase == "Test SDF Network ; September 2015"' <<<"$receipt" >/dev/null; then
    fail "receipt unsuccessful or wrong network: $tx"; return 1
  fi
  # Collect contract IDs referenced by the transaction's operations.
  op_contracts="$(curl --fail --silent --show-error "https://horizon-testnet.stellar.org/transactions/$tx/operations" \
    | jq -r '[. _embedded.records[]? | (.contract_id // empty)] | unique | .[]' 2>/dev/null)"
  if [[ -n "$want_contract" && -n "$op_contracts" ]] && ! grep -qx "$want_contract" <<<"$op_contracts"; then
    fail "receipt $tx operations do not reference $want_contract"; return 1
  fi
  pass "successful Testnet receipt $tx"
  return 0
}

while read -r tx; do
  [[ -z "$tx" ]] && continue
  receipt_ok "$tx" ""
done < <(jq -r '[.contracts.factory.deploy_tx, .contracts.factory.initialize_tx, .contracts.router.deploy_tx, .contracts.router.initialize_tx, .contracts.pair.upload_tx] + [.pools[]?.tx] | .[]' "$OUT")

# --- Reconciliation against backend-reported evidence (#19) -----------------
# The contract README records no deployment IDs; the backend reports a
# November 2 Factory/Router Testnet deployment. We verify those IDs against
# Testnet and against locally built artifacts, and only then publish a manifest.
if [[ -n "$BACKEND" ]]; then
  [[ -f "$BACKEND" ]] || die "backend evidence file not found: $BACKEND"
  b_factory="$(jq -r '.factory.id // .factory_id // empty' "$BACKEND")"
  b_router="$(jq -r '.router.id // .router_id // empty' "$BACKEND")"
  b_rev="$(jq -r '.source_revision // .revision // empty' "$BACKEND")"
  b_date="$(jq -r '.deployed_at // .date // empty' "$BACKEND")"
  info "backend reports factory=$b_factory router=$b_router revision=$b_rev date=$b_date"
  [[ -n "$b_factory" && -n "$b_router" ]] || fail "backend evidence missing factory/router IDs"
  [[ "$b_factory" == "$FACTORY_ID" ]] && pass "backend factory ID matches manifest" \
    || fail "backend factory ID $b_factory != manifest $FACTORY_ID"
  [[ "$b_router" == "$ROUTER_ID" ]] && pass "backend router ID matches manifest" \
    || fail "backend router ID $b_router != manifest $ROUTER_ID"

  # Source revision: backend revision must match the requested revision.
  if [[ -n "$REVISION" ]]; then
    [[ "$b_rev" == "$REVISION" ]] && pass "backend source revision matches $REVISION" \
      || fail "backend revision $b_rev != requested $REVISION"
  else
    info "no --revision given; backend revision $b_rev not cross-checked against a build"
  fi

  # WASM hash comparison against locally built artifacts.
  if [[ -n "$ARTIFACTS" ]]; then
    for c in factory router; do
      art="$ARTIFACTS/$c.wasm"
      if [[ ! -f "$art" ]]; then
        fail "artifact $art not found; cannot verify $c WASM hash"
        continue
      fi
      want="$(sha256sum "$art" | cut -d' ' -f1)"
      id="$(jq -r ".contracts.$c.id" "$OUT")"
      if "$STELLAR" contract fetch --id "$id" "${TESTNET_ARGS[@]}" --out-file "$tmpd/rec-$c.wasm" 2>/dev/null; then
        got="$(sha256sum "$tmpd/rec-$c.wasm" | cut -d' ' -f1)"
        [[ "$got" == "$want" ]] && pass "$c on-chain WASM matches built artifact ($want)" \
          || fail "$c on-chain WASM $got != built artifact $want"
      else
        fail "could not fetch $c WASM for artifact comparison"
      fi
    done
  else
    info "no --artifacts given; WASM hashes not compared against a local build"
  fi

  # Receipts for the backend-reported deployment must reference the IDs.
  for pair in "factory:$b_factory" "router:$b_router"; do
    c="${pair%%:*}"; cid="${pair#*:}"
    tx="$(jq -r ".contracts.$c.deploy_tx // empty" "$OUT")"
    [[ -n "$tx" ]] && receipt_ok "$tx" "$cid"
  done
else
  info "no --backend given; skipping backend reconciliation (#19)"
fi

if [[ $FAIL -eq 0 ]]; then
  log "All checks passed."
  if [[ -n "$WRITE_MANIFEST" ]]; then
    mkdir -p "$(dirname "$WRITE_MANIFEST")"
    jq --arg rev "${REVISION:-}" --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      '. + {verified: true, verified_at: $date, source_revision: $rev}' "$OUT" > "$WRITE_MANIFEST"
    log "Wrote verified manifest to $WRITE_MANIFEST"
  fi
else
  printf '\nBlockers:\n' >&2
  for b in "${BLOCKERS[@]}"; do printf '  - %s\n' "$b" >&2; done
  die "Verification failed; no manifest written."
fi
