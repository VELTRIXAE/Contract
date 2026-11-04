#!/usr/bin/env bash
# Deploy and initialize VELTRIXAE Factory and Router on Stellar TESTNET.
#
# Nothing is deployed unless you explicitly authorize it, either by typing the
# confirmation phrase when prompted or by passing --i-authorize-testnet-deploy.
# Mainnet is refused unconditionally.
#
# Usage:
#   ./scripts/deploy-testnet.sh --source <stellar-keys-name> [--network testnet]
#                               [--fee-setter <G...>] [--i-authorize-testnet-deploy]
#
# Steps:
#   1. build (and optimize if possible) the WASM
#   2. upload the pair WASM and record its hash
#   3. deploy + initialize the factory, then verify fee_to_setter
#   4. deploy + initialize the router, then verify get_factory
#   5. write contract IDs, WASM hashes and tx hashes to deployments/<network>.json
#
# Constructors initialize factory/router in their deployment transactions.
# There is no separate uninitialized interval or initialize transaction.
source "$(dirname "$0")/lib.sh"

NETWORK="testnet"
SOURCE=""
FEE_SETTER=""
AUTHORIZED=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="$2"; shift 2 ;;
    --source) SOURCE="$2"; shift 2 ;;
    --fee-setter) FEE_SETTER="$2"; shift 2 ;;
    --i-authorize-testnet-deploy) AUTHORIZED=1; shift ;;
    -h|--help) sed -n '2,25p' "$0"; exit 0 ;;
    *) die "Unknown argument: $1" ;;
  esac
done

[[ -n "$SOURCE" ]] || die "--source <identity> is required (a name from 'stellar keys ls')."
require_public_testnet "$NETWORK"
require_stellar
require_jq
check_identity "$SOURCE"

SOURCE_ADDR="$("$STELLAR" keys address "$SOURCE")"
FEE_SETTER="${FEE_SETTER:-$SOURCE_ADDR}"
OUT="$(deployment_file "$NETWORK")"
[[ ! -e "$OUT" ]] || die "Existing record $OUT: verify or remediate it; refusing redeployment."
LOG_DIR="$(mktemp -d)"
trap 'rm -rf "$LOG_DIR"' EXIT

cat >&2 <<MSG

  VELTRIXAE deployment plan
  -------------------------
  network      : $NETWORK
  source       : $SOURCE ($SOURCE_ADDR)
  fee setter   : $FEE_SETTER
  output       : $OUT

  This will submit real transactions to $NETWORK.
MSG

if [[ $AUTHORIZED -ne 1 ]]; then
  [[ -t 0 ]] || die "Not a terminal. Re-run with --i-authorize-testnet-deploy to authorize."
  read -r -p "  Type 'DEPLOY TO TESTNET' to continue: " answer
  [[ "$answer" == "DEPLOY TO TESTNET" ]] || die "Not authorized. Nothing was deployed."
fi

"$ROOT_DIR/scripts/build.sh"

pick_wasm() {
  local c="$1"
  if [[ -f "$WASM_DIR/$c.optimized.wasm" ]]; then echo "$WASM_DIR/$c.optimized.wasm"; else echo "$WASM_DIR/$c.wasm"; fi
}
PAIR_WASM="$(pick_wasm veltrixae_pair)"
FACTORY_WASM="$(pick_wasm veltrixae_factory)"
ROUTER_WASM="$(pick_wasm veltrixae_router)"

run() { # run <logname> <stellar args...>  -> stdout of the command; log kept for tx hash
  local name="$1"; shift
  local command="$1" subcommand="$2"; shift 2
  # CLI options must precede the contract method separator (--).
  local status=0
  "$STELLAR" "$command" "$subcommand" --source "$SOURCE" "${TESTNET_ARGS[@]}" "$@" 2> "$LOG_DIR/$name.log" || status=$?
  cat "$LOG_DIR/$name.log" >&2
  return "$status"
}

