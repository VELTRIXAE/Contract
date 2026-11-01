# Testnet deployment

> Deployment is **never automatic**. Every script that submits transactions
> asks for an explicit confirmation phrase (or the
> `--i-authorize-testnet-deploy` flag), refuses mainnet unconditionally, and
> uses keys only from the Stellar CLI keystore.

> **Not yet exercised against the network.** The scripts were written for
> Stellar CLI v28 and their argument handling and safety guards were tested
> locally, but no deployment has been run from this repository yet. Do a dry
> first run on testnet and check `deployments/testnet.json` by hand.

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

The script:

1. prints the plan and waits for you to type `DEPLOY TO TESTNET`
2. uploads the pair WASM (`stellar contract upload`) and keeps its hash
3. deploys the factory, calls `initialize(setter, pair_wasm_hash)`, then reads
   `fee_to_setter` back and **aborts if it does not match** (front-run check)
4. deploys the router, calls `initialize(factory)`, reads `get_factory` back
   and aborts on mismatch
5. writes `deployments/testnet.json` with the real contract IDs, WASM
   hashes, and the transaction hashes reported by the CLI

Non-interactive use (CI, scripted demos) requires the explicit flag:

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
hand-edit IDs or hashes. If a deployment is abandoned (e.g. failed front-run
check), note it in the pull request but do not record the abandoned IDs as
live.

## Mainnet

Out of scope. The scripts refuse `mainnet`/`public` network names. A mainnet
launch needs an independent audit, a multisig `fee_to_setter`, and a
separate, reviewed procedure.
