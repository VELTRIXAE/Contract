# Public Testnet verification

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