log "Uploading pair WASM"
PAIR_HASH="$(run upload_pair contract upload --wasm "$PAIR_WASM" | tail -n1)"
PAIR_UPLOAD_TX="$(extract_tx_hash "$LOG_DIR/upload_pair.log")"
[[ "$PAIR_HASH" =~ ^[0-9a-f]{64}$ ]] || die "Unexpected pair WASM hash: $PAIR_HASH"

log "Deploying factory"
FACTORY_ID="$(run deploy_factory contract deploy --wasm "$FACTORY_WASM" -- --setter "$FEE_SETTER" --pair_wasm_hash "$PAIR_HASH" | tail -n1)"
FACTORY_DEPLOY_TX="$(extract_tx_hash "$LOG_DIR/deploy_factory.log")"

# Constructor initialization and deployment have the same actual receipt.
FACTORY_INIT_TX="$FACTORY_DEPLOY_TX"

ACTUAL_SETTER="$(run verify_factory contract invoke --send=no --id "$FACTORY_ID" -- fee_to_setter | tr -d '"')"
[[ "$ACTUAL_SETTER" == "$FEE_SETTER" ]] \
  || die "Factory fee_to_setter is '$ACTUAL_SETTER', expected '$FEE_SETTER'. Refusing to proceed; investigate code, arguments and receipt."

log "Deploying router"
ROUTER_ID="$(run deploy_router contract deploy --wasm "$ROUTER_WASM" -- --factory "$FACTORY_ID" | tail -n1)"
ROUTER_DEPLOY_TX="$(extract_tx_hash "$LOG_DIR/deploy_router.log")"

ROUTER_INIT_TX="$ROUTER_DEPLOY_TX"

ACTUAL_FACTORY="$(run verify_router contract invoke --send=no --id "$ROUTER_ID" -- get_factory | tr -d '"')"
[[ "$ACTUAL_FACTORY" == "$FACTORY_ID" ]] \
  || die "Router factory is '$ACTUAL_FACTORY', expected '$FACTORY_ID'. Refusing to proceed; investigate code, arguments and receipt."

mkdir -p "$DEPLOYMENTS_DIR"
sha() { sha256sum "$1" | cut -d' ' -f1; }
jq -n \
  --arg network "$NETWORK" \
  --arg deployed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg deployer "$SOURCE_ADDR" \
  --arg fee_setter "$FEE_SETTER" \
  --arg commit "$(git -C "$ROOT_DIR" rev-parse HEAD 2>/dev/null || echo unknown)" \
  --arg factory "$FACTORY_ID" --arg router "$ROUTER_ID" \
  --arg pair_hash "$PAIR_HASH" \
  --arg factory_wasm "$(sha "$FACTORY_WASM")" --arg router_wasm "$(sha "$ROUTER_WASM")" \
  --arg pair_upload_tx "$PAIR_UPLOAD_TX" \
  --arg factory_deploy_tx "$FACTORY_DEPLOY_TX" --arg factory_init_tx "$FACTORY_INIT_TX" \
  --arg router_deploy_tx "$ROUTER_DEPLOY_TX" --arg router_init_tx "$ROUTER_INIT_TX" \
  '{
    deployment_model: "atomic-constructor-v1",
    network: $network, deployed_at: $deployed_at, git_commit: $commit,
    deployer: $deployer, fee_to_setter: $fee_setter,
    contracts: {
      factory: { id: $factory, wasm_sha256: $factory_wasm,
                 deploy_tx: $factory_deploy_tx, initialize_tx: $factory_init_tx },
      router:  { id: $router, wasm_sha256: $router_wasm,
                 deploy_tx: $router_deploy_tx, initialize_tx: $router_init_tx },
      pair:    { wasm_hash: $pair_hash, upload_tx: $pair_upload_tx }
    },
    pools: []
  }' > "$OUT"

log "Deployment recorded in $OUT"
cat "$OUT"
log "Next: ./scripts/verify-deployment.sh --network $NETWORK"
