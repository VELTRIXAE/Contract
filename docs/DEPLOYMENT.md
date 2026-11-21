# Testnet deployment

> Deployment is **never automatic**. Every script that submits transactions
> asks for an explicit confirmation phrase (or the
> `--i-authorize-testnet-deploy` flag), refuses mainnet unconditionally, and
> uses keys only from the Stellar CLI keystore. Public Testnet RPC and passphrase
> are explicit; local aliases cannot redirect these scripts.

> **Not yet exercised against the network.** The scripts were written for
> Stellar CLI v28 and their argument handling and safety guards were tested
> locally. An older deployment's public records have since been recovered in
> the backend repository; they do not exercise this constructor-based flow. See
> [TESTNET_VERIFICATION.md](TESTNET_VERIFICATION.md).

## Prerequisites

- Rust toolchain (from `rust-toolchain.toml`), `make`, `jq`
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) v28 or newer
  (`stellar --version`)
- A funded testnet identity in the CLI keystore:

```bash
stellar keys generate veltrixae-deployer --network testnet --fund
stellar keys address veltrixae-deployer
```

The secret key stays in the CLI keystore (`~/.config/stellar`). Never paste
it into a script, a file in this repository or an environment variable that
gets committed.

## 1. Build, optimize, test

```bash
make test                 # builds WASM and runs all tests
./scripts/build.sh        # build + `stellar contract optimize` + sha256 list
```

## 2. Deploy factory and router

```bash
./scripts/deploy-testnet.sh --source veltrixae-deployer
# optional: --fee-setter <G... of a multisig>   (defaults to the deployer)
```

The script (not executed during this review):

1. prints the plan and waits for you to type `DEPLOY TO TESTNET`
2. uploads the pair WASM (`stellar contract upload`) and keeps its hash
3. deploys the factory with constructor arguments `-- --setter ...
   --pair_wasm_hash ...`; initialization is atomic in that transaction, then
   verifies `fee_to_setter` using simulation
4. deploys the router with constructor argument `-- --factory ...`, then
   verifies `get_factory` using simulation
5. writes `deployments/testnet.json` with the real contract IDs, WASM
   hashes, and the transaction hashes reported by the CLI

Explicitly approved, manually initiated non-interactive use requires the flag
below. Do not deploy from CI:

```bash
./scripts/deploy-testnet.sh --source veltrixae-deployer --i-authorize-testnet-deploy
```

## 3. Create pools

```bash
# Pair only
./scripts/create-pool.sh --source veltrixae-deployer \
  --token-a C...TOKENA --token-b C...TOKENB

# Pair + initial liquidity through the router (amounts in smallest units)
./scripts/create-pool.sh --source veltrixae-deployer \
  --token-a C...TOKENA --token-b C...TOKENB \
  --amount-a 10000000000 --amount-b 40000000000
```

For a classic Stellar asset, get its contract ID with
`stellar contract id asset --asset CODE:ISSUER --network testnet` and deploy
the SAC first if needed (`stellar contract asset deploy ...`). The source
account must hold both tokens (and have trustlines for classic assets).

Each created pool is appended to `deployments/testnet.json`.

## 4. Verify

```bash
./scripts/verify-deployment.sh --network testnet
```

Read-only (simulation only, submits nothing). Checks:

- `router.get_factory()` equals the recorded factory
- `factory.fee_to_setter()` equals the recorded setter
- the on-chain WASM of factory and router matches the recorded sha256
- each recorded pool is the factory's registered pair for its tokens

## 5. Publish to consumers

- Give `deployments/testnet.json` (IDs) to veltrixae-frontend and
  veltrixae-aggregator.
- Regenerate bindings if the interface changed: `make bindings`.
- Commit `deployments/testnet.json` together with the git commit it was
  built from (recorded in the file as `git_commit`).

## Durable deployment journal

`scripts/deploy-testnet.sh` keeps a public-only journal of every successful
step so a failure after upload or deployment does not lose provenance. The
journal lives at:

```
deployments/testnet.journal.json
```

It is written **atomically** (temp file + `fsync` + rename) after each
successful step, so a crash leaves either the previous or the new complete
record, never a partial one. The journal is append-only in effect: existing
records are never rewritten, and the final `deployments/testnet.json` is
refused if it already exists (the script will not overwrite a manifest).

### Schema

```json
{
  "schema": "veltrixae-deployment-journal-v1",
  "network": "testnet",
  "network_passphrase": "Test SDF Network ; September 2015",
  "git_commit": "<sha of the tree the WASM was built from>",
  "created_at": "<RFC3339 UTC>",
  "steps": [
    {
      "step": "upload-pair-wasm",
      "status": "succeeded",
      "artifact_sha256": "<sha256 of the optimized pair WASM>",
      "wasm_hash": "<on-chain WASM hash returned by the CLI>",
      "receipt": {
        "operation": "upload",
        "contract_id": null,
        "wasm_hash": "<same as wasm_hash>",
        "tx_hash": "<transaction hash>",
        "ledger": 123456
      },
      "recorded_at": "<RFC3339 UTC>"
    }
  ]
}
```

