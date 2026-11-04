# Testnet deployment

> Deployment is **never automatic**. Every script that submits transactions
> asks for an explicit confirmation phrase (or the
> `--i-authorize-testnet-deploy` flag), refuses mainnet unconditionally, and
> uses keys only from the Stellar CLI keystore. Public Testnet RPC and passphrase
> are explicit; local aliases cannot redirect these scripts.

> **Not yet exercised against the network.** The scripts were written for
> Stellar CLI v28 and their argument handling and safety guards were tested
> locally. No verified deployment IDs or receipts are available. Recover any
> existing public records before considering a newly approved deployment; see
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

## After a deployment: record keeping

Only values returned by the network belong in `deployments/*.json`. Do not
hand-edit IDs or hashes. If a deployment is abandoned (e.g. state mismatch), note it in the pull request but do not record the abandoned IDs as
live.

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
