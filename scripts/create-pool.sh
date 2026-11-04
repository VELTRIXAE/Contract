#!/usr/bin/env bash
# Create a VELTRIXAE pool on TESTNET and optionally seed it with liquidity.
#
# Usage:
#   ./scripts/create-pool.sh --source <identity> --token-a <C...> --token-b <C...>
#        [--amount-a <i128> --amount-b <i128>] [--network testnet]
#        [--i-authorize-testnet-deploy]
#
# Without amounts, only factory.create_pair is called. With amounts, the
# router's add_liquidity is used instead (it creates the pair if needed); the
# source account must hold both tokens. Requires explicit authorization.
source "$(dirname "$0")/lib.sh"

NETWORK="testnet"; SOURCE=""; TOKEN_A=""; TOKEN_B=""; AMOUNT_A=""; AMOUNT_B=""; AUTHORIZED=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    --source) SOURCE="$2"; shift 2 ;;
    --token-a) TOKEN_A="$2"; shift 2 ;;
    --token-b) TOKEN_B="$2"; shift 2 ;;
    --amount-a) AMOUNT_A="$2"; shift 2 ;;
    --amount-b) AMOUNT_B="$2"; shift 2 ;;
    --i-authorize-testnet-deploy) AUTHORIZED=1; shift ;;
    -h|--help) sed -n '2,13p' "$0"; exit 0 ;;
    *) die "Unknown argument: $1" ;;
  esac
done

[[ -n "$SOURCE" && -n "$TOKEN_A" && -n "$TOKEN_B" ]] || die "--source, --token-a and --token-b are required."
[[ "$TOKEN_A" =~ ^C[A-Z2-7]{55}$ && "$TOKEN_B" =~ ^C[A-Z2-7]{55}$ ]] || die "Tokens must be contract IDs (C...)."
[[ "$TOKEN_A" != "$TOKEN_B" ]] || die "token-a and token-b must differ."
if [[ -n "$AMOUNT_A$AMOUNT_B" ]]; then
  [[ "$AMOUNT_A" =~ ^[1-9][0-9]*$ && "$AMOUNT_B" =~ ^[1-9][0-9]*$ ]] || die "Amounts must be positive integers (smallest units)."
fi
require_public_testnet "$NETWORK"; require_stellar; require_jq; check_identity "$SOURCE"

OUT="$(deployment_file "$NETWORK")"
[[ -f "$OUT" ]] || die "$OUT not found. Deploy first with ./scripts/deploy-testnet.sh"
FACTORY_ID="$(jq -r .contracts.factory.id "$OUT")"
ROUTER_ID="$(jq -r .contracts.router.id "$OUT")"
SOURCE_ADDR="$("$STELLAR" keys address "$SOURCE")"
LOG="$(mktemp)"; trap 'rm -f "$LOG"' EXIT

log "Pool: $TOKEN_A / $TOKEN_B on $NETWORK (factory $FACTORY_ID)"
if [[ $AUTHORIZED -ne 1 ]]; then
  [[ -t 0 ]] || die "Not a terminal. Re-run with --i-authorize-testnet-deploy to authorize."
  read -r -p "  Type 'CREATE POOL' to submit: " answer
  [[ "$answer" == "CREATE POOL" ]] || die "Not authorized. Nothing was submitted."
fi

inv() { "$STELLAR" contract invoke --source "$SOURCE" "${TESTNET_ARGS[@]}" "$@" 2> >(tee -a "$LOG" >&2); }

if [[ -z "$AMOUNT_A" ]]; then
  PAIR_ID="$(inv --id "$FACTORY_ID" -- create_pair --token_a "$TOKEN_A" --token_b "$TOKEN_B" | tr -d '"')"
else
  DEADLINE=$(( $(date +%s) + 600 ))
  inv --id "$ROUTER_ID" -- add_liquidity \
    --token_a "$TOKEN_A" --token_b "$TOKEN_B" \
    --amount_a_desired "$AMOUNT_A" --amount_b_desired "$AMOUNT_B" \
    --amount_a_min "$AMOUNT_A" --amount_b_min "$AMOUNT_B" \
    --to "$SOURCE_ADDR" --deadline "$DEADLINE" >/dev/null
  PAIR_ID="$(inv --send=no --id "$FACTORY_ID" -- get_pair --token_a "$TOKEN_A" --token_b "$TOKEN_B" | tr -d '"')"
fi
TX="$(extract_tx_hash "$LOG")"
[[ "$PAIR_ID" =~ ^C[A-Z2-7]{55}$ ]] || die "Unexpected pair id: $PAIR_ID"

RESERVES="$(inv --send=no --id "$PAIR_ID" -- get_reserves)"
tmp="$(mktemp)"
jq --arg pair "$PAIR_ID" --arg a "$TOKEN_A" --arg b "$TOKEN_B" --arg tx "$TX" \
   --arg at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
   '.pools += [{pair: $pair, token_a: $a, token_b: $b, tx: $tx, created_at: $at}]' "$OUT" > "$tmp" && mv "$tmp" "$OUT"
log "Pair $PAIR_ID recorded in $OUT (reserves: $RESERVES)"