Only public data is recorded: contract IDs, WASM hashes, artifact sha256
digests, transaction hashes, ledger numbers and timestamps. The journal
**never** contains secret keys, CLI keystore contents, signed envelopes or
secret environment variables. Do not add them.

### Structured receipt validation

Receipts are validated structurally, not by scraping the last hexadecimal
string from CLI output. A receipt is accepted only if:

- its `operation` matches the step (`upload`, `deploy`, `invoke`);
- for `deploy`/`invoke`, the `contract_id` equals the ID the step was
  expected to produce (mismatched contract identity is rejected);
- for `upload`, the `wasm_hash` equals the hash the CLI reported for that
  upload (mismatched code identity is rejected);
- the `tx_hash` is a well-formed 64-hex-character transaction hash.

Unrelated operations or mismatched contract/code identities fail the step
loudly. A malformed or stale journal (wrong `schema`, wrong `network`,
unparseable JSON) aborts the script with a clear error rather than being
silently ignored or overwritten.

### Dry-run recovery view

To see which steps completed and which are missing, without submitting any
transaction:

```bash
./scripts/deploy-testnet.sh --recover --network testnet
```

This is read-only: it parses the journal, prints a report of completed versus
missing steps (with artifact hashes and receipt identifiers), and exits. It
does not upload, deploy, invoke or resume anything. Automatic transaction
resumption is out of scope; resuming requires a separately reviewed change.

### Manual recovery procedure

1. Run the dry-run recovery view above and note the completed steps.
2. Compare the journal's `git_commit` and artifact hashes with the tree you
   intend to deploy. If they differ, stop: the journal is stale for this
   build.
3. For a missing step, re-run the deployment manually with human approval.
   The script refuses to overwrite an existing `deployments/testnet.json`; if
   a manifest already exists, treat it as authoritative and verify it with
   `./scripts/verify-deployment.sh --network testnet`.
4. Record the outcome in the pull request. Never hand-edit the journal or the
   manifest to make them agree.

## After a deployment: record keeping

Only values returned by the network belong in `deployments/*.json`. Do not
hand-edit IDs or hashes. If a deployment is abandoned (e.g. state mismatch), note it in the pull request but do not record the abandoned IDs as
live.

## Reconciling backend-reported deployments

The backend may report a deployment (for example a November 2 Factory/Router
deployment) that the contract repository has no manifest for. Reconciling it
is a separate, read-only task from the durable journal above:

1. Fetch the reported contract IDs and transaction hashes from the backend
   records.
2. Query Testnet for each ID (`stellar contract info`, ledger/transaction
   lookups) and confirm the operations exist and match the reported IDs.
3. Match the on-chain WASM hash against a locally rebuilt artifact and record
   the source revision it corresponds to.
4. Publish a manifest **only** if every check passes. Otherwise record the
   mismatch as a blocker.

Mocked or simulated evidence is not success. Do not claim a deployment is
verified on the basis of offline mocks, log scraping or backend assertions
alone; only network-returned values count.

## Mainnet

Out of scope. The scripts refuse `mainnet`/`public` network names. A mainnet
launch needs an independent audit, a multisig `fee_to_setter`, and a
separate, reviewed procedure.

## Resume incomplete initialization

Use `scripts/remediate-testnet.sh` to verify code/state and print a simulation-only
plan for recovered existing contracts. It never deploys replacements. A missing
manifest is a blocker, not authorization to redeploy. Review the plan and obtain
human approval before the explicit `--apply --source KEYSTORE_NAME` mode, which
also requires an interactive confirmation phrase. Record resulting public
receipts and run the read-only verifier. Legacy artifacts still have the original
initialization race; a conflicting initialized contract is refused. New manifests
use `deployment_model: atomic-constructor-v1`; deployment and initialization
share the same actual receipt. New constructor manifests never attempt recovery
by calling initialize after an unreadable state check. Missing constructor
arguments fail creation, rather than leaving an uninitialized instance.

Constructor support is a deployment-interface change. Use a CLI that supports
constructor arguments after `--`; the [v28 deploy source](https://github.com/stellar/stellar-cli/blob/v28.0.0/cmd/soroban-cli/src/commands/contract/deploy/wasm.rs) documents that parsing.
This review exercised production WASM through SDK 22 `deploy_v2`, but no live CLI
transaction was executed. No approval to deploy is implied by these instructions.
