# Public Testnet verification

## Reconciliation of backend-reported 2026-11-02 deployment

The Contract README records no deployment IDs. The backend reports a November 2
Factory/Router deployment. This section reconciles those receipts against public
Testnet evidence. It is distinct from #13 (durable partial receipts): #13 covers
persisting incomplete receipt data, while this section covers whether the
reported deployment can be verified at all.

**Manifest status: NOT PUBLISHED.** No `deployments/testnet.json` is published
unless every check below passes against live public Testnet. Mocked, fixture or
offline evidence is not success and must not be recorded as verification.

### Reproducible read-only steps

These steps require no key, funded account or write. `FACTORY_ID` and
`ROUTER_ID` are the IDs reported by the backend; `SOURCE_REV` is the reported
source revision.

```bash
FACTORY_ID=CCBE2WCHO5PY6WVXTKZLIZA446M43RTOVD4S6XZYHPHJEL7K33LHW6RA
ROUTER_ID=CDSNTA4DH5CXDJJMY4QQ6IUF2M3VLR2WVPCL3UUQMCVUDQ5ZHVYWUZA4
SOURCE_REV=133b18d00505108c26382cb44045abab48b51882

# 1. Confirm public Testnet identity.
curl --fail --silent --show-error https://horizon-testnet.stellar.org/ \
  | jq '{network_passphrase, history_latest_ledger, history_latest_ledger_closed_at}'
curl --fail --silent --show-error https://soroban-testnet.stellar.org/ \
  -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}' | jq .

# 2. Fetch each contract's WASM and hash it.
for ID in "$FACTORY_ID" "$ROUTER_ID"; do
  stellar contract fetch --id "$ID" --network testnet --out-file "/tmp/$ID.wasm"
  sha256sum "/tmp/$ID.wasm"
done

# 3. Rebuild from the reported source revision with the recorded toolchain and
#    compare artifact hashes to step 2. Requires the preserved backend bundle.
git -C <contract-checkout> rev-parse "$SOURCE_REV"
# build with the exact compiler/optimizer versions from DEPLOYMENT_RECONCILIATION.md
sha256sum <built-factory.wasm> <built-router.wasm>

# 4. Verify router/factory linkage and initialization state (read-only getters).
stellar contract invoke --id "$ROUTER_ID" --network testnet --send=no -- factory
stellar contract invoke --id "$FACTORY_ID" --network testnet --send=no -- all_pairs_length

# 5. Verify each reported receipt against public Horizon.
#    TX_HASH must be an actual 64-hex hash from the backend evidence.
curl --fail --silent --show-error \
  "https://horizon-testnet.stellar.org/transactions/$TX_HASH" \
  | jq '{hash, successful, ledger, envelope_xdr, result_xdr, result_meta_xdr}'
```

### Actual results

| Check | Result | Notes |
| --- | --- | --- |
| Factory WASM hash vs recorded SHA-256 | PASS | Observed WASM matches recorded hash |
| Router WASM hash vs recorded SHA-256 | PASS | Observed WASM matches recorded hash |
| Router→Factory linkage | PASS | Router's `factory` getter matches reported Factory ID |
| Source revision | PARTIAL | `133b18d...` is an older local branch preserved in the backend bundle, not a public Contract branch; byte-identical rebuild requires that bundle |
| Receipts | PARTIAL | Seven historical transactions match success, hash, ledger, fee and source through Horizon; outside current RPC retention; XDR operation binding not independently decoded |
| Pair code | BLOCKED | Uploaded WASM hash matches, but no Pair instance exists and reported code TTL is zero |
| Pools | BLOCKED | Zero; no funded pool or live wallet-signed swap demonstrated |
| Aggregator | BLOCKED | Not deployed |
| Deployment of current Contract `main` | BLOCKED | Not demonstrated |

### Explicit blockers

- The reported source revision is not a public Contract branch; reproducing the
  byte-identical build requires the preserved backend evidence bundle. This
  checkout alone is insufficient.
- Receipts are outside current RPC retention; only Horizon status/hash/ledger/fee
  checks were possible. Each historical XDR operation is not independently
  decoded and bound to a contract operation.
- Pair code TTL is reported as zero; usable code lifetime/restoration must be
  checked before any pool creation.
- No funded pool, live swap or Aggregator deployment exists to verify.
- Licensing remains unresolved; see [LICENSING_REVIEW.md](LICENSING_REVIEW.md).

Because the source-revision rebuild and XDR provenance binding are not fully
verified, **no manifest is published**. Mocked or fixture evidence does not
satisfy any row above.

## Current evidence — 2026-11-21

The November 4 review below describes the records available in that checkout.
Public records have since been recovered from the backend repository. This
supersedes its blanket missing-records status, without claiming deployment of
current Contract `main`.

Deployed source: `133b18d00505108c26382cb44045abab48b51882`, an older local
branch preserved in the backend evidence bundle, not a public Contract branch.
Current constructor and rounding fixes do not alter those immutable contracts.

| Check | Current result |
| --- | --- |
| Factory | `CCBE2WCHO5PY6WVXTKZLIZA446M43RTOVD4S6XZYHPHJEL7K33LHW6RA`; observed WASM matches recorded SHA-256 |
| Router | `CDSNTA4DH5CXDJJMY4QQ6IUF2M3VLR2WVPCL3UUQMCVUDQ5ZHVYWUZA4`; observed WASM matches recorded SHA-256; factory link matches |
| Pair code | Uploaded WASM hash matches; no Pair instance exists; reported code TTL is zero, so usable code lifetime/restoration must be checked before pool creation |
| Receipts | Seven historical transactions match success, hash, ledger, fee and source through Horizon; outside current RPC retention |
| Pools | Zero; no funded VELTRIXAE pool or live wallet-signed swap demonstrated |
| Aggregator | Not deployed |
| Deployment of current main | Not demonstrated |

Evidence lives in [the backend evidence directory](https://github.com/VELTRIXAE/Aggregator-and-Backend/tree/main/docs/evidence).
See `contracts-testnet-deployment-2026-11-02.json`,
`contracts-testnet-completion-2026-11-02.json`,
`testnet-verification-2026-11-21.json` and `horizon-receipts-2026-11-21.json`.
The [reconciliation](https://github.com/VELTRIXAE/Aggregator-and-Backend/blob/main/docs/DEPLOYMENT_RECONCILIATION.md)
records the historical byte-identical rebuild and toolchain. Reproducing that
build requires its preserved bundle; this checkout alone is insufficient.
Receipt status checks do not independently decode/bind each historical XDR
operation to a contract. Preserve that provenance limitation.

Licensing remains unresolved; see [LICENSING_REVIEW.md](LICENSING_REVIEW.md).
No chain write was performed for this verification. Legacy manifests are not
placed in `deployments/`, where current scripts might act on them.

## Historical review — 2026-11-04

Checked 2026-11-04 from baseline `499fef4` plus this review's changes.
**Project deployment evidence: NOT VERIFIED.** The maintainer confirmed no
verified IDs, manifests or transaction hashes are currently available.

No deployment, initialization, upload, pool creation, funding or TTL transaction
was submitted. All live requests were read-only public HTTP/RPC requests.

## Actual results

| Check | Result | Evidence |
|---|---|---|
| Horizon public network identity | PASS | `review/evidence/horizon-root.json`: network_passphrase = `Test SDF Network ; September 2015`, ledger 5554252, closed 2026-11-04T12:21:40Z |
| Soroban RPC public network identity | PASS | `review/evidence/rpc-network.json`: same passphrase, protocolVersion 30 |
| Tracked deployment manifests | NOT RUN for contract verification | `git ls-files deployments` lists only README.md; no recorded IDs, receipts or code hashes |
| Contract IDs, successful deployment/initialization receipts | NOT RUN | Missing manifest/receipts; Stellar CLI is not installed locally |
| Project on-chain code hashes, initialized state, reserves/admin | NOT RUN | No contract IDs available to query |
| New remediation on real network | NOT RUN | Requires recovered IDs, installed CLI and human approval for writes |

Available `git log --all -- deployments` contains the script/documentation
introduction, with no deployment JSON. This establishes only absence of evidence
in this checkout and available refs, not absence of any VELTRIXAE deployment.
Public endpoint identity alone is not project deployment evidence.

## Reproduce read-only checks

These commands do not need a key or funded account:

```bash
git ls-files deployments
git log --all --oneline -- deployments
curl --fail --silent --show-error https://horizon-testnet.stellar.org/ | jq '{network_passphrase, history_latest_ledger, history_latest_ledger_closed_at}'
curl --fail --silent --show-error https://soroban-testnet.stellar.org/ \
  -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","id":1,"method":"getNetwork"}' | jq .
```

After a maintainer recovers an authentic `deployments/testnet.json` from prior
records, install a compatible Stellar CLI and run:

```bash
stellar --version
./scripts/verify-deployment.sh --network testnet
```

The verifier pins the public RPC URL/passphrase independently of CLI aliases,
uses `--send=no` for every getter, fetches code for SHA-256 comparison, checks
router/factory linkage, admin, pool registry, pool token identities/code and
reserves, and requires successful receipts from public Testnet Horizon.
It does not treat an unavailable receipt as success. The factory has no public
pair-code-hash getter; inspect its persistent `PairWasmHash` entry using
`getLedgerEntries` with an XDR-encoded contract-data key and compare the returned
hash to the manifest before approving initialization of a dependent router.
That storage/provenance check is not automated by the shell verifier. Old transactions may be
outside provider retention or invalidated by a Testnet reset; report this as a
blocker. Contract code hashes must also be compared to the artifacts built from
the recorded commit with its exact compiler/optimizer versions.

A successful transaction hash alone is not proof of the claimed operation.
For each receipt fetch its `envelope_xdr`, `result_xdr` and `result_meta_xdr`:

```bash
# TX_HASH must be an actual 64-hex hash from the recovered manifest.
curl --fail --silent --show-error \
  "https://horizon-testnet.stellar.org/transactions/$TX_HASH" \
  | jq '{hash, successful, ledger, envelope_xdr, result_xdr, result_meta_xdr}'
```

Decode with the matching Stellar XDR tooling and compare operation, contract
creation result/ID, initialization arguments and uploaded code hash to the
manifest. The verifier checks receipt success/hash, but does not automate that
XDR provenance binding. This step is mandatory before claiming complete verified
deployment provenance. No receipt/XDR values are invented in this review.

## Exact blocker and safe remediation

Missing: existing factory/router/pair IDs, upload/deploy/initialize receipts,
artifact/compiler/optimizer provenance, and current authority information.
Ask the deploying maintainer to recover public records first. Do not create a
new deployment solely to fill the evidence directory. No initialization action
can be selected safely until those records identify the intended contracts.

`./scripts/remediate-testnet.sh` is a simulation-only plan for existing recorded
contracts. It verifies factory/router code, checks current initialization, and
simulates initialization if getters fail; simulation errors fail closed. Correct
initialized contracts are skipped, conflicting state/code is refused. It never
deploys, funds or adds liquidity. It cannot retroactively fix legacy first-caller initialization,
restore archived entries, recover absent receipts, or repair a claimed contract.

Only after separately reviewing/approving the printed plan may a human run
`./scripts/remediate-testnet.sh --apply --source KEYSTORE_NAME`. That mode also
requires an interactive exact confirmation phrase and uses public Testnet only.
New constructor manifests must not use separate initialize recovery. The
updated deploy script passes constructor arguments atomically and records
`deployment_model: atomic-constructor-v1`; initialize_tx equals deploy_tx
because both operations share the same real transaction. This schema describes
a future authorized execution, not evidence of an executed deployment.

Capture actual transaction receipts durably and re-run read-only verification.
It has offline guard tests, but has not been exercised against live contracts.

The verifier now treats an unavailable or malformed `all_pairs_length` response
as FAIL. Offline HTTP/CLI fixtures verify this behavior; it does not establish
project on-chain state. Project deployment evidence remains NOT VERIFIED.
